import {mount,flushPromises,enableAutoUnmount} from '@vue/test-utils';
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import Launcher from './LauncherApp.vue';
import * as library from './platform/library.js';
import * as ai from './platform/launcherAi.js';
import {resetSquare,setSquarePageTransport} from './platform/square.js';
enableAutoUnmount(afterEach);
beforeEach(()=>{library.resetMemoryLibrary();resetSquare();});
afterEach(()=>{vi.restoreAllMocks();vi.useRealTimers();});
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
