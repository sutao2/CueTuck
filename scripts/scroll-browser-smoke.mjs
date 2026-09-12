// CLI-driven browser smoke: fresh snapshots supply every interaction target.
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
const exec = promisify(execFile);
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const artifacts = resolve(root, 'output/playwright', `scroll-${process.argv[2] || "check"}-${Date.now()}`);
await mkdir(artifacts, { recursive: true });
const cli = resolve(root, 'desktop/node_modules/@playwright/cli/playwright-cli.js');
const session = `promptark-scroll-${process.pid}`;
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
    const content = Array.from({ length: 400 }, (_, i) => '第 ' + i + ' 行：测试滚动时的正文与页面返回。').join('\\n');
    for (let i = 0; i < 48; i++) await library.createLocalPrompt({ title: '滚动测试 ' + i, content });
    const items = Array.from({ length: 480 }, (_, i) => ({ id: 'scroll-' + i, kind: 'prompt', title: '广场滚动 ' + i, content: '滚动摘要', reference: i % 4 === 0 ? { images: ['/src/assets/app-icon.png'] } : null }));
    square.setCatalogTransport(async () => ({ categories: [], models: [] }));
    square.setSquareTransport(async () => items);
  }); }`);
  const reports = [];
  async function profile(label, selector) {
    const report = await run('run-code', `async page => { return await page.evaluate(async ({selector, label}) => {
      const root = document.querySelector(selector); root.scrollTop = 0;
      await new Promise(resolve => setTimeout(resolve, 150));
      let styleReads = 0;
      const original = window.getComputedStyle;
      window.getComputedStyle = function(element, ...rest) { if (element.classList?.contains('prompt-grid')) styleReads++; return original.call(this, element, ...rest); };
      const intervals = []; let last;
      try {
        for (let i = 0; i < 120; i++) {
          const now = await new Promise(requestAnimationFrame);
          if (last !== undefined) intervals.push(now - last); last = now;
          root.scrollTop = Math.min(root.scrollHeight - root.clientHeight, i * 35);
        }
        const sorted = [...intervals].sort((a,b) => a-b);
        return { label, samples: intervals.length, p95: +sorted[Math.floor(sorted.length * .95)].toFixed(2), over25ms: intervals.filter(n => n > 25).length, styleReads, mountedCards: root.querySelectorAll('.prompt-card').length, scrollTop: root.scrollTop, maxScroll: root.scrollHeight - root.clientHeight };
      } finally { window.getComputedStyle = original; }
    }, ${JSON.stringify({label, selector})}); }`);
    reports.push(report);
  }
  await click(/tab "提示词广场"/); await target(/button "广场滚动 0"/);
  await profile('square-grid', '[data-region=content]');
  await click(/button "列表视图"/); await profile('square-list', '[data-region=content]');
  await click(/tab "本地提示词(?: \d+)?"/); await click(/button "网格视图"/);
  await profile('local-grid', '[data-region=content]');
  await click(/button "滚动测试 \d+"/); await profile('local-detail', '[data-testid=local-detail] .create-body');
  await click(/button "编辑"/); await profile('editor-body', '[data-testid=prompt-editor] .create-body');
  await profile('editor-text', '[data-testid=prompt-editor] textarea');
  await run('screenshot');
  await writeFile(resolve(artifacts, 'metrics.txt'), reports.join('\n'));
  console.log(reports.join('\n'));
  console.log('Scroll profiling completed. Timing is diagnostic, not a hardware-independent FPS guarantee.');
} finally {
  await writeFile(resolve(artifacts, 'actions.log'), actions);
  await writeFile(resolve(artifacts, 'vite.log'), serverError);
  await run('close').catch(() => {});
  server.kill('SIGTERM');
  console.log(`Artifacts: ${artifacts}`);
}
