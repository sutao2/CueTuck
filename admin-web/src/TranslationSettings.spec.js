import {mount,flushPromises,enableAutoUnmount} from '@vue/test-utils';
import {afterEach,beforeEach,it,expect,vi} from 'vitest';
import Translation from './TranslationSettings.vue';
import * as api from './adminApi.js';
enableAutoUnmount(afterEach);afterEach(()=>vi.restoreAllMocks());
const initial=()=>({config:{revision:0,endpoint:'https://example.com/v1/chat/completions',model:'qwen-mt-flash',enabled:false,daily_tokens:2000000,has_key:true},counts:{queued:10,ready:2},zh_ready:2,total:12,used_tokens:1000,failures:[]});
beforeEach(()=>{vi.spyOn(api,'getTranslation').mockResolvedValue(initial());});
it('shows persisted progress without starting or enabling jobs',async()=>{const action=vi.spyOn(api,'translationAction');const w=mount(Translation);await flushPromises();expect(w.text()).toContain('后台翻译已暂停');expect(w.get('progress').attributes('value')).toBe('2');expect(action).not.toHaveBeenCalled();expect(w.findAll('input[type=password]').every(n=>n.element.value==='')).toBe(true);});
it('loads the provider catalog before saving and surfaces failures',async()=>{const discover=vi.spyOn(api,'discoverAiModels').mockResolvedValue({models:['qwen-mt-flash','qwen-mt-lite']});const w=mount(Translation);await flushPromises();await w.findAll('button').find(b=>b.text()==='获取模型列表').trigger('click');await flushPromises();expect(discover).toHaveBeenCalledWith({endpoint:'https://example.com/v1/chat/completions',key:'',translation:true});expect(w.text()).toContain('已获取 2 个模型');discover.mockResolvedValue({error:'Key 无权读取目录'});await w.findAll('button').find(b=>b.text()==='获取模型列表').trigger('click');await flushPromises();expect(w.get('[role=alert]').text()).toContain('Key 无权读取目录');});
