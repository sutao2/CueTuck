// Capture current application components in an isolated, signed-out browser.
import {execFile, spawn} from 'node:child_process';
import {promisify, stripVTControlCharacters} from 'node:util';
import {mkdir, readFile, writeFile} from 'node:fs/promises';
import {resolve, dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';

const root=resolve(dirname(fileURLToPath(import.meta.url)), '..');
const output=resolve(root,'docs/assets/readme');
const artifacts=resolve(root,'output/playwright',`readme-${Date.now()}`);
await mkdir(output,{recursive:true});await mkdir(artifacts,{recursive:true});
const cli=resolve(root,'desktop/node_modules/@playwright/cli/playwright-cli.js');
const session=`cuetuck-readme-${process.pid}`, origin='http://127.0.0.1:1445';
const exec=promisify(execFile);
let snapshot='', log='';
const server=spawn(process.execPath,['--input-type=module','-e',`import {createServer} from ${JSON.stringify(resolve(root,'desktop/node_modules/vite/dist/node/index.js'))}; const server=await createServer({server:{host:'127.0.0.1',port:1445,strictPort:true,proxy:{'/v1':{target:'https://prompt.likh.cn',changeOrigin:true}}}}); await server.listen(); server.printUrls();`],{cwd:resolve(root,'desktop'),env:{...process.env,VITE_API_BASE:origin},stdio:['ignore','pipe','pipe']});
server.stdout.on('data',chunk=>log+=chunk);server.stderr.on('data',chunk=>log+=chunk);
async function run(...args){
  const {stdout}=await exec(process.execPath,[cli,`-s=${session}`,...args],{cwd:artifacts,timeout:60000,maxBuffer:2*1024*1024});
  if(stdout.includes('### Error'))throw Error(stdout);
  const path=stdout.match(/\[Snapshot\]\(([^)]+)\)/)?.[1];
  if(path)snapshot=await readFile(resolve(artifacts,path),'utf8');
  const inline=stdout.match(/```yaml\n([\s\S]*?)```/)?.[1];
  if(inline)snapshot=inline;
  return stdout;
}
async function target(pattern){
  for(let i=0;i<8;i++){
    await run('snapshot');
    const line=snapshot.split('\n').find(line=>pattern.test(line)&&!line.includes('[disabled]'));
    if(line)return line.match(/\[ref=([a-z0-9]+)\]/)[1];
  }
  throw Error(`Missing ${pattern}\n${snapshot}`);
}
const click=async pattern=>run('click',await target(pattern));
const fill=async(pattern,value)=>run('fill',await target(pattern),value);
async function capture(name){
  await run('run-code',`async page=>{
    await page.evaluate(()=>document.fonts.ready);
    if(await page.locator('body').innerText().then(text=>/stacktao@|sk-[a-z0-9]{20}/i.test(text)))throw Error('Unexpected personal content');
    await page.screenshot({path:${JSON.stringify(resolve(output,`${name}.png`))}});
  }`);
}
const samples=[
  {title:'把想法变成产品方案',content:'你是一名产品经理。请围绕 {{产品想法}}，为 {{目标用户}} 制定一份可执行的产品方案。\n\n请依次给出：\n1. 用户问题与使用场景\n2. 最小可行产品的核心功能\n3. 用户操作流程\n4. 验收指标与下一步行动\n\n区分已知事实和待验证的假设。',model:'ChatGPT'},
  {title:'代码审查与改进建议',content:'审阅以下 {{编程语言}} 代码。重点检查正确性、边界条件、可读性与测试覆盖。\n\n按影响程度列出问题，说明触发条件，并给出最小修改建议。不要为了风格重写整段代码。',model:'Claude'},
  {title:'把会议记录整理成行动清单',content:'将 {{会议记录}} 整理成简洁的会议纪要。\n\n包括：讨论主题、已确认决定、待解决问题、负责人和截止时间。没有明确的信息请标为待确认。',model:'通用'},
  {title:'中英文写作助手',content:'面向 {{读者}} 改写下文，使表达清晰、自然。保留事实、专有名词和数字。\n\n先给出中文版本，再提供英文版本，并用三句话说明主要修改。',model:'通用'},
  {title:'研究资料摘要',content:'阅读 {{研究资料}}，提取核心论点、证据与局限。\n\n区分作者观点与原始数据；标注需要进一步验证的信息，并提出三个后续研究问题。',model:'ChatGPT'},
  {title:'一周内容创作计划',content:'为 {{品牌}} 制定面向 {{受众}} 的一周内容计划。\n\n每天提供一个主题、内容形式、开头示例和行动引导，避免重复角度和空泛口号。',model:'Claude'},
];
try{
  for(let i=0;i<100&&!stripVTControlCharacters(log).includes('Local:');i++){if(server.exitCode!==null)throw Error(log);await new Promise(r=>setTimeout(r,50));}
  assert.match(stripVTControlCharacters(log),/Local:/);
  await run('open',origin,'--browser','chrome');await run('resize','1440','960');
  await run('run-code',`async page=>{await page.evaluate(async samples=>{
    const library=await import('/src/platform/library.js');
    for(const sample of samples)await library.createLocalPrompt(sample);
  },${JSON.stringify(samples)});}`);
  if(!process.argv.includes('--launcher-only')){
  await click(/tab "提示词广场"/);await target(/共 .* 个结果/);
  await run('run-code',`async page=>{await page.locator('.prompt-card img').first().waitFor({state:'visible'});await page.waitForFunction(()=>[...document.querySelectorAll('.prompt-card img')].slice(0,3).every(image=>image.complete&&image.naturalWidth>0));}`);
  await capture('square');
  await click(/tab "本地提示词"/);await target(/button "把想法变成产品方案"/);await capture('library');
  await click(/button "把想法变成产品方案"/);await click(/button "使用提示词"/);
  await fill(/textbox "产品想法"/,'一个能随时唤起的提示词工作台');
  await click(/button "下一步"/);await fill(/textbox "目标用户"/,'经常使用 AI 的开发者与内容创作者');
  await click(/button "预览"/);await target(/可执行的产品方案/);await capture('variables');
  }
  await run('goto',`${origin}/launcher.html`);await run('resize','760','560');
  await run('run-code',`async page=>{await page.evaluate(async sample=>{await (await import('/src/platform/library.js')).createLocalPrompt(sample);},${JSON.stringify(samples[0])});}`);
  await run('run-code',`async page=>{await page.getByRole('combobox').fill('产品'); await page.getByRole('option').filter({hasText:'把想法变成产品方案'}).waitFor({state:'visible'}); await page.getByRole('combobox').press('Enter');}`);await target(/填写变量/);
  await fill(/textbox "产品想法"/,'一个能随时唤起的提示词工作台');await fill(/textbox "目标用户"/,'经常使用 AI 的开发者与内容创作者');await capture('launcher');
  console.log(`README screenshots captured: ${output}`);
}finally{
  await writeFile(resolve(artifacts,'vite.log'),log);await run('close').catch(()=>{});server.kill('SIGTERM');
}
