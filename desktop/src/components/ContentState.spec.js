import {mount} from '@vue/test-utils';
import {it,expect} from 'vitest';
import ContentState from './ContentState.vue';
it('announces errors, loading and actions consistently without interpreting server text',async()=>{
 const w=mount(ContentState,{props:{kind:'error',title:'无法加载',description:'<script>bad</script>'},slots:{default:'<button>重试</button>'}});
 expect(w.attributes('role')).toBe('alert');expect(w.find('script').exists()).toBe(false);expect(w.get('button').text()).toBe('重试');
 await w.setProps({kind:'loading'});expect(w.attributes('aria-busy')).toBe('true');expect(w.get('.state-skeleton').attributes('aria-hidden')).toBe('true');w.unmount();
});
