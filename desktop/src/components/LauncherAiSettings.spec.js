import {mount,flushPromises,enableAutoUnmount} from '@vue/test-utils';
import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import Settings from './LauncherAiSettings.vue';
import * as ai from '../platform/launcherAi.js';
enableAutoUnmount(afterEach);
beforeEach(()=>{window.__TAURI_INTERNALS__={};});afterEach(()=>{delete window.__TAURI_INTERNALS__;vi.restoreAllMocks();});
it('never displays saved secrets and passes only explicit local configuration',async()=>{
 vi.spyOn(ai,'getLauncherAiConfig').mockResolvedValue({endpoint:'https://example.test/v1',model:'one',has_key:true});
 const save=vi.spyOn(ai,'saveLauncherAiConfig').mockImplementation(async c=>({...c,has_key:true}));
 vi.spyOn(ai,'listLauncherAiModels').mockResolvedValue(['one','two','qwen-mt-flash']);
 const w=mount(Settings);await flushPromises();expect(w.get('[data-testid="ai-key"]').element.value).toBe('');
 await w.get('[data-testid="ai-fetch-models"]').trigger('click');await flushPromises();
 await w.get('[data-testid="ai-model"]').trigger('click');await flushPromises();
 document.querySelector('[role=combobox]').value='two';document.querySelector('[role=combobox]').dispatchEvent(new Event('input',{bubbles:true}));await flushPromises();
 document.querySelector('[role=option]').click();await flushPromises();expect(w.emitted('dirty').at(-1)).toEqual([true]);await w.get('[data-testid="ai-save"]').trigger('click');await flushPromises();
 expect(save).toHaveBeenCalledWith({endpoint:'https://example.test/v1',model:'two',translation_model:'',api_key:''});expect(w.emitted('dirty').at(-1)).toEqual([false]);
});
it('reports failed reads without allowing accidental default overwrites',async()=>{
 vi.spyOn(ai,'getLauncherAiConfig').mockRejectedValue(Error('凭据库不可用'));const save=vi.spyOn(ai,'saveLauncherAiConfig');
 const w=mount(Settings);await flushPromises();expect(w.get('fieldset').attributes('disabled')).toBeDefined();expect(w.text()).toContain('凭据库不可用');expect(save).not.toHaveBeenCalled();
});

it('fetches models from a new unsaved endpoint and requires explicit selection',async()=>{
 vi.spyOn(ai,'getLauncherAiConfig').mockResolvedValue({});
 const list=vi.spyOn(ai,'listLauncherAiModels').mockResolvedValue(['qwen-mt-flash']);
 const w=mount(Settings);await flushPromises();
 await w.get('[data-testid="ai-endpoint"]').setValue('https://example.test/v1');
 await w.get('[data-testid="ai-key"]').setValue('fixture');
 await w.get('[data-testid="ai-fetch-models"]').trigger('click');await flushPromises();
 expect(list).toHaveBeenCalledWith({endpoint:'https://example.test/v1',model:'',translation_model:'',api_key:'fixture'});
 expect(w.get('[data-testid="ai-save"]').element.disabled).toBe(true);
 await w.get('[data-testid="ai-translation-model"]').trigger('click');await flushPromises();
 const choice=[...document.querySelectorAll('[role=option]')].find(e=>e.textContent==='qwen-mt-flash');choice.click();await flushPromises();
 expect(w.get('[data-testid="ai-save"]').element.disabled).toBe(false);
 list.mockResolvedValue([]);await w.get('[data-testid="ai-fetch-models"]').trigger('click');await flushPromises();expect(w.text()).toContain('未返回可选模型');
});
