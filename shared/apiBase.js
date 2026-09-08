// All browser clients use the same build-time service origin.
export function normalizeApiBase(value = 'http://127.0.0.1:8787') {
  let url;
  try { url = new URL(value.trim()); } catch { throw Error('服务地址无效'); }
  const local = ['localhost', '127.0.0.1', '[::1]'].includes(url.hostname);
  if (!['http:', 'https:'].includes(url.protocol) || (!local && url.protocol !== 'https:') || url.username || url.password || url.pathname !== '/' || url.search || url.hash) {
    throw Error('服务地址必须为 HTTPS origin（本机可用 HTTP），不能包含凭据、路径或查询参数');
  }
  return url.origin;
}

export function apiBase() {
  return normalizeApiBase(import.meta.env?.VITE_API_BASE || 'http://127.0.0.1:8787');
}
