import { afterEach, expect, it } from 'vitest';
import { mcpConfig, prepareMcpConfig } from './mcp.js';
afterEach(() => { delete window.__TAURI_INTERNALS__; });
it('generates escaped absolute paths and explicitly disables inherited network opt-in by default', () => {
  const paths = { command: '/path with spaces/"mcp"', library_dir: '/private/library' };
  const server = JSON.parse(mcpConfig(paths)).mcpServers.promptark;
  expect(server.command).toBe(paths.command); expect(server.env).toEqual({PROMPTARK_LIBRARY_DIR:paths.library_dir,PROMPTARK_MCP_SQUARE:'0'});
});
it('enables only the selected public origin and rejects unsafe configurations', () => {
  const paths = {command:'/bin/mcp',library_dir:'/library'};
  const env=JSON.parse(mcpConfig(paths,{square:true})).mcpServers.promptark.env;
  expect(env.PROMPTARK_MCP_SQUARE).toBe('1'); expect(env.PROMPTARK_MCP_API_BASE).toBe('http://127.0.0.1:8787');
  for(const base of ['http://example.com','https://user:secret@example.com','https://example.com/path','https://example.com?token=a','file:///tmp']) expect(()=>mcpConfig(paths,{square:true,base})).toThrow();
  expect(()=>mcpConfig({command:'~/mcp',library_dir:'/library'})).toThrow();
});
it('does not invent native paths in browser previews', async () => { await expect(prepareMcpConfig('/bin/mcp')).rejects.toThrow('桌面端'); });
