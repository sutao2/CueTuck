// CLI-driven browser smoke: fresh snapshots supply every interaction target.
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
const exec = promisify(execFile);
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const artifacts = resolve(root, 'output/playwright', `smoke-${Date.now()}`);
await mkdir(artifacts, { recursive: true });
const cli = resolve(root, 'desktop/node_modules/@playwright/cli/playwright-cli.js');
const session = `promptark-smoke-${process.pid}`;
const origin = 'http://127.0.0.1:1431';
let snapshot = '', serverError = '', actions = '';
const server = spawn(process.execPath, [resolve(root, 'desktop/node_modules/vite/bin/vite.js'), '--host', '127.0.0.1', '--port', '1431', '--strictPort'], { cwd: resolve(root, 'desktop'), stdio: ['ignore', 'pipe', 'pipe'] });
server.stderr.on('data', chunk => { serverError += chunk; });
server.stdout.on('data', chunk => { serverError += chunk; });

async function run(...args) {
  const { stdout } = await exec(process.execPath, [cli, `-s=${session}`, ...args], { cwd: artifacts, timeout: 60000, maxBuffer: 2 * 1024 * 1024 });
  actions += stdout;
  if (/### Error/.test(stdout)) throw Error(stdout);
  const path = stdout.match(/\[Snapshot\]\(([^)]+)\)/)?.[1];
  if (path) snapshot = await readFile(resolve(artifacts, path), 'utf8');
  const inline = stdout.match(/```yaml\n([\s\S]*?)```/)?.[1];
  if (inline) snapshot = inline;
  return stdout;
}
async function target(pattern, last = false) {
  for (let attempt = 0; attempt < 8; attempt++) {
    await run('snapshot');
    const lines = snapshot.split('\n').filter(line => pattern.test(line) && !line.includes('[disabled]'));
    const line = last ? lines.at(-1) : lines[0];
    if (line) return line.match(/\[ref=(e\d+)\]/)?.[1];
  }
  throw Error(`Target missing: ${pattern}\n${snapshot}`);
}
const click = async (pattern, last = false) => run('click', await target(pattern, last));
const fill = async (pattern, value) => run('fill', await target(pattern), value);

try {
  // Do not attach to a pre-existing service on this port.
  for (let attempt = 0; attempt < 100 && !serverError.includes('Local:'); attempt++) {
    if (server.exitCode !== null) throw Error(serverError);
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  assert.match(serverError, /Local:/);
  await run('open', 'about:blank', '--browser', 'chrome');
  // Network and clipboard adapters are test-only, scoped to this fresh browser.
  await run('route', '**/v1/**', '--status', '503', '--body', '{}');
  await run('run-code', `async page => {
    await page.addInitScript(() => { window.smokeClipboard = ''; Object.defineProperty(navigator, 'clipboard', { value: { writeText: async text => { window.smokeClipboard = text; } }, configurable: true }); });
  }`);
  await run('goto', origin);
  await click(/button "新建"/);
  await fill(/textbox "标题"/, 'E2E 临时提示词');
  await fill(/textbox "提示词内容"/, '你好 {{姓名}}');
  await click(/button "保存"/);
  await target(/button "E2E 临时提示词"/);
  await click(/button "E2E 临时提示词"/);
  await click(/button "编辑"/);
  await fill(/textbox "提示词内容"/, '欢迎 {{姓名}}');
  await click(/button "保存"/);
  await click(/button "使用提示词"/);
  await fill(/textbox/, 'Ada');
  await click(/button "下一步"/);
  assert.match(snapshot, /欢迎 Ada/);
  await click(/button "复制并完成"/);
  assert.match(await run('eval', 'window.smokeClipboard'), /欢迎 Ada/);
  await click(/button "编辑"/);
  await click(/button "删除"/);
  await click(/button "取消"/, true);
  assert.match(snapshot, /E2E 临时提示词/);
  await click(/button "删除"/);
  await click(/button "确认删除"/);
  await target(/本地库是空的/);
  await click(/button "切换深色主题"/);
  await run('screenshot');
  await click(/button "设置 ›"/);
  await target(/搜索设置/);
  await run('resize', '800', '600');
  await run('screenshot');
  await click(/button "返回应用"/);
  await target(/本地库是空的/);
  // Isolated reference-image fixture. No real public download or local desktop write.
  await run('run-code', `async page => {
    await page.route('https://cms-assets.youmind.com/image-smoke.png',route=>route.fulfill({status:200,contentType:'image/png',path:${JSON.stringify(resolve(root,'desktop/src/assets/app-icon.png'))},headers:{'access-control-allow-origin':'*'}}));
    await page.evaluate(async()=>{
      const square=await import('/src/platform/square.js');
      const item={id:'image-smoke',title:'图片下载验收',kind:'prompt',content:'只复制正文',reference:{images:['https://cms-assets.youmind.com/image-smoke.png']}};
      square.setSquareTransport(async()=>[item]);square.setSquareContentTransport(async()=>item);square.setCatalogTransport(async()=>({categories:[],models:[]}));
    });
  }`);
  await click(/tab "提示词广场"/);await click(/button "图片下载验收"/);await click(/button "查看大图"/);
  await click(/button "100%"/);await run('screenshot');await run('press','Escape');await target(/heading "图片下载验收"/);
  await click(/button "下载到本地"/);await target(/button "补全参考图"/);
  await click(/button "补全参考图"/);await target(/参考图已补全/);
  await run('run-code',`async page=>{await page.evaluate(async()=>{const lib=await import('/src/platform/library.js');const rows=await lib.listLocalPrompts();const row=rows.find(row=>row.remote_id==='image-smoke');const assets=await (await import('/src/platform/assets.js')).listPromptAssets(row.id);if(assets.length!==1||row.content!=='只复制正文')throw Error('Image import or dedup failed');});}`);
  await click(/button "返回"/);await click(/tab "本地提示词/);
  await target(/button "查看 图片下载验收 的图片"/);await run('screenshot');
  await click(/button "查看 图片下载验收 的图片"/);await target(/dialog "图片预览：图片下载验收"/);await run('press','Escape');
  await click(/button "列表视图"/);await target(/button "查看 图片下载验收 的图片"/);await run('screenshot');
  assert.match(await run('eval',`(()=>{const cover=document.querySelector('.local-prompt-cover'),card=cover.closest('article');return JSON.stringify({width:cover.getBoundingClientRect().width,row:card.classList.contains('as-row'),overflow:document.documentElement.scrollWidth>innerWidth});})()`),/\\"width\\":80,\\"row\\":true,\\"overflow\\":false/);
  await click(/button "图片下载验收"/);await click(/button "查看 参考图-1.png"/);
  await click(/button "放大图片"/);await run('screenshot');await run('resize','600','700');await click(/button "适应窗口"/);await run('screenshot');await run('press','Escape');await target(/heading "图片下载验收"/);
  // New reading / writing / organizing flows use only the fresh browser memory library.
  await run('resize', '1280', '850');
  await click(/button "(?:← )?返回"/);
  await click(/button "新建"/);
  await fill(/textbox "标题"/, '交互验收');
  await fill(/textbox "提示词内容"/, '{{城市}} {{预算}}');
  await click(/button "试填预览"/);
  await fill(/textbox "城市"/, '京都'); await fill(/textbox "预算"/, '1000');
  await target(/京都 1000/); await run('screenshot');
  await run('resize', '800', '700'); await run('screenshot');
  assert.match(await run('eval', 'document.documentElement.scrollWidth <= innerWidth'), /true/);
  await click(/button "保存"/); await click(/button "交互验收"/);
  await click(/button "使用提示词"/); await click(/button "2 预算 待填"/);
  await fill(/textbox "预算"/, '2000'); await click(/button "预览"/);
  await target(/还有 1 项未填写/);
  await click(/button "1 城市 待填"/); await fill(/textbox "城市"/, '上海');
  await click(/button "更新预览"/); await target(/上海 2000/); await run('screenshot');
  await click(/button "复制并完成"/); await click(/button "(?:← )?返回"/);
  await click(/button "批量整理"/); await run('check', await target(/checkbox "选择 交互验收"/));
  await run('select', await target(/combobox "目标分类"/), 'cat-image');
  await click(/button "应用到所选"/); await target(/已完成 1 条/); await run('screenshot');
  await click(/button "取消多选"/);
  await click(/button "新建"/); await click(/button "提示词合集/); await fill(/textbox "合集名称"/, '验收合集');
  await click(/button "创建合集"/); await click(/button "验收合集"/);
  await fill(/searchbox "搜索并加入提示词"/, '交互验收');
  await run('check', await target(/checkbox "交互验收"/));
  await click(/button "加入合集"/); await target(/button "交互验收"/); await run('screenshot');
  await click(/button "(?:← )?返回"/);
  await click(/button "设置 ›"/); await click(/button "数据与备份"/); await run('screenshot');
  const importFile = resolve(artifacts, 'import.json');
  await writeFile(importFile, JSON.stringify({ prompts: [{ title: '文件选择验收', content: '仅写隔离内存库' }] }));
  await click(/button "选择 JSON 文件"/); await run('upload', importFile);
  await target(/将导入 1 条提示词/); await run('screenshot');
  await click(/button "确认导入"/); await target(/导入完成/);

  await click(/button "返回应用"/);
  await run('run-code', `async page => { await page.evaluate(async () => {
    const session = await import('/src/platform/session.js');
    session.setSessionTransport(async () => ({ email: 'qa@example.test', access_token: 'isolated-test' }));
    await session.loginSession({ email: 'qa@example.test', password: 'test' });
    (await import('/src/platform/square.js')).setMineTransport(async () => [{ id: 'qa', title: '隔离验收投稿', status: 'pending' }]);
  }); }`);
  await click(/button "游 登录"/); await click(/button "查看我的发布"/);
  await target(/隔离验收投稿/); await run('screenshot');
  await click(/button "(?:← )?返回"/); await click(/button "切换浅色主题"/);
  await run('resize', '1280', '850'); await run('screenshot');
  // Fresh, isolated multi-page fixture; never touch the native library.
  await run('run-code', `async page => { await page.evaluate(async () => {
    const library = await import('/src/platform/library.js'); library.resetMemoryLibrary();
    await library.createLocalCollection({ title: '分页合集' });
    for (let i = 0; i < 50; i++) await library.createLocalPrompt({ title: '跨页 ' + i, content: '连续操作验收' });
  }); }`);
  await click(/tab "提示词广场"/); await click(/tab "本地提示词(?: \d+)?"/);
  await click(/button "批量整理"/); await click(/button "选择本页"/); await target(/已选 47 条/);
  await click(/button "下一页"/); await target(/已选 47 条/);
  await click(/button "选择本页"/); await target(/已选 50 条/);
  await run('resize', '800', '700'); await run('screenshot');
  await click(/button "应用到所选"/); await target(/已完成 50 条/); await target(/第 2 \/ 2 页/);
  await click(/button "取消多选"/); await click(/button "跨页 \d+"/); await click(/button "编辑"/);
  await fill(/textbox "提示词内容"/, '保留第二页'); await click(/button "保存"/);
  await click(/button "(?:← )?返回"/); await target(/第 2 \/ 2 页/);
  await click(/button "列表视图"/); await click(/button "批量整理"/); await click(/button "选择本页"/);
  await target(/已选 3 条/); await run('screenshot');
  const overflow = await run('eval', 'document.documentElement.scrollWidth > window.innerWidth');
  assert.match(overflow, /false/);
  console.log('Browser smoke passed: create/edit/variables/copy/delete-cancel/delete/settings-return/image-download/supplement/zoom/narrow-viewer/trial-preview/parameter-navigation/batch-category/searchable-collection/file-import/publications/light-and-dark/cross-page-selection/return-position.');
} finally {
  await writeFile(resolve(artifacts, 'actions.log'), actions);
  await writeFile(resolve(artifacts, 'vite.log'), serverError);
  await run('close').catch(() => {});
  server.kill('SIGTERM');
  console.log(`Artifacts: ${artifacts}`);
}
