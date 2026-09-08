// Run through the ignored Rust test: real API/PostgreSQL/MinIO, isolated browser-local library.
import { spawn, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const api=process.env.PROMPTARK_BUSINESS_API, probe=process.env.PROMPTARK_BUSINESS_PROBE;
assert.match(api||'',/^http:\/\/127\.0\.0\.1:\d+$/); assert.ok(probe,'Use the isolated Rust business test');
const artifacts=resolve(root,'output/playwright',`business-${Date.now()}`);await mkdir(artifacts,{recursive:true});
const file=resolve(artifacts,'business-notes.txt'), fileText='Business attachment: verified bytes.\n';await writeFile(file,fileText);
const exec=promisify(execFile),session=`business-${process.pid}`,servers=[];
let snapshot='',actions='',logs='';
async function run(...args){const {stdout}=await exec(process.execPath,[resolve(root,'desktop/node_modules/@playwright/cli/playwright-cli.js'),`-s=${session}`,...args],{cwd:artifacts,timeout:60000,maxBuffer:2097152});actions+=stdout;if(stdout.includes('### Error'))throw Error(stdout);const path=stdout.match(/\[Snapshot\]\(([^)]+)\)/)?.[1];if(path)snapshot=await readFile(resolve(artifacts,path),'utf8');snapshot=stdout.match(/```yaml\n([\s\S]*?)```/)?.[1]||snapshot;return stdout;}
async function target(pattern,last=false){for(let i=0;i<10;i++){await run('snapshot');const lines=snapshot.split('\n').filter(l=>pattern.test(l)&&!l.includes('[disabled]'));const line=last?lines.at(-1):lines[0];if(line)return line.match(/\[ref=(e\d+)\]/)?.[1];}throw Error(`Missing ${pattern}\n${snapshot}`);}
const click=async(pattern,last=false)=>run('click',await target(pattern,last));const fill=async(pattern,value)=>run('fill',await target(pattern),value);
async function server(dir,port){let log='';const child=spawn(process.execPath,[resolve(root,dir,'node_modules/vite/bin/vite.js'),'--host','127.0.0.1','--port',String(port),'--strictPort'],{cwd:resolve(root,dir),env:{...process.env,VITE_API_BASE:api},stdio:['ignore','pipe','pipe']});servers.push(child);child.stdout.on('data',c=>{log+=c;logs+=c});child.stderr.on('data',c=>{log+=c;logs+=c});for(let i=0;i<100&&!log.includes('Local:');i++){if(child.exitCode!==null)throw Error(log);await new Promise(r=>setTimeout(r,50));}assert.match(log,/Local:/);}
try {
  await server('desktop',1433);await server('admin-web',1434);
  await run('open','http://127.0.0.1:1433','--browser','chrome');
  // Only native-only entry points are bridged; no HTTP responses are mocked.
  await run('run-code',`async page => { await page.evaluate(async api => {
    const call=async(path,body)=>{const response=await fetch(api+path,{method:body?'POST':'GET',headers:{'content-type':'application/json'},...(body?{body:JSON.stringify(body)}:{})});if(!response.ok)throw Error('HTTP '+response.status);return response.json()};
    const session=await import('/src/platform/session.js');session.setSessionTransport(body=>call('/v1/session',body));
    (await import('/src/platform/identity.js')).setIdentityTransport((action,body)=>call('/v1/session/identity/'+action,action==='options'?null:body));
    Object.defineProperty(navigator,'clipboard',{value:{writeText:async text=>{window.businessClipboard=text}},configurable:true});
  },${JSON.stringify(api)}); }`);
  await click(/button ".*登录"/);await click(/button "创建账号"/);await fill(/textbox "邮箱"/,'reader@business.test');await click(/button "发送验证邮件"/);await target(/邮件验证码/);
  const response=await fetch(api+'/__test/code',{headers:{'x-business-test':probe}});assert.equal(response.status,200);const {code}=await response.json();
  await fill(/textbox "邮件验证码"/,code);await fill(/textbox "新密码"/,'Business-reader-password');await fill(/textbox "确认新密码"/,'Business-reader-password');await click(/button "验证并设置密码"/);await target(/邮箱验证完成/);
  await fill(/textbox "密码"/,'Business-reader-password');await click(/button "登录"/,true);await target(/reader@business.test/);
  await click(/button "新建"/);await fill(/textbox "标题"/,'Business 验收提示词');await fill(/textbox "提示词内容"/,'欢迎 {{姓名}}，请查看附件。');
  await click(/button "＋ 添加文件"/);await run('upload',file);await target(/查看 business-notes.txt/);await click(/button "保存"/);
  await click(/tab "提示词广场"/);await click(/button "发布提示词"/);
  await run('select',await target(/combobox "本地内容"/),'Business 验收提示词');
  await run('check',await target(/checkbox "business-notes.txt/));await click(/button "提交审核"/);await target(/已提交|提交成功/);
  await run('run-code',`async page=>{await page.evaluate(async api=>{const token=(await import('/src/platform/session.js')).getSession().accessToken;const headers={authorization:'Bearer '+token};if((await fetch(api+'/v1/admin/me',{headers})).status!==403)throw Error('User has admin access');const mine=await (await fetch(api+'/v1/publications/mine',{headers})).json();const p=mine.items.find(p=>p.title==='Business 验收提示词');if(!p||p.status!=='pending'||p.asset_refs.length!==1)throw Error('Missing pending snapshot');if((await fetch(api+'/v1/square/items/'+p.id+'/assets/'+p.asset_refs[0].id)).status!==404)throw Error('Pending file publicly exposed');},${JSON.stringify(api)});}`);
  await run('tab-new','http://127.0.0.1:1434');await fill(/textbox "邮箱"/,'owner@business.test');await fill(/textbox "密码"/,'Business-only-password');await click(/button "登录"/);
  await fill(/textbox "搜索投稿"/,'Business 验收提示词');await click(/button "查询"/);await target(/Business 验收提示词/);await click(/button "通过"/);await click(/button "确认审核"/);await target(/审核已保存/);await run('screenshot');
  await run('tab-select','0');await click(/button "刷新"/);await fill(/searchbox "搜索/, 'Business 验收提示词');await target(/Business 验收提示词/);await click(/button "Business 验收提示词"/);await click(/button "下载到本地"/);await target(/已下载|已保存到本地/);await run('screenshot');
  // Assert bytes and duplicate download behavior against the actual client library code.
  await run('run-code',`async page=>{await page.evaluate(async expected=>{const lib=await import('/src/platform/library.js');const rows=await lib.listLocalPrompts();const downloaded=rows.find(r=>r.remote_id);if(!downloaded)throw Error('No downloaded record');await (await import('/src/platform/square.js')).downloadSquareItem(downloaded.remote_id);const files=await (await import('/src/platform/assets.js')).listPromptAssets(downloaded.id);if((await lib.listLocalPrompts()).filter(r=>r.remote_id===downloaded.remote_id).length!==1)throw Error('Duplicate download');if(files.length!==1||atob(files[0].data)!==expected)throw Error('Attachment bytes differ');if(downloaded.content!=='欢迎 {{姓名}}，请查看附件。')throw Error('Content changed');},${JSON.stringify(fileText)});}`);
  await click(/button "返回"/);await click(/tab "本地提示词/);await click(/button "使用"/,true);await fill(/textbox/,'Ada');await click(/button "下一步"/);await target(/欢迎 Ada/);await click(/button "复制并完成"/);assert.match(await run('eval','window.businessClipboard'),/欢迎 Ada，请查看附件。/);
  console.log('Business roundtrip passed: verified registration, login, local attachment, real upload/publication, admin approval, square download, byte equality, deduplication and variable copy.');
} finally {
  await writeFile(resolve(artifacts,'actions.log'),actions);await writeFile(resolve(artifacts,'vite.log'),logs);await run('close').catch(()=>{});for(const child of servers)child.kill('SIGTERM');console.log(`Artifacts: ${artifacts}`);
}
