// Test the installed NSIS binary on the disposable Windows Actions runner.
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdtemp, readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';
import { setTimeout as delay } from 'node:timers/promises';
const { chromium } = createRequire(new URL('../desktop/package.json', import.meta.url))('playwright');
if (process.platform !== 'win32' || process.env.GITHUB_ACTIONS !== 'true') throw new Error('Requires a disposable Windows Actions runner');
const executable = process.argv[2];
const password = randomUUID();
const profile = await mkdtemp(join(process.env.RUNNER_TEMP, 'cuetuck-webview-'));
let child, exited, browser;
async function start() {
  child = spawn(executable, [], { env: { ...process.env, WEBVIEW2_USER_DATA_FOLDER: profile,
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-address=127.0.0.1 --remote-debugging-port=0' } });
  exited = once(child, 'exit');
  const deadline = Date.now()+60000;
  while (Date.now()<deadline && child.exitCode===null) {
    try {
      const port = Number((await readFile(join(profile,'DevToolsActivePort'),'utf8')).split(/\r?\n/)[0]);
      if (port) browser = await chromium.connectOverCDP('http://127.0.0.1:'+port, {timeout:2000});
    } catch { await delay(250); continue; }
    if (browser) break;
  }
  if (!browser) throw new Error('Installed WebView2 was unavailable');
  while (Date.now()<deadline) {
    const page = browser.contexts().flatMap(c=>c.pages()).find(p=>/^https?:\/\/tauri\.localhost\/(?:index\.html)?$/.test(p.url()));
    if (page) { page.setDefaultTimeout(20000); return page; }
    await delay(200);
  }
  throw new Error('Could not locate the installed main window');
}
async function stop() {
  await browser?.close().catch(()=>{}); browser=null;
  if (child && child.exitCode===null) child.kill();
  if (exited) await exited;
}
try {
  let page = await start();
  const settings = p=>p.locator('.sidebar-bottom button').filter({hasText:'应用锁'});
  await settings(page).click();
  const deviceAvailable = await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('app_lock_device_available'));
  if (deviceAvailable) await page.getByRole('checkbox',{name:/Windows Hello/}).waitFor();
  else if (await page.getByRole('checkbox',{name:/Windows Hello/}).count()) throw new Error('Hello offered on unavailable device');
  const dialog = page.getByRole('dialog');
  await dialog.getByLabel(/新口令（/).fill(password);
  await dialog.getByLabel('再次输入新口令').fill(password);
  await dialog.getByRole('button',{name:'启用应用锁'}).click();
  await page.getByRole('button',{name:'立即锁定'}).click();
  const locked = p=>p.getByRole('heading',{name:'解锁工作台'});
  await locked(page).waitFor();
  if (await settings(page).count()) throw new Error('Private workspace still mounted');
  const unlock = async (p,value)=>{
    await p.getByLabel('解锁口令',{exact:true}).fill(value);
    await p.getByRole('button',{name:'解锁',exact:true}).click();
  };
  await unlock(page,'incorrect-test-value');
  await page.getByRole('alert').filter({hasText:'解锁口令不正确'}).waitFor();
  await locked(page).waitFor();
  await unlock(page,password);
  await settings(page).waitFor();
  await stop();
  page = await start();
  await locked(page).waitFor();
  await unlock(page,password);
  await settings(page).waitFor();
  console.log(JSON.stringify({test:'installed NSIS',enable:true,manualLock:true,rejectWrongPassword:true,
    passwordUnlock:true,restartLocked:true,helloAvailable:deviceAvailable,interactiveHello:'not tested on hosted runner'}));
} finally { await stop(); }
