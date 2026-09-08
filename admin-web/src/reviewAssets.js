import { getAdminSession, expireAdminSession } from './session.js';

export async function fetchReviewAsset(publicationId, reference) {
  const token = getAdminSession().accessToken;
  if (!token) throw new Error('需要先登录');
  if (!Number.isSafeInteger(reference.size) || reference.size < 0 || reference.size > 5 * 1024 * 1024 || !/^[a-f0-9]{64}$/.test(reference.sha256)) throw new Error('附件引用无效');
  const base = import.meta.env.VITE_API_BASE || 'http://127.0.0.1:8787';
  const response = await fetch(`${base}/v1/admin/publications/${encodeURIComponent(publicationId)}/assets/${encodeURIComponent(reference.id)}`, { headers: { Authorization: `Bearer ${token}` }, redirect: 'error', signal: AbortSignal.timeout(45000) });
  if (response.status === 401) { expireAdminSession(token); throw new Error('登录已失效，请重新登录'); }
  if (!response.ok) throw new Error(`附件读取失败（${response.status}）`);
  const bytes = new Uint8Array(reference.size), reader = response.body.getReader(); let offset = 0;
  try {
    while (true) {
      const { value, done } = await reader.read(); if (done) break;
      if (offset + value.length > bytes.length) throw new Error('附件长度不一致');
      bytes.set(value, offset); offset += value.length;
    }
  } finally { await reader.cancel(); }
  const hash = [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))].map(n => n.toString(16).padStart(2, '0')).join('');
  if (offset !== reference.size || hash !== reference.sha256) throw new Error('附件内容校验失败');
  if (getAdminSession().accessToken !== token) throw new Error('会话已变化，请重新读取');
  return bytes;
}
