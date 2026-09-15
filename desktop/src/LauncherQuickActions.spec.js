import {mount,flushPromises,enableAutoUnmount} from '@vue/test-utils';
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import Launcher from './LauncherApp.vue';
import * as library from './platform/library.js';
import * as ai from './platform/launcherAi.js';
import {resetSquare,setSquarePageTransport,setSquareContentTransport,setDownloadStatsTransport} from './platform/square.js';
import * as tauri from './platform/tauri.js';
enableAutoUnmount(afterEach);
beforeEach(()=>{library.resetMemoryLibrary();resetSquare();});
afterEach(()=>{vi.restoreAllMocks();vi.useRealTimers();delete navigator.clipboard;delete window.__TAURI_INTERNALS__;});
const action=(w,text)=>w.findAll('button').find(b=>b.text().includes(text));
async function open(){const w=mount(Launcher);await flushPromises();await w.get('input').setValue('为 {{读者}} 写一篇介绍');await flushPromises();return w;}
it('creates once from input through keyboard and saves original variables',async()=>{
 const w=await open();await w.get('input').trigger('keydown',{key:'Enter'});await flushPromises();
 expect(w.get('[data-testid="quick-content"]').element.value).toBe('为 {{读者}} 写一篇介绍');
 await w.get('[data-testid="quick-title"]').setValue('介绍');await action(w,'保存到本地').trigger('click');await flushPromises();
 const rows=await library.listLocalPrompts({query:''});expect(rows).toHaveLength(1);expect(rows[0].content).toContain('{{读者}}');expect(w.text()).toContain('已创建并保存');
});
it('keeps input on AI failure and requires adopting successful result',async()=>{
 vi.spyOn(ai,'getLauncherAiConfig').mockResolvedValue({endpoint:'https://example.test/v1',model:'fixture'});
 const optimize=vi.spyOn(ai,'optimizeLauncherPrompt').mockRejectedValueOnce(Error('模型超时')).mockResolvedValue('为 {{读者}} 写一篇结构清晰的介绍');
 const w=await open();await action(w,'AI 优化').trigger('click');await flushPromises();expect(w.get('input').element.value).toContain('{{读者}}');expect(w.text()).toContain('模型超时');
 await action(w,'AI 优化').trigger('click');await flushPromises();expect(w.get('[data-testid="quick-content"]').element.value).toContain('结构清晰');expect(await library.listLocalPrompts({query:''})).toHaveLength(0);
 expect(optimize).toHaveBeenCalledWith('为 {{读者}} 写一篇介绍');await action(w,'返回输入').trigger('click');await flushPromises();expect(w.get('input').element.value).toBe('为 {{读者}} 写一篇介绍');
});
it('only searches square after an explicit action, ignores stale network results and respects disabled access',async()=>{
 vi.useFakeTimers();let finish;const remote=vi.fn(()=>new Promise(r=>finish=r));setSquarePageTransport(remote);
 const w=await open();expect(remote).not.toHaveBeenCalled();await action(w,'搜索提示词广场').trigger('click');await vi.advanceTimersByTimeAsync(260);expect(remote).toHaveBeenCalledTimes(1);
 await action(w,'返回本地').trigger('click');await flushPromises();finish({items:[{id:'remote',title:'迟到结果'}],total:1,next_offset:null});await flushPromises();expect(w.text()).not.toContain('迟到结果');
 await library.setLocalSetting('square_access','0');await action(w,'搜索提示词广场').trigger('click');await vi.advanceTimersByTimeAsync(260);expect(remote).toHaveBeenCalledTimes(1);expect(w.text()).toContain('广场访问已关闭');
});

it('only offers create, optimize and square search for current input',async()=>{
 const w=await open();
 expect(w.text()).not.toContain('复制当前输入');
 expect(action(w,'创建提示词')).toBeTruthy();expect(action(w,'AI 优化提示词')).toBeTruthy();expect(action(w,'搜索提示词广场')).toBeTruthy();
});

async function remoteResult(content) {
 vi.useFakeTimers();
 setSquarePageTransport(async()=>({items:[{id:'remote',title:'完整模板',kind:content.kind||'prompt',summary:'仅摘要…'}],total:1,next_offset:null}));
 setSquareContentTransport(async()=>content);
 await library.setLocalSetting('close_launcher_after_use','0');
 const w=await open(); await action(w,'搜索提示词广场').trigger('click');await vi.advanceTimersByTimeAsync(260);await flushPromises();
 return w;
}

it('loads a complete ready Chinese remote template and copies variables without local import or download statistics',async()=>{
 const writeText=vi.fn().mockResolvedValue();Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText}});
 const stats=vi.fn();setDownloadStatsTransport(stats);const record=vi.spyOn(library,'recordLocalPromptUse');
 const original='Hello {{name}}';
 const w=await remoteResult({id:'remote',title:'完整模板',content:original,translations:{zh:{status:'ready',version:{source:{content:original},content:'你好 {{name}}'}}}});
 await w.get('input').trigger('keydown',{key:'Enter'});await flushPromises();
 expect(w.findAll('textarea')).toHaveLength(1);expect(action(w,'查看详情')).toBeTruthy();
 await w.get('textarea').setValue('小明');await w.get('textarea').trigger('keydown',{key:'Enter'});await flushPromises();
 expect(writeText).toHaveBeenCalledExactlyOnceWith('你好 小明');
 expect(stats).not.toHaveBeenCalled();expect(record).not.toHaveBeenCalled();expect(await library.listLocalPrompts()).toHaveLength(0);
 expect(await library.getLocalSetting('last_rendered_prompt')).toBe('你好 小明');
});

it('keeps the remote query retryable after a content failure and never copies the summary',async()=>{
 const writeText=vi.fn();Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText}});
 const w=await remoteResult({});const content=vi.fn().mockRejectedValueOnce(Error('断网')).mockResolvedValue({id:'remote',title:'完整模板',content:'完整正文'});setSquareContentTransport(content);
 await w.get('input').trigger('keydown',{key:'Enter'});await flushPromises();
 expect(w.text()).toContain('断网');expect(w.get('input').element.value).toContain('读者');expect(writeText).not.toHaveBeenCalled();
 await w.get('input').trigger('keydown',{key:'Enter'});await flushPromises();
 expect(writeText).toHaveBeenCalledExactlyOnceWith('完整正文');
});

it('opens a collection in the workbench instead of copying an empty collection',async()=>{
 const navigate=vi.spyOn(tauri,'invokeCommand').mockResolvedValue();
 const w=await remoteResult({kind:'collection'});window.__TAURI_INTERNALS__={};const content=vi.fn();setSquareContentTransport(content);
 await w.get('input').trigger('keydown',{key:'Enter'});await flushPromises();
 expect(navigate).toHaveBeenCalledWith('open_launcher_destination',{destination:'square-detail',id:'remote'});
 expect(content).not.toHaveBeenCalled();
});

it('ignores remote content that resolves after the launcher unmounts',async()=>{
 const writeText=vi.fn();Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText}});
 const w=await remoteResult({});let finish;setSquareContentTransport(()=>new Promise(resolve=>{finish=resolve;}));
 await w.get('input').trigger('keydown',{key:'Enter'});w.unmount();finish({id:'remote',content:'迟到内容'});await flushPromises();
 expect(writeText).not.toHaveBeenCalled();
});

it('does not use a late remote response after returning to local search',async()=>{
 const writeText=vi.fn();Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText}});
 const w=await remoteResult({});let finish;setSquareContentTransport(()=>new Promise(resolve=>{finish=resolve;}));
 await w.get('input').trigger('keydown',{key:'Enter'});
 await action(w,'返回本地').trigger('click');await flushPromises();
 finish({id:'remote',content:'迟到正文'});await flushPromises();
 expect(writeText).not.toHaveBeenCalled();expect(w.find('.preview').exists()).toBe(false);
});

it('offers viewing the selected remote result without using or fetching its content',async()=>{
 const w=await remoteResult({});const content=vi.fn();setSquareContentTransport(content);
 const navigate=vi.spyOn(tauri,'invokeCommand').mockResolvedValue();window.__TAURI_INTERNALS__={};
 await action(w,'查看详情').trigger('click');await flushPromises();
 expect(navigate).toHaveBeenCalledWith('open_launcher_destination',{destination:'square-detail',id:'remote'});
 expect(content).not.toHaveBeenCalled();
});
