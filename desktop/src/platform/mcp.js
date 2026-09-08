import { invokeCommand } from './tauri.js';

export function mcpConfig({ command, library_dir }, { square = false, base = 'http://127.0.0.1:8787' } = {}) {
  if (![command, library_dir].every(path => typeof path === 'string' && (/^\//.test(path) || /^[A-Za-z]:[\\/]/.test(path)))) throw new Error('程序与库目录必须使用绝对路径');
  const env = { PROMPTARK_LIBRARY_DIR: library_dir, PROMPTARK_MCP_SQUARE: square ? '1' : '0' };
  if (square) {
    let url; try { url = new URL(base); } catch { throw new Error('广场地址无效'); }
    if (!(url.protocol === 'https:' || (url.protocol === 'http:' && ['localhost','127.0.0.1','[::1]'].includes(url.hostname))) || url.username || url.password || url.pathname !== '/' || url.search || url.hash) throw new Error('请填写 HTTPS 或本机 HTTP 站点地址，不含凭据、路径或参数');
    env.PROMPTARK_MCP_API_BASE = url.origin;
  }
  return JSON.stringify({ mcpServers: { promptark: { command, env } } }, null, 2);
}
export async function prepareMcpConfig(executable, options) {
  if (!window.__TAURI_INTERNALS__) throw new Error('请在桌面端生成配置，浏览器无法读取本机程序和库路径');
  return mcpConfig(await invokeCommand('mcp_connection_info', { executable: executable.trim() }), options);
}
