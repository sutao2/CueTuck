import { mount, flushPromises, enableAutoUnmount } from '@vue/test-utils';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import Launcher from './LauncherApp.vue';
import { createLocalPrompt, resetMemoryLibrary, setLocalSetting, recordLocalPromptUse } from './platform/library.js';
import { toggleLocalFavorite } from './platform/localFavorites.js';
enableAutoUnmount(afterEach);
beforeEach(()=>resetMemoryLibrary());
afterEach(()=>{vi.restoreAllMocks();delete navigator.clipboard;});
const titles = w => w.findAll('.result-copy .row-title').map(n=>n.text());
it('ranks before limiting and Enter copies the exact title hit',async()=>{
 const exact=await createLocalPrompt({title:'SQL',content:'准确的正文'});
 for(let i=0;i<30;i++)await createLocalPrompt({title:`其他 ${i}`,content:'SQL'});
 await setLocalSetting('launcher_preferences',JSON.stringify({resultLimit:10}));
 const writeText=vi.fn().mockResolvedValue();Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText}});
 const w=mount(Launcher);await flushPromises();await w.get('input').setValue('SQL');await flushPromises();
 expect(titles(w)[0]).toBe('SQL');expect(titles(w)).toHaveLength(10);
 await w.get('input').trigger('keydown',{key:'Enter'});await flushPromises();
 expect(writeText).toHaveBeenCalledWith('准确的正文');expect(exact.use_count).toBe(1);
});
it('reloads favorite and usage preferences on the next local query without networking',async()=>{
 const a=await createLocalPrompt({title:'测试 A',content:'A'}),b=await createLocalPrompt({title:'测试 B',content:'B'});
 const fetcher=vi.spyOn(globalThis,'fetch').mockRejectedValue(Error('must stay local'));
 const w=mount(Launcher);await flushPromises();await w.get('input').setValue('测试');await flushPromises();
 expect(titles(w)).toEqual(['测试 A','测试 B']);
 await toggleLocalFavorite(b.id);await w.get('input').setValue('');await w.get('input').setValue('测试');await flushPromises();
 expect(titles(w)).toEqual(['测试 B','测试 A']);
 await recordLocalPromptUse(a.id);await w.get('input').setValue('');await w.get('input').setValue('测试');await flushPromises();
 expect(titles(w)).toEqual(['测试 A','测试 B']);expect(fetcher).not.toHaveBeenCalled();
});
