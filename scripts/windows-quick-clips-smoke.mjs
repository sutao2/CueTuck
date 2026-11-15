// Exercise the installed WebView2 windows on an ephemeral Actions runner.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdtemp } from 'node:fs/promises';
import { createServer } from 'node:net';
import { join } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import { chromium } from '../desktop/node_modules/playwright-core/index.mjs';

assert(process.platform === 'win32' && process.env.GITHUB_ACTIONS === 'true');
const profile = await mkdtemp(join(process.env.RUNNER_TEMP, 'clip-webview-'));
// WebView2 does not reliably write Chromium's DevToolsActivePort file.
const reservation = createServer();
reservation.listen(0, '127.0.0.1');
await once(reservation, 'listening');
const { port } = reservation.address();
await new Promise(resolve => reservation.close(resolve));
const endpoint = 'http://127.0.0.1:' + port;
const env = { ...process.env, WEBVIEW2_USER_DATA_FOLDER: profile,
  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-address=127.0.0.1 --remote-debugging-port=' + port };
delete env.TAURI_SIGNING_PRIVATE_KEY;
delete env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD;
const app = spawn(process.argv[2], [], { env, stdio: 'inherit' });
const stopped = once(app, 'exit');
let browser;
try {
  let ready = false;
  for (let i = 0; i < 120 && !ready; i++) {
    assert.equal(app.exitCode, null, 'Candidate closed before its main window appeared');
    ready = await fetch(endpoint + '/json/version', { signal: AbortSignal.timeout(1000) })
      .then(res => res.ok).catch(() => false);
    if (!ready) await sleep(250);
  }
  assert(ready, 'Installed WebView2 did not open the assigned loopback port');
  browser = await chromium.connectOverCDP(endpoint);
  const context = browser.contexts()[0];
  async function appPage(launcher) {
    for (let i = 0; i < 100; i++) {
      const page = context.pages().find(p => /^https?:\/\/tauri\.localhost\//.test(p.url()) && p.url().includes('launcher.html') === launcher);
      if (page) { page.setDefaultTimeout(15000); return page; }
      await sleep(200);
    }
    throw new Error('Missing installed ' + (launcher ? 'launcher' : 'workbench') + ' window');
  }

  const main = await appPage(false);
  await main.getByRole('tab', { name: '常用片段' }).click();
  const rows = main.locator('article.clip-row');
  await rows.getByText('Legacy clip', { exact: true }).waitFor();
  await main.getByRole('button', { name: '新建分类', exact: true }).click();
  await main.getByRole('textbox', { name: '新建分类' }).fill('QA group');
  await main.locator('.group-form').getByRole('button', { name: '保存' }).click();
  await main.getByRole('button', { name: /QA group\s*0/ }).waitFor();
  await main.locator('footer').getByRole('button', { name: /启动器/ }).click();
  const launcher = await appPage(true);
  const url = 'https://example.invalid/clip-qa';
  await launcher.getByRole('combobox').fill(url);
  await launcher.getByRole('option', { name: '创建常用片段' }).click();
  const form = launcher.locator('.clip-editor');
  assert.equal(await form.locator('textarea').inputValue(), url);
  assert(await form.getByRole('radio', { name: '网址' }).isChecked());
  await form.getByRole('textbox', { name: '名称' }).fill('QA site');
  await form.getByRole('button', { name: '片段分类' }).click();
  await launcher.getByRole('option', { name: 'QA group' }).click();
  await form.getByRole('button', { name: '保存' }).click();
  await launcher.getByRole('option', { name: /QA site/ }).waitFor();
  await launcher.getByRole('combobox').press('Escape');
  await rows.getByText('QA site', { exact: true }).waitFor();
  await main.getByRole('button', { name: /QA group\s*1/ }).waitFor();
  const counts = await main.locator('.category-tree .tree-count').evaluateAll(nodes => nodes.map(n => n.getBoundingClientRect().right));
  assert(counts.length >= 3 && Math.max(...counts) - Math.min(...counts) <= 1, 'Sidebar counts misaligned');
  await main.getByRole('searchbox', { name: '搜索常用片段' }).fill('clip-qa');
  await main.getByRole('button', { name: '筛选片段类型' }).click();
  await main.getByRole('option', { name: '文本', exact: true }).click();
  await main.getByText('没有匹配的片段', { exact: true }).waitFor();
  await main.getByRole('button', { name: '筛选片段类型' }).click();
  await main.getByRole('option', { name: '网址', exact: true }).click();
  await rows.getByText('QA site', { exact: true }).waitFor();
  console.log('Installed NSIS passed legacy clip, category, launcher URL capture, live refresh, counts and combined filters.');
} finally {
  await browser?.close();
  if (app.exitCode === null) app.kill();
  await stopped;
}
