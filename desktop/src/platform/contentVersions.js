import { getSession } from './session.js';
import { invokeCommand } from './tauri.js';
import { clearThumbnailCache } from './thumbnailCache.js';
import { apiBase } from '../../../shared/apiBase.js';
export async function contentVersions(kind, id, version = null) {
  const access_token = getSession().accessToken || null;
  if (window.__TAURI_INTERNALS__) return invokeCommand('content_versions_request', { kind, id, version, access_token });
  const response = await fetch(`${apiBase()}/v1/content/${kind}/${encodeURIComponent(id)}/versions${version === null ? '' : '/' + version}`, {
    headers: access_token ? { Authorization: `Bearer ${access_token}` } : {}, signal: AbortSignal.timeout(15000),
  });
  if (!response.ok) throw Error(response.status === 404 ? '内容已下架或尚无版本记录' : '读取版本失败，请重试');
  return response.json();
}

export async function localContentUpdate(request) {
  if (!window.__TAURI_INTERNALS__) throw Error('下载版本更新请在桌面客户端操作');
  const result=await invokeCommand('content_update_request', { request });
  if(['import','apply','restore_copy'].includes(request.action))clearThumbnailCache();
  return result;
}

export async function prepareVersionDownload(revision, onProgress) {
  const { validateReferences, validateCollectionAssets, downloadPublishedAsset } = await import('./privateMedia.js');
  const { listLocalCategories } = await import('./library.js');
  const { downloadReferenceImages } = await import('./referenceAssets.js');
  const source = revision.snapshot, refs = validateReferences(source.asset_refs ?? []);
  const categoryIds = new Set((await listLocalCategories()).map(c => c.id));
  const category = value => categoryIds.has(value) ? value : null;
  if (revision.kind === 'collection') validateCollectionAssets(source.members, refs, source.cover);
  const assets = new Map();
  for (const ref of refs) {
    onProgress?.({ stage: 'downloading', completed: assets.size, total: refs.length });
    assets.set(ref.id, { ...await downloadPublishedAsset(revision.id, ref, getSession().accessToken, revision.version), id: crypto.randomUUID() });
  }
  if (revision.kind === 'collection') return {
    collections: [{ title: source.title, category_id: category(source.category_id), cover_type: source.cover?.layout || 'none', cover_json: JSON.stringify((source.cover?.asset_ids || []).map(id => { const a = assets.get(id); return `data:${a.mime};base64,${a.data}`; })) }],
    prompts: source.members.map(m => ({ remote_member_id: m.member_id ?? null, title: m.title, content: m.content, category_id: category(m.category_id), model: m.model, assets: (m.asset_ids || []).map(id => assets.get(id)) })),
  };
  return { prompts: [{ title: source.title, content: source.content || '', category_id: category(source.category_id), model: source.model, assets: [...assets.values(), ...await downloadReferenceImages(source, onProgress)] }] };
}

export async function withdrawPublication(id) {
  const token=getSession().accessToken;
  if(!token)throw Error('请先登录');
  if(window.__TAURI_INTERNALS__)return invokeCommand('withdraw_publication',{id,access_token:token});
  const response=await fetch(`${apiBase()}/v1/publications/${encodeURIComponent(id)}/withdraw`,{method:'POST',headers:{Authorization:`Bearer ${token}`},signal:AbortSignal.timeout(15000)});
  if(!response.ok)throw Error('撤回失败，投稿可能已经完成审核，请刷新后查看');
  return response.json();
}
