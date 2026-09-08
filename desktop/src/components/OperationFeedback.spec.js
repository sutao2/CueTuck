import {mount,flushPromises} from '@vue/test-utils';
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import CreatePromptModal from './CreatePromptModal.vue';
import * as library from '../platform/library.js';
import {resetMemorySession} from '../platform/session.js';
import {resetSquare,setCatalogTransport,setSquareTransport} from '../platform/square.js';
let w;
beforeEach(()=>{library.resetMemoryLibrary();resetMemorySession();resetSquare();setCatalogTransport(async()=>({categories:[],models:[]}));setSquareTransport(async()=>[])});
afterEach(()=>{w?.unmount();vi.restoreAllMocks();vi.unstubAllGlobals()});
async function shell(){w=mount(WorkbenchShell,{attachTo:document.body});await flushPromises()}
const editor=()=>w.get('[data-testid=prompt-editor]');
const save=async()=>{await editor().get('.modal-footer .primary-button').trigger('click');await flushPromises()};

it.each(['createPrompt','editPrompt','createCollection','editCollection'])('separates durable %s success from failed refresh; retry never repeats a write',async mode=>{
  const collection=mode.includes('Collection'),editing=mode.startsWith('edit');
  if(editing)await (collection?library.createLocalCollection:library.createLocalPrompt)({title:'original',content:'body'});
  await shell();
  if(editing){await w.get('.prompt-card').trigger('click');await flushPromises();if(collection)await w.get('[data-testid=edit-collection]').trigger('click')}
  else {await w.get('.content-actions .primary-button').trigger('click');if(collection)await editor().findAll('.create-type')[1].trigger('click')}
  await editor().get('input').setValue('saved title');
  const method=editing?(collection?'updateLocalCollection':'updateLocalPrompt'):(collection?'createLocalCollection':'createLocalPrompt');
  const write=vi.spyOn(library,method);
  vi.spyOn(library,collection?'listLocalCollections':'listLocalPrompts').mockRejectedValueOnce(Error('读取中断'));
  await save();
  expect(w.find('[data-testid=prompt-editor]').exists()).toBe(false);
  expect(w.get('[data-testid=save-notice]').text()).toContain('已保存，但刷新失败');
  expect(write).toHaveBeenCalledTimes(1);
  if(mode==='createPrompt'){
    await w.get('[data-space=square]').trigger('click');await flushPromises();expect(w.get('[data-testid=retry-operation-refresh]').element.disabled).toBe(true);expect(w.get('[data-testid=retry-operation-refresh]').text()).toBe('请回到本地刷新');
    await w.get('[data-space=local]').trigger('click');await flushPromises();
  }
  await w.get('[data-testid=retry-operation-refresh]').trigger('click');await flushPromises();
  expect(write).toHaveBeenCalledTimes(1);expect(w.get('[data-testid=save-notice]').text()).toContain('已刷新');
  const items=await (collection?library.listLocalCollections:library.listLocalPrompts)();expect(items).toHaveLength(1);expect(items[0].title).toBe('saved title');
});

it('uses one save path for keyboard and click, retains a failed draft and reports eventual success',async()=>{
  await shell();await w.get('.content-actions .primary-button').trigger('click');await editor().get('input').setValue('draft');await editor().get('textarea').setValue('keep {{value}}');
  const original=library.createLocalPrompt;let fail;
  const write=vi.spyOn(library,'createLocalPrompt').mockImplementationOnce(()=>new Promise((_,reject)=>{fail=reject}));
  await editor().get('textarea').trigger('keydown',{key:'s',metaKey:true});await editor().get('.primary-button').trigger('click');
  await editor().trigger('keydown',{key:'s',ctrlKey:true});expect(write).toHaveBeenCalledTimes(1);
  await w.get('[data-space=square]').trigger('click');expect(editor().exists()).toBe(true);
  fail(Error('磁盘不可写'));await flushPromises();expect(editor().text()).toContain('保存失败');expect(editor().get('textarea').element.value).toBe('keep {{value}}');
  write.mockImplementation(original);await editor().trigger('keydown',{key:'s',ctrlKey:true});await flushPromises();
  expect(write).toHaveBeenCalledTimes(2);expect(w.get('[data-testid=save-notice]').text()).toContain('已保存「draft」');expect(await library.listLocalPrompts()).toHaveLength(1);
});

it('ignores composition, repeats, invalid title and discard confirmation for save shortcuts',async()=>{
  w=mount(CreatePromptModal);await flushPromises();
  await w.trigger('keydown',{key:'s',metaKey:true});expect(w.emitted('save')).toBeUndefined();
  await w.get('input').setValue('draft');
  for(const event of [{key:'s'},{key:'s',metaKey:true,isComposing:true},{key:'s',ctrlKey:true,repeat:true}])await w.trigger('keydown',event);
  expect(w.emitted('save')).toBeUndefined();await w.get('.page-back').trigger('click');await w.trigger('keydown',{key:'s',metaKey:true});expect(w.emitted('save')).toBeUndefined();
});

async function usePrompt(){await library.createLocalPrompt({title:'copy test',content:'hello {{name}}'});await shell();await w.get('.prompt-card').trigger('contextmenu');await w.get('[data-action=use]').trigger('click');await w.get('[data-testid=use-value]').setValue('Ada');await w.get('[data-testid=use-next]').trigger('click')}
it.each(['record','refresh'])('keeps copied feedback visible after %s fails and never recopies on refresh',async failure=>{
  const copy=vi.fn().mockResolvedValue();vi.stubGlobal('navigator',{platform:'MacIntel',clipboard:{writeText:copy}});await usePrompt();
  const record=vi.spyOn(library,'recordLocalPromptUse');if(failure==='record')record.mockRejectedValueOnce(Error('记录失败'));else vi.spyOn(library,'listLocalPrompts').mockRejectedValueOnce(Error('刷新失败'));
  await w.get('[data-testid=use-next]').trigger('click');await flushPromises();
  expect(w.find('[data-testid=use-modal]').exists()).toBe(false);expect(w.get('[data-testid=copy-notice]').text()).toContain(failure==='record'?'已复制，但保存使用记录失败':'已复制，但刷新失败');
  expect(copy).toHaveBeenCalledExactlyOnceWith('hello Ada');expect(record).toHaveBeenCalledTimes(1);
  if(failure==='refresh'){await w.get('[data-testid=retry-operation-refresh]').trigger('click');await flushPromises();expect(copy).toHaveBeenCalledTimes(1);expect(record).toHaveBeenCalledTimes(1)}
});
it('blocks previous step while copying and keeps filled values on clipboard failure',async()=>{
  let reject;const copy=vi.fn(()=>new Promise((_,fail)=>{reject=fail}));vi.stubGlobal('navigator',{platform:'MacIntel',clipboard:{writeText:copy}});await usePrompt();const record=vi.spyOn(library,'recordLocalPromptUse');
  await w.get('[data-testid=use-next]').trigger('click');const back=w.get('[data-testid=use-modal] .modal-footer .ghost-button');expect(back.element.disabled).toBe(true);await back.trigger('click');expect(w.find('[data-testid=use-preview]').exists()).toBe(true);
  reject(Error('denied'));await flushPromises();expect(w.get('[data-testid=use-modal]').text()).toContain('复制失败');expect(w.get('[data-testid=use-preview]').text()).toBe('hello Ada');expect(record).not.toHaveBeenCalled();
});
