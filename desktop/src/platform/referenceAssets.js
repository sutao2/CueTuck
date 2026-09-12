import { invokeCommand } from './tauri.js';
import { validateAssets } from './assets.js';
import { referenceImages } from '../lib/squareReference.js';

export async function downloadReferenceImages(item) {
  const assets = [];
  for (const [index,url] of referenceImages(item).entries()) {
    try {
      let asset;
      if (window.__TAURI_INTERNALS__) asset = await invokeCommand('download_reference_image', { url, index });
      else {
        const response = await fetch(url, { credentials:'omit', referrerPolicy:'no-referrer', redirect:'error', signal:AbortSignal.timeout(45000) });
        if (!response.ok) throw Error('图片连接失败');
        const reader = response.body.getReader(), chunks = []; let size = 0;
        try {
          while (true) {
            const {value,done} = await reader.read(); if (done) break;
            size += value.length; if (size > 5 * 1024 * 1024) throw Error('图片超过 5 MiB');
            chunks.push(value);
          }
        } finally { await reader.cancel(); }
        let raw = ''; for (const bytes of chunks) for (let i=0;i<bytes.length;i+=8192) raw += String.fromCharCode(...bytes.subarray(i,i+8192));
        const type = raw.startsWith('\x89PNG\r\n\x1a\n') ? ['png','image/png'] : raw.startsWith('\xff\xd8\xff') ? ['jpg','image/jpeg'] : /^GIF8[79]a/.test(raw) ? ['gif','image/gif'] : raw.startsWith('RIFF') && raw.slice(8,12)==='WEBP' ? ['webp','image/webp'] : null;
        if (!type) throw Error('不支持的图片内容');
        asset = {id:crypto.randomUUID(),name:`参考图-${index+1}.${type[0]}`,mime:type[1],data:btoa(raw)};
      }
      assets.push(asset); validateAssets(assets);
    } catch (error) { throw Error(`参考图 ${index+1} 下载失败：${error.message || error}，请重试`); }
  }
  return assets;
}
