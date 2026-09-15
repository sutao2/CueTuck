import { mount, flushPromises, enableAutoUnmount } from '@vue/test-utils';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import Workbench from './WorkbenchShell.vue';
import { resetMemoryLibrary } from '../platform/library.js';
import { resetMemorySession } from '../platform/session.js';
import { resetSquare, setSquarePageTransport, setCatalogTransport } from '../platform/square.js';
enableAutoUnmount(afterEach);
beforeEach(()=>{resetMemoryLibrary();resetMemorySession();resetSquare();setCatalogTransport(async()=>({categories:[],models:[]}));});
const button=(w,text)=>w.findAll('button').find(b=>b.text()===text);
async function open(){const w=mount(Workbench);await flushPromises();await w.get('[data-space="square"]').trigger('click');await flushPromises();return w;}
it('reuses the returned seed on continuation and starts a new batch explicitly',async()=>{
 const transport=vi.fn(async ({offset,recommendation})=>({items:[{id:`${recommendation}-${offset}`,title:'模板',kind:'prompt'}],total:100,next_offset:offset?null:48,recommendation:recommendation||'hour-test'}));setSquarePageTransport(transport);
 const w=await open();expect(w.text()).toContain('每小时轮换');
 await w.get('[data-testid="square-load-more"]').trigger('click');await flushPromises();
 expect(transport.mock.calls.at(-1)[0]).toMatchObject({offset:48,recommendation:'hour-test'});
 await button(w,'换一批').trigger('click');await flushPromises();
 const batch=transport.mock.calls.at(-1)[0];expect(batch.offset).toBe(0);expect(batch.recommendation).not.toBe('hour-test');expect(batch.recommendation).toMatch(/^[a-f0-9-]+$/);
 expect(w.findAll('.prompt-card')).toHaveLength(1);
 await w.get('[data-sort="最新"]').trigger('click');await flushPromises();
 expect(transport.mock.calls.at(-1)[0]).toMatchObject({sort:'最新',recommendation:null,offset:0});expect(button(w,'刷新')).toBeTruthy();expect(w.text()).toContain('按上架时间');
});
it('keeps visible results on expiration and restarts from the first page',async()=>{
 const transport=vi.fn(async ({offset,recommendation})=>{if(offset)throw Error('本轮推荐已过期，请点击换一批重新加载。');return {items:[{id:'first',title:'已显示',kind:'prompt'}],total:100,next_offset:48,recommendation:recommendation||'expired'};});setSquarePageTransport(transport);
 const w=await open();await w.get('[data-testid="square-load-more"]').trigger('click');await flushPromises();
 expect(w.text()).toContain('推荐已过期');expect(w.findAll('.prompt-card')).toHaveLength(1);
 await w.get('[data-testid="square-load-more"]').trigger('click');await flushPromises();
 expect(transport.mock.calls.at(-1)[0].offset).toBe(0);expect(transport.mock.calls.at(-1)[0].recommendation).not.toBe('expired');
});
it('does not accept a late continuation from the previous recommendation batch',async()=>{
 let finish;const transport=vi.fn(async ({offset,recommendation})=>offset?new Promise(resolve=>{finish=resolve;}):{items:[{id:recommendation||'first',title:'模板',kind:'prompt'}],total:100,next_offset:48,recommendation:recommendation||'hour-test'});setSquarePageTransport(transport);
 const w=await open();await w.get('[data-testid="square-load-more"]').trigger('click');
 await button(w,'换一批').trigger('click');await flushPromises();
 finish({items:[{id:'stale',title:'迟到内容',kind:'prompt'}],total:100,next_offset:null,recommendation:'hour-test'});await flushPromises();
 expect(w.text()).not.toContain('迟到内容');expect(w.findAll('.prompt-card')).toHaveLength(1);
});

it('can continue when every item on the first snapshot page was removed',async()=>{
 setSquarePageTransport(async ({offset})=>({items:offset?[{id:'surviving',title:'下一页内容',kind:'prompt'}]:[],total:100,next_offset:offset?null:48,recommendation:'removed-page'}));
 const w=await open();await w.get('[data-testid="square-load-more"]').trigger('click');await flushPromises();
 expect(w.text()).toContain('下一页内容');expect(w.findAll('.prompt-card')).toHaveLength(1);
});
it('persists dismissals, preserves exclusions during paging, and restores preferences',async()=>{
 const transport=vi.fn(async ({exclude,offset,recommendation})=>({items:exclude.includes('one')?[{id:'two',title:'第二条',kind:'prompt'}]:[{id:'one',title:'第一条',kind:'prompt'}],total:2,next_offset:offset?null:48,recommendation:recommendation||'feedback'}));setSquarePageTransport(transport);
 const w=await open();await w.get('[data-testid="card-more"]').trigger('click');await button(w,'不感兴趣').trigger('click');await flushPromises();
 expect(transport.mock.calls.at(-1)[0].exclude).toContain('one');expect(w.text()).not.toContain('第一条');
 await w.get('[data-testid="square-load-more"]').trigger('click');await flushPromises();expect(transport.mock.calls.at(-1)[0].exclude).toEqual(['one']);
 await w.get('[data-sort="最新"]').trigger('click');await flushPromises();expect(transport.mock.calls.at(-1)[0].exclude).toEqual([]);
 await w.get('[data-sort="推荐"]').trigger('click');await flushPromises();await button(w,'恢复推荐偏好').trigger('click');await flushPromises();expect(transport.mock.calls.at(-1)[0].exclude).toEqual([]);
});
it('excludes the previous loaded batch when changing recommendations',async()=>{
 const transport=vi.fn(async({exclude,recommendation})=>({items:[{id:exclude.includes('one')?'two':'one',title:'内容',kind:'prompt'}],total:2,next_offset:null,recommendation:recommendation||'batch'}));setSquarePageTransport(transport);
 const w=await open();await button(w,'换一批').trigger('click');await flushPromises();expect(transport.mock.calls.at(-1)[0].exclude).toEqual(['one']);
});
