// Isolated UI contract regression. The fixture is NOT a production backend.
import {createServer} from 'node:http';
import {spawn,execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {mkdir,readFile,writeFile} from 'node:fs/promises';
import {resolve,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const artifacts=resolve(root,'output/playwright',`admin-${Date.now()}`);await mkdir(artifacts,{recursive:true});
const origin='http://127.0.0.1:1432', session=`admin-smoke-${process.pid}`,exec=promisify(execFile);
const identity={email:'owner@fixture.test',role:'owner',permissions:{users:true,configuration:true,roles:true}};
let site={revision:0,name:'Fixture community',description:'Test only',support_email:'',logo_url:'',publishing_open:true,square_public:true,announcement:'',announcement_start:null,announcement_end:null};
let status='pending',saves=0,queries=[],actions='',snapshot='',serverLog='';
const api=createServer(async(req,res)=>{
  res.setHeader('Access-Control-Allow-Origin',origin);res.setHeader('Access-Control-Allow-Headers','authorization,content-type');res.setHeader('Access-Control-Allow-Methods','GET,POST,PUT,DELETE,OPTIONS');
  const send=(code,value)=>{res.writeHead(code,{'content-type':'application/json'});res.end(JSON.stringify(value));};
  if(req.method==='OPTIONS')return send(200,{});
  const url=new URL(req.url,'http://fixture');let raw='';for await(const chunk of req)raw+=chunk;const body=raw?JSON.parse(raw):{};
  if(url.pathname==='/v1/session/oauth/providers')return send(200,{items:[]});
  if(url.pathname==='/v1/session'&&req.method==='POST')return send(200,{access_token:'fixture-access',...identity});
  if(req.headers.authorization!=='Bearer fixture-access')return send(401,{});
  if(url.pathname==='/v1/session'&&req.method==='DELETE')return send(200,{});
  if(url.pathname==='/v1/admin/me')return send(200,identity);
  if(url.pathname==='/v1/admin/reviews'){queries.push(Object.fromEntries(url.searchParams));return send(200,{items:status==='pending'?[{id:'fixture-publication',source_id:'fixture',title:'Smoke review',content:'Fixture public text',status,kind:'prompt',asset_refs:[],members:[],history:[]}]:[],total:status==='pending'?1:0,limit:25,offset:0});}
  if(url.pathname==='/v1/admin/publications/fixture-publication/reject'){assert.equal(body.reason,'Needs revision');status='rejected';return send(200,{status});}
  if(url.pathname==='/v1/admin/users')return send(200,{items:[],total:0,limit:25,offset:0});
  if(url.pathname==='/v1/admin/site'){if(req.method==='PUT'){if(++saves===1)return send(503,{});site={...body,revision:site.revision+1};}return send(200,site);}
  return send(404,{});
});
await new Promise(done=>api.listen(0,'127.0.0.1',done));
const vite=spawn(process.execPath,[resolve(root,'admin-web/node_modules/vite/bin/vite.js'),'--host','127.0.0.1','--port','1432','--strictPort'],{cwd:resolve(root,'admin-web'),env:{...process.env,VITE_API_BASE:`http://127.0.0.1:${api.address().port}`},stdio:['ignore','pipe','pipe']});
vite.stdout.on('data',c=>serverLog+=c);vite.stderr.on('data',c=>serverLog+=c);
async function run(...args){const {stdout}=await exec(process.execPath,[resolve(root,'desktop/node_modules/@playwright/cli/playwright-cli.js'),`-s=${session}`,...args],{cwd:artifacts,timeout:60000,maxBuffer:2097152});actions+=stdout;if(stdout.includes('### Error'))throw Error(stdout);const path=stdout.match(/\[Snapshot\]\(([^)]+)\)/)?.[1];if(path)snapshot=await readFile(resolve(artifacts,path),'utf8');snapshot=stdout.match(/```yaml\n([\s\S]*?)```/)?.[1]||snapshot;return stdout;}
async function target(pattern){for(let i=0;i<8;i++){await run('snapshot');const line=snapshot.split('\n').find(l=>pattern.test(l)&&!l.includes('[disabled]'));if(line)return line.match(/\[ref=(e\d+)\]/)?.[1];}throw Error(`Missing ${pattern}\n${snapshot}`);}
const click=async pattern=>run('click',await target(pattern));const fill=async(pattern,value)=>run('fill',await target(pattern),value);
try{
  for(let i=0;i<100&&!serverLog.includes('Local:');i++){if(vite.exitCode!==null)throw Error(serverLog);await new Promise(r=>setTimeout(r,50));}assert.match(serverLog,/Local:/);
  await run('open',origin,'--browser','chrome');
  await fill(/textbox "邮箱"/,'owner@fixture.test');await fill(/textbox "密码"/,'Fixture-only-password');await click(/button "登录"/);
  await fill(/textbox "搜索投稿"/,'Smoke');await click(/button "查询"/);await target(/Smoke review/);
  await click(/button ".*用户"/);await target(/搜索用户/);await click(/button ".*内容审核"/);await target(/Smoke review/);assert.equal(queries.at(-1).q,'Smoke');
  await click(/button "驳回"/);await fill(/textbox "驳回原因"/,'Needs revision');await click(/button "确认审核"/);await target(/审核已保存/);assert.equal(status,'rejected');
  await click(/button ".*站点设置"/);await fill(/textbox "站点名称"/,'Saved fixture community');await click(/button "保存设置"/);await target(/alert/);
  assert.equal(site.name,'Fixture community');assert.match(snapshot,/Saved fixture community/);
  await click(/button "保存设置"/);await target(/设置已保存/);assert.equal(site.name,'Saved fixture community');assert.equal(saves,2);
  await run('screenshot');await run('resize','800','700');await run('screenshot');
  await click(/button "退出登录"/);await target(/登录管理台/);
  console.log('Admin smoke passed: isolated login, list-return, rejection, failed-save/retry, logout.');
}finally{await writeFile(resolve(artifacts,'actions.log'),actions);await writeFile(resolve(artifacts,'vite.log'),serverLog);await run('close').catch(()=>{});vite.kill('SIGTERM');api.closeAllConnections();await new Promise(done=>api.close(done));console.log(`Artifacts: ${artifacts}`);}
