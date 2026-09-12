import { invokeCommand } from './tauri.js';

export const FILE_TYPES = {
  png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg', gif: 'image/gif', webp: 'image/webp',
  pdf: 'application/pdf', txt: 'text/plain', md: 'text/plain', csv: 'text/plain', json: 'text/plain',
  docx: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
  xlsx: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
  pptx: 'application/vnd.openxmlformats-officedocument.presentationml.presentation',
};
export const ASSET_ACCEPT = Object.keys(FILE_TYPES).map(ext => `.${ext}`).join(',');
const memory = new Map();
const native = () => Boolean(window.__TAURI_INTERNALS__);
export const resetMemoryAssets = () => memory.clear();
export const memoryAssets = id => structuredClone(memory.get(id) ?? []);
export function storeMemoryAssets(id, assets) { memory.set(id, JSON.parse(JSON.stringify(assets))); }
export const assetSize = asset => Math.floor(asset.data.length * 3 / 4) - (asset.data.endsWith('==') ? 2 : asset.data.endsWith('=') ? 1 : 0);
export const assetUrl = asset => `data:${asset.mime};base64,${asset.data}`;
export function formatBytes(size) { return size < 1024 ? `${size} B` : size < 1024 * 1024 ? `${Math.ceil(size / 1024)} KB` : `${(size / 1024 / 1024).toFixed(1)} MB`; }

export function validateAssets(assets) {
  if (!Array.isArray(assets) || assets.length > 12) throw new Error('每条提示词最多 12 个附件');
  let total = 0;
  const ids = new Set();
  for (const asset of assets) {
    if (!asset || !/^[\da-f]{8}(-[\da-f]{4}){3}-[\da-f]{12}$/i.test(asset.id) || ids.has(asset.id)) throw new Error('附件 id 无效或重复');
    ids.add(asset.id);
    if (typeof asset.name !== 'string' || !asset.name || [...asset.name].length > 255 || /[\x00-\x1f\x7f/\\]/.test(asset.name)) throw new Error('附件名称无效');
    const ext = asset.name.split('.').pop().toLowerCase();
    if (!FILE_TYPES[ext] || FILE_TYPES[ext] !== asset.mime) throw new Error(`不支持的附件类型：${asset.name}`);
    if (typeof asset.data !== 'string' || asset.data.length > 7 * 1024 * 1024 || asset.data.length % 4 || !/^[A-Za-z0-9+/]*={0,2}$/.test(asset.data)) throw new Error('附件编码或大小无效');
    const raw = atob(asset.data);
    total += raw.length;
    if (raw.length > 5 * 1024 * 1024 || total > 20 * 1024 * 1024) throw new Error('单文件上限 5 MiB，每条提示词附件总量上限 20 MiB');
    const valid = asset.mime === 'image/png' ? raw.startsWith('\x89PNG\r\n\x1a\n')
      : asset.mime === 'image/jpeg' ? raw.startsWith('\xff\xd8\xff')
      : asset.mime === 'image/gif' ? /^GIF8[79]a/.test(raw)
      : asset.mime === 'image/webp' ? raw.startsWith('RIFF') && raw.slice(8, 12) === 'WEBP'
      : asset.mime === 'application/pdf' ? raw.startsWith('%PDF-')
      : asset.mime === 'text/plain' ? !raw.includes('\0') : raw.startsWith('PK\x03\x04');
    if (!valid) throw new Error(`文件内容与类型不匹配：${asset.name}`);
    if (asset.mime === 'text/plain') new TextDecoder('utf-8', { fatal: true }).decode(Uint8Array.from(raw, c => c.charCodeAt(0)));
  }
  return assets;
}

export async function readAssetFiles(files, existing = []) {
  if (existing.length + files.length > 12) throw new Error('每条提示词最多 12 个附件');
  if (files.some(f => f.size > 5 * 1024 * 1024) || files.reduce((n, f) => n + f.size, existing.reduce((n, a) => n + assetSize(a), 0)) > 20 * 1024 * 1024) throw new Error('单文件上限 5 MiB，每条提示词附件总量上限 20 MiB');
  const additions = await Promise.all(files.map(file => new Promise((resolve, reject) => {
    const ext = file.name.split('.').pop().toLowerCase();
    if (!FILE_TYPES[ext]) { reject(new Error(`不支持的附件类型：${file.name}`)); return; }
    const reader = new FileReader();
    reader.onerror = () => reject(new Error(`读取失败：${file.name}`));
    reader.onload = () => resolve({ id: crypto.randomUUID(), name: file.name, mime: FILE_TYPES[ext], data: String(reader.result).split(',')[1] });
    reader.readAsDataURL(file);
  })));
  return validateAssets([...existing, ...additions]);
}

export async function listPromptAssets(promptId) {
  return native() ? invokeCommand('list_local_prompt_assets', { prompt_id: promptId }) : memoryAssets(promptId);
}

export async function firstPromptImage(promptId) {
  if (native()) return invokeCommand('get_local_prompt_image', { prompt_id: promptId });
  const asset = memory.get(promptId)?.find(asset => asset.mime.startsWith('image/'));
  return asset ? structuredClone(asset) : null;
}

// Browser-only memory adapter; the desktop resizes in a bounded native worker.
export async function firstPromptThumbnail(promptId) {
  if (native()) return invokeCommand('get_local_prompt_thumbnail', { prompt_id: promptId });
  const asset = await firstPromptImage(promptId);
  if (!asset) return null;
  const image = new Image(); image.decoding = 'async'; image.src = assetUrl(asset);
  await image.decode();
  const scale = Math.min(1, 480 / Math.max(image.naturalWidth, image.naturalHeight));
  const canvas = document.createElement('canvas');
  canvas.width = Math.max(1, Math.round(image.naturalWidth * scale));
  canvas.height = Math.max(1, Math.round(image.naturalHeight * scale));
  canvas.getContext('2d').drawImage(image, 0, 0, canvas.width, canvas.height);
  return { ...asset, name: 'thumbnail.png', mime: 'image/png', data: canvas.toDataURL('image/png').split(',')[1] };
}

export async function exportPromptAsset(promptId, asset) {
  if (native()) return invokeCommand('export_local_prompt_asset', { prompt_id: promptId, asset_id: asset.id });
  const bytes = Uint8Array.from(atob(asset.data), c => c.charCodeAt(0));
  const url = URL.createObjectURL(new Blob([bytes], { type: asset.mime }));
  const anchor = document.createElement('a'); anchor.href = url; anchor.download = asset.name;
  anchor.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
  return '已交给浏览器下载';
}

export function textPreview(asset) {
  if (asset.mime !== 'text/plain') return '';
  const bytes = Uint8Array.from(atob(asset.data), c => c.charCodeAt(0));
  const text = new TextDecoder().decode(bytes.slice(0, 12000));
  return text + (bytes.length > 12000 ? '\n…仅预览前 12 KB，导出可查看完整文件。' : '');
}
