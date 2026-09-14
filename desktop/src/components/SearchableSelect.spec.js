import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, expect, it, vi } from 'vitest';
import Select from './SearchableSelect.vue';
let w;
afterEach(() => { w?.unmount(); document.body.innerHTML=''; vi.restoreAllMocks(); });
it('filters labels, preserves typed null and closes without changing on Escape', async () => {
 w=mount(Select,{props:{modelValue:'x',options:[{value:null,label:'未分类'},{value:'x',label:'软件 / 开发'}]},attachTo:document.body});
 await w.get('button').trigger('click'); const input=document.querySelector('input');
 input.value='未分'; input.dispatchEvent(new Event('input',{bubbles:true})); await flushPromises();
 expect(document.querySelectorAll('[role=option]')).toHaveLength(1);
 document.querySelector('[role=option]').click(); await flushPromises();
 expect(w.emitted('update:modelValue')).toEqual([[null]]);
 await w.get('button').trigger('click'); document.querySelector('input').dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true})); await flushPromises();
 expect(w.emitted('update:modelValue')).toHaveLength(1); expect(document.activeElement).toBe(w.get('button').element);
});
it('supports keyboard selection, IME and empty results with bounded rendering', async () => {
 w=mount(Select,{props:{options:Array.from({length:500},(_,i)=>({value:i,label:`选项 ${i}`}))}});
 await w.get('button').trigger('click'); let input=document.querySelector('input');
 expect(document.querySelectorAll('[role=option]')).toHaveLength(100);
 input.value='不存在';input.dispatchEvent(new Event('input',{bubbles:true}));await flushPromises();expect(document.querySelector('[role=status]').textContent).toContain('没有匹配');
 input.value='选项 499';input.dispatchEvent(new Event('input',{bubbles:true}));await flushPromises();
 input.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',isComposing:true,bubbles:true}));expect(w.emitted('update:modelValue')).toBeUndefined();
 input.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true}));await flushPromises();expect(w.emitted('update:modelValue')).toEqual([[499]]);
});

it('keeps a wide right-aligned popup inside the viewport', async () => {
 w=mount(Select,{props:{options:[{value:'x',label:'模型'}]}});
 vi.spyOn(w.get('button').element,'getBoundingClientRect').mockReturnValue({left:window.innerWidth-420,width:400,top:100,bottom:136});
 await w.get('button').trigger('click');
 let popup=document.querySelector('.select-popup');
 expect(parseFloat(popup.style.left)+parseFloat(popup.style.width)).toBeLessThanOrEqual(window.innerWidth-8);
 // A wide trigger extending past the right edge still has to clamp the whole popup.
 vi.spyOn(w.get('button').element,'getBoundingClientRect').mockReturnValue({left:window.innerWidth-200,width:400,top:100,bottom:136});
 await w.get('button').trigger('click');await w.get('button').trigger('click');
 popup=document.querySelector('.select-popup');
 expect(parseFloat(popup.style.left)+parseFloat(popup.style.width)).toBeLessThanOrEqual(window.innerWidth-8);
});

it('preserves the selected value beyond the first hundred options on Enter', async () => {
 w=mount(Select,{props:{modelValue:499,options:Array.from({length:500},(_,i)=>({value:i,label:`选项 ${i}`}))}});
 await w.get('button').trigger('click');
 expect(document.querySelectorAll('[role=option]')).toHaveLength(100);
 expect(document.querySelector('.highlighted')?.textContent).toBe('选项 499');
 document.querySelector('input').dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true}));await flushPromises();
 expect(w.emitted('update:modelValue')).toEqual([[499]]);
});
