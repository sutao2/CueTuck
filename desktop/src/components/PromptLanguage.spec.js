import { mount, flushPromises, enableAutoUnmount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import PromptLanguage from './PromptLanguage.vue';
import UsePrompt from './UsePromptModal.vue';
import * as service from '../platform/translation.js';
enableAutoUnmount(afterEach);
afterEach(()=>vi.restoreAllMocks());
beforeEach(()=>{vi.spyOn(service,'localVersion').mockResolvedValue(null);vi.spyOn(service,'squareVersions').mockResolvedValue({});});
function button(w,text){return w.findAll('button').find(b=>b.text().includes(text));}
it('generates only on explicit request and reuses the English source',async()=>{
 const generate=vi.spyOn(service,'translateOnce').mockResolvedValue({text:'为 {{name}} 写欢迎词。'});
 const w=mount(PromptLanguage,{props:{text:'Write a welcome message for {{name}}.'}});await flushPromises();expect(generate).not.toHaveBeenCalled();
 await button(w,'English').trigger('click');expect(w.emitted('change').at(-1)).toEqual(['Write a welcome message for {{name}}.']);expect(generate).not.toHaveBeenCalled();
 await button(w,'中文').trigger('click');await button(w,'翻译成中文').trigger('click');await flushPromises();expect(generate).toHaveBeenCalledTimes(1);expect(w.emitted('change').at(-1)).toEqual(['为 {{name}} 写欢迎词。']);
 await button(w,'原文').trigger('click');await button(w,'中文').trigger('click');expect(generate).toHaveBeenCalledTimes(1);
});
it('ignores an old response after source changes',async()=>{
 let resolve;vi.spyOn(service,'translateOnce').mockImplementation(()=>new Promise(r=>resolve=r));
 const w=mount(PromptLanguage,{props:{text:'Write a welcome message.'}});await flushPromises();await button(w,'中文').trigger('click');await button(w,'翻译成中文').trigger('click');
 await w.setProps({text:'New prompt content.'});resolve({text:'旧译文'});await flushPromises();expect(w.emitted('change').at(-1)).toEqual(['New prompt content.']);
});
it('reports failure and preserves the original for retry',async()=>{
 vi.spyOn(service,'translateOnce').mockRejectedValue(Error('模型响应被截断'));
 const w=mount(PromptLanguage,{props:{text:'Write a welcome message.'}});await flushPromises();await button(w,'中文').trigger('click');await button(w,'翻译成中文').trigger('click');await flushPromises();expect(w.get('[role=alert]').text()).toContain('模型响应被截断');expect(w.emitted('change').at(-1)).toEqual(['Write a welcome message.']);
});
it('translates the template without changing filled parameter values',async()=>{
 vi.spyOn(service,'translateOnce').mockResolvedValue({text:'你好，{{name}}！'});
 const w=mount(UsePrompt,{props:{prompt:{id:'test',title:'Welcome',content:'Hello, {{name}}!'}}});await flushPromises();
 await w.get('[data-testid=use-value]').setValue('小明');await w.get('[data-testid=use-next]').trigger('click');
 await button(w,'中文').trigger('click');await button(w,'翻译成中文').trigger('click');await flushPromises();expect(w.get('[data-testid=use-preview]').text()).toBe('你好，小明！');
 await button(w,'原文').trigger('click');expect(w.get('[data-testid=use-preview]').text()).toBe('Hello, 小明!');
 await button(w,'中文').trigger('click');await w.get('[data-testid=use-next]').trigger('click');expect(w.emitted('copied').at(-1)).toEqual(['你好，小明！']);
});
it('reads ready public versions without requesting generation',async()=>{
 const versions=vi.spyOn(service,'squareVersions').mockResolvedValue({zh:{status:'ready',version:{content:'现成中文'}}});
 const w=mount(PromptLanguage,{props:{text:'Original English text.',squareId:'remote-id',defaultLanguage:'zh'}});await flushPromises();expect(w.emitted('change').at(-1)).toEqual(['现成中文']);expect(versions).toHaveBeenCalledWith('remote-id');
});
