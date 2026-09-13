// CLI-driven browser smoke: fresh snapshots supply every interaction target.
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
const exec = promisify(execFile);
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const artifacts = resolve(root, 'output/playwright', `metrics-${Date.now()}`);
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
  for (let attempt = 0; attempt < 100 && !serverError.includes('Local:'); attempt++) {
    if (server.exitCode !== null) throw Error(serverError);
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  assert.match(serverError, /Local:/);
  await run('open', 'about:blank', '--browser', 'chrome');
  await run('route', '**/v1/**', '--status', '503', '--body', '{}');
  await run('goto', origin);
  await run('resize', '1440', '1000');
  await run('run-code', `async page => {
    await page.route('https://cms-assets.youmind.com/metrics-fixture.png', route => route.fulfill({status:200,contentType:'image/png',path:${JSON.stringify(resolve(root,'desktop/src/assets/app-icon.png'))}}));
    await page.evaluate(async () => {
      const session = await import('/src/platform/session.js');
      const square = await import('/src/platform/square.js');
      session.setSessionTransport(async () => ({ email: 'writer@example.com', access_token: 'fixture-account' }));
      square.setMineTransport(async () => [
        {id:'new',title:'写给产品发布的叙事框架',status:'pending',download_count:0,favorite_count:0},
        {id:'hot',title:'从研究资料到一份清晰的内容提纲',status:'approved',visibility:'online',download_count:1234,favorite_count:86}
      ]);
      square.setCatalogTransport(async () => ({categories:[],models:[]}));
      square.setSquarePageTransport(async ({sort}) => {
        const items = Array.from({length:48},(_,i)=>({id:'metrics-'+i,title:'从研究资料到一份清晰的内容提纲 '+i,kind:'prompt',excerpt:'用清晰的结构整理材料，提炼重点，连接观点，输出适合分享的提纲。'.repeat(8),model:'ChatGPT',download_count:1234+i,favorite_count:86+i,reference:i%3===0?{images:['https://cms-assets.youmind.com/metrics-fixture.png']}:null}));
        if(sort==='热门') items.reverse();
        return {items,total:48,next_offset:null};
      });
    });
  }`);
  await click(/tab "提示词广场"/);
  await click(/tab "热门"/);
  await target(/button "从研究资料到/);
  assert.match(await run('eval', 'document.querySelector(".square-sort-note").textContent'), /按已记录下载量/);
  const geometry = '[...document.querySelectorAll(".prompt-card")].every(card => { const r=card.getBoundingClientRect(); return [...card.querySelectorAll(".square-card-metrics,.card-footer")].every(el=>{const c=el.getBoundingClientRect();return c.bottom<=r.bottom+1&&c.right<=r.right+1;}); })';
  assert.match(await run('eval', geometry), /true/);
  await run('screenshot');
  await click(/button "列表视图"/);
  assert.match(await run('eval', geometry), /true/);
  await run('screenshot');
  await click(/button "游 登录"/);
  await fill(/textbox "邮箱"/, 'writer@example.com');
  await fill(/textbox "密码"/, 'fixture');
  await click(/button "登录"/);
  await click(/button "设置 ›"/);
  await click(/button "账号与广场"/);
  await click(/button "查看我的发布/);
  await target(/heading "我的发布"/);
  assert.match(await run('eval', 'document.querySelector(".publication-metrics").textContent'), /1,234/);
  await run('select', await target(/combobox "作品排序"/), 'download_count');
  assert.match(await run('eval', 'document.querySelector(".publication-heading strong").textContent'), /研究资料/);
  await run('screenshot');
  await run('run-code', 'async page => { await page.evaluate(() => document.body.classList.add("theme-dark")); }');
  await run('resize','800','850');
  assert.match(await run('eval', 'document.querySelector(".publication-metrics").scrollWidth <= document.querySelector(".publication-metrics").clientWidth + 1'), /true/);
  await run('screenshot');
  console.log('Metrics browser passed: card grid/list geometry, hot explanation, author totals/sort, dark/narrow view.');
} finally {
  await writeFile(resolve(artifacts, 'actions.log'), actions);
  await writeFile(resolve(artifacts, 'vite.log'), serverError);
  await run('close').catch(() => {});
  server.kill('SIGTERM');
  console.log(`Artifacts: ${artifacts}`);
}
