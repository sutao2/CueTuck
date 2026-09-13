// CLI-driven browser smoke: fresh snapshots supply every interaction target.
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
const exec = promisify(execFile);
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const artifacts = resolve(root, 'output/playwright', `global-search-${Date.now()}`);
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


  await run('resize', '1280', '850');
  await run('run-code', `async page => { await page.evaluate(async () => {
    const library = await import('/src/platform/library.js');
    const square = await import('/src/platform/square.js');
    await library.createLocalPrompt({title:'产品发布方案',content:'根据产品定位与目标用户，设计清晰的发布计划。',categoryId:'cat-software'});
    await library.createLocalPrompt({title:'产品反馈整理',content:'分析用户反馈，提取需求与改进建议。',categoryId:'cat-software'});
    await library.createLocalCollection({title:'产品团队常用合集'});
    window.globalSearchRemoteCalls = 0;
    square.setCatalogTransport(async () => ({categories:[],models:[]}));
    square.setSquarePageTransport(async ({query}) => {
      if (!query) return {items:[],total:0,next_offset:null};
      window.globalSearchRemoteCalls++;
      if (query==='离线') throw Error('当前无法连接广场');
      return {items:[{id:'global-remote',title:'社区产品策划',kind:'prompt',content:'为不同用户群体设计产品体验。'}],total:1,next_offset:null};
    });
    square.setSquareContentTransport(async () => ({content:'社区完整正文，已加载成功。'}));
  }); }`);
  await click(/button ".*未分类/);
  await fill(/searchbox "搜索标题或正文"/, '保留页面筛选');
  await run('press', 'Meta+k'); await target(/heading "全局搜索"/);
  await fill(/combobox "全局搜索关键词"/, '产品'); await target(/option "产品发布方案/);
  assert.match(await run('eval', 'window.globalSearchRemoteCalls === 0'), /true/);
  await run('screenshot');
  assert.match(await run('eval', 'document.querySelector(".workspace").inert === true'), /true/);
  await run('press','ArrowDown'); await run('press','Enter');
  await target(/heading "产品发布方案"/);
  await click(/button "(?:← )?返回"/);
  assert.match(await run('eval', 'document.querySelector(".inline-search input").value === "保留页面筛选"'), /true/);
  await click(/button "切换深色主题"/); await run('resize','800','700');
  await click(/button "全局搜索/); await fill(/combobox "全局搜索关键词"/, '产品'); await target(/option "产品发布方案/); await run('screenshot');
  assert.match(await run('eval', 'document.documentElement.scrollWidth <= innerWidth'), /true/);
  await run('press','Escape'); await click(/button "新建"/);
  await fill(/textbox "标题"/, '未保存的产品草稿');
  await run('press','Meta+k'); await fill(/combobox "全局搜索关键词"/, '产品'); await target(/option "产品发布方案/); await run('press','Escape');
  assert.match(await run('eval', 'document.querySelector("[data-testid=prompt-editor] input").value === "未保存的产品草稿"'), /true/);
  await run('press','Meta+k'); await fill(/combobox "全局搜索关键词"/, '产品'); await click(/option "产品发布方案/);
  await target(/heading "放弃未保存的修改/); await click(/button "继续编辑"/);
  assert.match(await run('eval', 'document.querySelector("[data-testid=prompt-editor] input").value === "未保存的产品草稿"'), /true/);
  await click(/button "(?:← )?返回"/); await click(/button "放弃修改"/);
  await click(/button "全局搜索/); await click(/button "提示词广场"/);
  await fill(/combobox "全局搜索关键词"/, '离线'); await target(/button "重试搜索"/); await run('screenshot');
  await fill(/combobox "全局搜索关键词"/, '产品'); await target(/option "社区产品策划/); await run('screenshot');
  await click(/option "社区产品策划/); await target(/generic.*社区完整正文/);
  await run('screenshot');
  assert.match(await run('eval', 'document.querySelector("[data-testid=square-detail] .page-back").textContent.includes("返回本地列表")'), /true/);
  await click(/button "返回"/);
  assert.match(await run('eval', 'document.querySelector(".inline-search input").value === "保留页面筛选"'), /true/);
  console.log('Global search browser passed: separate page filters, local keyboard selection, draft cancellation/guard, square error/results, light/dark and 800px.');
} finally {
  await writeFile(resolve(artifacts, 'actions.log'), actions);
  await writeFile(resolve(artifacts, 'vite.log'), serverError);
  await run('close').catch(() => {});
  server.kill('SIGTERM');
  console.log(`Artifacts: ${artifacts}`);
}
