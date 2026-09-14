import { mount } from '@vue/test-utils';
import { expect, it } from 'vitest';
import SquareDetail from './SquareDetailModal.vue';
const render = item => mount(SquareDetail, { props: { item: { id: 'item', title: 'Title', kind: 'prompt', content: 'body', ...item } }, global: { stubs: { PromptLanguage: true, ReportPanel: true, PublishedAttachments: true } } });
it('shows publisher public name and bio separately from original author', () => {
 const w=render({publisher:{display_name:'发布昵称',bio:'公开简介'},reference:{author:'原作者'}});
 const identity=w.get('[data-testid="square-publisher"]');
 expect(identity.text()).toContain('发布者');expect(identity.text()).toContain('发布昵称');expect(identity.text()).toContain('公开简介');
 expect(w.get('.reference-credit').text()).toContain('原作者');w.unmount();
});
it('labels imported attribution as source author and hides absent attribution',async()=>{
 const w=render({reference:{author:'Source Author'}});
 expect(w.get('[data-testid="square-publisher"]').text()).toContain('来源作者');
 await w.setProps({item:{id:'other',title:'Other',kind:'prompt'}});
 expect(w.find('[data-testid="square-publisher"]').exists()).toBe(false);w.unmount();
});
it('renders untrusted public profile as text',()=>{
 const w=render({publisher:{display_name:'<img src=x>',bio:'<script>bad()</script>'}});
 expect(w.get('[data-testid="square-publisher"]').find('img').exists()).toBe(false);
 expect(w.get('.publisher-bio').text()).toBe('<script>bad()</script>');w.unmount();
});
