// CLI-driven browser smoke: fresh snapshots supply every interaction target.
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
const exec = promisify(execFile);
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const artifacts = resolve(root, 'output/playwright', `account-${Date.now()}`);
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
  await run('run-code', `async page => { await page.evaluate(async () => {
    const session = await import('/src/platform/session.js');
    const billing = await import('/src/platform/billing.js');
    session.setSessionTransport(async () => ({ email: 'writer@example.com', access_token: 'fixture-account' }));
    let profile = { display_name: '林晚', bio: '分享写作、设计与日常工作中好用的提示词。' };
    session.setMeTransport({ get: async () => profile, put: async body => { profile = body; return profile; } });
    billing.setBillingTransport({ status: async () => ({ pro: false, mock: true, mock_pro: false, note: '仅模拟，不扣款，不改变真实权益。' }) });

  }); }`);
  await click(/button "游 登录"/);
  await fill(/textbox "邮箱"/, "writer@example.com");
  await fill(/textbox "密码"/, "fixture");
  await click(/button "登录"/);
  await click(/button "设置 ›"/);
  await click(/button "账号与广场"/);
  await target(/heading "林晚"/);
  assert.match(await run('eval', 'document.querySelector(".account-billing-details").open === false'), /true/);
  await run('screenshot');
  await fill(/textbox "显示名"/, '创作笔记');
  assert.match(await run('eval', 'document.querySelector("[data-testid=account-name]").textContent'), /林晚/);
  await click(/button "保存资料"/);
  await target(/heading "创作笔记"/);
  await click(/.*模拟测试与测试码/);
  await target(/button "模拟成功"/);
  await click(/.*模拟测试与测试码/);
  await run('run-code', `async page => { await page.evaluate(() => document.body.classList.add('theme-dark')); }`);
  await run('screenshot');
  await run('resize', '800', '850');
  assert.match(await run('eval', '[...document.querySelectorAll(".account-grid, .account-card, .account-overview")].every(el => el.scrollWidth <= el.clientWidth + 1)'), /true/);
  assert.match(await run('eval', 'getComputedStyle(document.querySelector(".account-grid")).gridTemplateColumns.split(" ").length === 1'), /true/);
  await run('screenshot');
  console.log('Account browser passed: saved identity, profile save, collapsed mock actions, light/dark, 800px no card overflow.');
} finally {
  await writeFile(resolve(artifacts, 'actions.log'), actions);
  await writeFile(resolve(artifacts, 'vite.log'), serverError);
  await run('close').catch(() => {});
  server.kill('SIGTERM');
  console.log(`Artifacts: ${artifacts}`);
}
