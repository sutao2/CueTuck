// CLI-driven browser smoke: fresh snapshots supply every interaction target.
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
const exec = promisify(execFile);
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const artifacts = resolve(root, 'output/playwright', `design-${Date.now()}`);
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
  await run('run-code', `async page => {
    await page.route('https://cms-assets.youmind.com/visual-design.png', route => route.fulfill({status:200,contentType:'image/png',path:${JSON.stringify(resolve(root,'desktop/src/assets/app-icon.png'))},headers:{'access-control-allow-origin':'*'}}));
    await page.evaluate(async () => {
    const library = await import('/src/platform/library.js');
    const square = await import('/src/platform/square.js');
    const data = await fetch('/src/assets/app-icon.png').then(r => r.blob());
    const encoded = await new Promise(resolve => { const reader = new FileReader(); reader.onload = () => resolve(reader.result.split(',')[1]); reader.readAsDataURL(data); });
    const samples = ['产品发布：从想法到一份可执行的方案', '整理研究资料与关键发现', '为独立品牌制定内容计划', '审阅代码并解释修改建议'];
    const content = '结合上下文与约束条件，输出清晰、有依据的建议，说明取舍并给出下一步行动。'.repeat(10) + '{{目标}}';
    for (let i=0;i<8;i++) await library.createLocalPrompt({ title: samples[i%4], content, model: 'ChatGPT · 长模型名称测试', categoryId: 'cat-software', assets: i%3===0 ? [{ id: crypto.randomUUID(), name:'cover.png', mime:'image/png', data:encoded }] : [] });
    await library.createLocalCollection({title:'团队常用模板'});
    square.setCatalogTransport(async () => ({ categories: [], models: [] }));
    square.setSquareTransport(async () => Array.from({length:48}, (_,i) => ({ id:'visual-'+i,kind:'prompt',title:samples[i%4],content,model:'ChatGPT · 长模型名称测试',author:'创作者名字也可能较长',reference:i%3===0?{images:['https://cms-assets.youmind.com/visual-design.png']}:null })));
  }); }`);
  async function checkCards() {
    assert.match(await run('eval', 'document.documentElement.scrollWidth <= innerWidth'), /true/);
    const report = await run('eval', '[...document.querySelectorAll("[data-region=content] .prompt-card")].every(card => { const rect = card.getBoundingClientRect(); const footer = card.querySelector(".card-footer").getBoundingClientRect(); const excerpt = card.querySelector(".prompt-excerpt"); const lines = excerpt.getBoundingClientRect().height / parseFloat(getComputedStyle(excerpt).lineHeight); return footer.bottom <= rect.bottom + 1 && footer.right <= rect.right + 1 && Math.abs(lines - Math.round(lines)) < .1; })');
    assert.match(report, /true/);
  }
  await click(/tab "提示词广场"/); await target(/button "产品发布/); await run('screenshot'); await checkCards();
  await click(/button "列表视图"/); await run('screenshot'); await checkCards(); await click(/button "网格视图"/);
  await click(/tab "本地提示词"/); await target(/button "团队常用模板"/); await run('screenshot'); await checkCards();
  await click(/button "切换深色主题"/); await run('screenshot');
  await run('resize', '800','700'); await run('screenshot'); await checkCards();
  await click(/button "列表视图"/); await run('screenshot'); await checkCards();
  await click(/button "产品发布/); await click(/button "编辑"/); await run('screenshot');
  await click(/button "试填预览"/); await fill(/textbox "目标"/, '清晰的品牌介绍');
  await run('resize','1280','850'); await run('screenshot');
  assert.match(await run('eval','document.documentElement.scrollWidth <= innerWidth'), /true/);
  await click(/button "切换浅色主题"/); await run('screenshot');
  await click(/button "保存"/); await click(/button "(?:← )?返回"/);
  console.log('Design browser passed: mixed cards, metadata/actions fit, light/dark, narrow grid/list, editing/trial/save/return.');
} finally {
  await writeFile(resolve(artifacts, 'actions.log'), actions);
  await writeFile(resolve(artifacts, 'vite.log'), serverError);
  await run('close').catch(() => {});
  server.kill('SIGTERM');
  console.log(`Artifacts: ${artifacts}`);
}
