// CLI-driven browser smoke: fresh snapshots supply every interaction target.
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
const exec = promisify(execFile);
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const artifacts = resolve(root, 'output/playwright', `thumbnail-${Date.now()}`);
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
    const original = HTMLImageElement.prototype.decode; window.thumbnailDecodes = 0;
    HTMLImageElement.prototype.decode = function() { window.thumbnailDecodes++; return original.call(this); };
    const canvas = document.createElement('canvas'); canvas.width = 1600; canvas.height = 800;
    const ctx = canvas.getContext('2d'); ctx.fillStyle = '#167b70'; ctx.fillRect(0,0,1600,800);
    ctx.fillStyle = '#f5f0dc'; ctx.font = '120px sans-serif'; ctx.fillText('Original 1600 x 800',100,400);
    const asset = { id: crypto.randomUUID(), name: 'large.png', mime: 'image/png', data: canvas.toDataURL().split(',')[1] };
    window.expectedContent = '完整正文。'.repeat(800) + '全文末尾标记';
    const library = await import('/src/platform/library.js');
    for (let i=0;i<12;i++) await library.createLocalPrompt({ title: '图片缓存验收 ' + i, content: window.expectedContent, assets: [asset] });
  }); }`);
  await click(/tab "提示词广场"/); await click(/tab "本地提示词(?: \d+)?"/);
  await target(/button "查看 图片缓存验收 \d+ 的图片"/);
  const roundtrip = async () => run('run-code', `async page => { return await page.evaluate(async () => {
    const root = document.querySelector('[data-region=content]');
    root.scrollTop = 700; await new Promise(resolve => setTimeout(resolve, 350));
    root.scrollTop = 0; await new Promise(resolve => setTimeout(resolve, 350));
    return window.thumbnailDecodes;
  }); }`);
  await roundtrip();
  const before = await run('eval', 'window.thumbnailDecodes'); await roundtrip();
  assert.equal(await run('eval', 'window.thumbnailDecodes'), before);
  assert.match(await run('eval', '[...document.querySelectorAll(".local-prompt-cover img")].every(img => img.naturalWidth > 0 && img.naturalWidth <= 480 && img.naturalHeight <= 480)'), /true/);
  assert.match(await run('eval', '[...document.querySelectorAll(".prompt-excerpt")].every(node => Array.from(node.textContent.trim()).length <= 241)'), /true/);
  await run('screenshot');
  await click(/button "查看 图片缓存验收 \d+ 的图片"/); await target(/dialog "图片预览/);
  assert.match(await run('eval', 'document.querySelector(".image-viewer img").naturalWidth'), /1600/);
  await run('screenshot'); await run('press', 'Escape');
  await click(/button "复制"/);
  assert.match(await run('eval', 'window.smokeClipboard === window.expectedContent'), /true/);
  await fill(/searchbox "搜索标题或正文"/, '全文末尾标记'); await target(/button "图片缓存验收 \d+"/);
  await click(/button "图片缓存验收 \d+"/);
  assert.match(await run('eval', 'document.querySelector(".reading-content").textContent === window.expectedContent'), /true/);
  await click(/button "(?:← )?返回"/); await click(/button "列表视图"/);
  await run('resize', '800', '700'); await run('screenshot');
  assert.match(await run('eval', 'document.documentElement.scrollWidth <= innerWidth'), /true/);
  console.log('Thumbnail browser passed: bounded dimensions, cached scroll roundtrip, original image, full-text search/read/copy, narrow layout.');
} finally {
  await writeFile(resolve(artifacts, 'actions.log'), actions);
  await writeFile(resolve(artifacts, 'vite.log'), serverError);
  await run('close').catch(() => {});
  server.kill('SIGTERM');
  console.log(`Artifacts: ${artifacts}`);
}
