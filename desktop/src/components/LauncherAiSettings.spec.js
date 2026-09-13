import {mount,flushPromises,enableAutoUnmount} from '@vue/test-utils';
import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import Settings from './LauncherAiSettings.vue';
import * as ai from '../platform/launcherAi.js';
enableAutoUnmount(afterEach);
beforeEach(()=>{window.__TAURI_INTERNALS__={};});afterEach(()=>{delete window.__TAURI_INTERNALS__;vi.restoreAllMocks();});
it('never displays saved secrets and passes only explicit local configuration',async()=>{
 vi.spyOn(ai,'getLauncherAiConfig').mockResolvedValue({endpoint:'https://example.test/v1',model:'one',has_key:true});
 const save=vi.spyOn(ai,'saveLauncherAiConfig').mockImplementation(async c=>({...c,has_key:true}));
 const w=mount(Settings);await flushPromises();expect(w.get('[data-testid="ai-key"]').element.value).toBe('');
 await w.get('[data-testid="ai-model"]').setValue('two');expect(w.emitted('dirty').at(-1)).toEqual([true]);await w.get('[data-testid="ai-save"]').trigger('click');await flushPromises();
 expect(save).toHaveBeenCalledWith({endpoint:'https://example.test/v1',model:'two',api_key:''});expect(w.emitted('dirty').at(-1)).toEqual([false]);
});
it('reports failed reads without allowing accidental default overwrites',async()=>{
 vi.spyOn(ai,'getLauncherAiConfig').mockRejectedValue(Error('凭据库不可用'));const save=vi.spyOn(ai,'saveLauncherAiConfig');
 const w=mount(Settings);await flushPromises();expect(w.get('fieldset').attributes('disabled')).toBeDefined();expect(w.text()).toContain('凭据库不可用');expect(save).not.toHaveBeenCalled();
});
