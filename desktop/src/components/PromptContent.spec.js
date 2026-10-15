import { mount } from '@vue/test-utils';
import { expect, it } from 'vitest';
import PromptContent from './PromptContent.vue';
it('renders Markdown structure and preserves exact source including variables', async () => {
  const text = '# SQL 审查\n\n**约束** {{数据库}}\n\n- 检查索引\n\n| 字段 | 类型 |\n| --- | --- |\n| id | int |\n\n```sql\nSELECT * FROM users;\n```';
  const w = mount(PromptContent, { props: { text } });
  expect(w.get('h1').text()).toBe('SQL 审查');
  expect(w.get('strong').text()).toBe('约束');
  expect(w.get('li').text()).toBe('检查索引');
  expect(w.findAll('td')).toHaveLength(2);
  expect(w.get('pre code').element.textContent).toBe('SELECT * FROM users;\n');
  expect(w.text()).toContain('{{数据库}}');
  await w.findAll('button')[1].trigger('click');
  expect(w.get('.prompt-source').text()).toBe(text);
  await w.setProps({ text: '## 新译文' });
  expect(w.get('.prompt-source').text()).toBe('## 新译文');
  await w.findAll('button')[0].trigger('click');
  expect(w.get('h2').text()).toBe('新译文');
});
it('does not execute HTML, load external images, or navigate unsafe links', () => {
  const w = mount(PromptContent, { props: { text: '<script>alert(1)</script>\n\n<img src="https://evil.test/pixel" onerror="alert(1)">\n\n![远程图](https://evil.test/pixel)\n\n[危险](javascript:alert%281%29) [本机](file:///etc/passwd) [正常](https://example.com)' } });
  expect(w.find('script').exists()).toBe(false);
  expect(w.find('img').exists()).toBe(false);
  expect(w.text()).toContain('远程图');
  expect(w.findAll('a[href]')).toHaveLength(1);
  expect(w.get('a[href]').attributes()).toMatchObject({ href: 'https://example.com', target: '_blank', rel: 'noopener noreferrer' });
});

it('keeps the unrendered Markdown when using a local prompt', async () => {
  const { default: LocalPromptDetail } = await import('./LocalPromptDetail.vue');
  const { flushPromises } = await import('@vue/test-utils');
  const content = '# Review\n\n**Keep** {{target}}\n\n```sql\nSELECT 1;\n```';
  const w = mount(LocalPromptDetail, { props: { prompt: { id: 'md-test', title: 'Markdown', content, asset_count: 0 } } });
  await flushPromises();
  expect(w.get('h1').text()).toBe('Review');
  await w.get('.modal-footer .primary-button').trigger('click');
  expect(w.emitted('use')[0][0].content).toBe(content);
  w.unmount();
});
