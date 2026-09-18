import { afterEach, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import CreatePromptModal from './CreatePromptModal.vue';

let wrapper;
afterEach(() => { wrapper?.unmount(); vi.unstubAllGlobals(); });
const original = `data:image/png;base64,${btoa('\x89PNG\r\n\x1a\nold')}`;
function editor() {
  wrapper = mount(CreatePromptModal, { props: { prompt: { id:'collection', kind:'collection', title:'已有合集', cover_type:'grid', cover_json:JSON.stringify([original]) } } });
  return wrapper.get('[data-testid="cover-files"]');
}
async function select(input, files) {
  Object.defineProperty(input.element, 'files', { configurable:true, value:files });
  await input.trigger('change');
}
it('normalizes a mislabeled cover before saving and blocks saves while reading', async () => {
  let reader;
  vi.stubGlobal('FileReader', class { constructor() { reader = this; } readAsDataURL() {} });
  const input = editor();
  await select(input, [new File(['fake'], 'photo.png', {type:'image/png'})]);
  await wrapper.get('section').trigger('keydown', {key:'s', metaKey:true});
  expect(wrapper.emitted('save')).toBeUndefined();
  expect(input.element.disabled).toBe(true);
  const data = btoa('\xff\xd8\xffphoto');
  reader.result = `data:image/png;base64,${data}`; reader.onload(); await flushPromises();
  await wrapper.get('section').trigger('keydown', {key:'s', metaKey:true});
  expect(wrapper.emitted('save')[0][0].coverUrls).toEqual([`data:image/jpeg;base64,${data}`]);
});
it('shows unsupported image errors and preserves the previous cover', async () => {
  const input = editor();
  await select(input, [new File(['not an image'], 'broken.png', {type:'image/png'})]);
  await vi.waitFor(() => expect(wrapper.text()).toContain('封面未更改'));
  await wrapper.get('section').trigger('keydown', {key:'s', metaKey:true});
  expect(wrapper.emitted('save')[0][0].coverUrls).toEqual([original]);
});
it('rejects oversized files before reading and retains the existing cover when picker is cancelled', async () => {
  const input = editor(); const read = vi.fn();
  vi.stubGlobal('FileReader', class { readAsDataURL = read; });
  await select(input, [{size:5*1024*1024+1}]);
  expect(wrapper.text()).toContain('单文件上限'); expect(read).not.toHaveBeenCalled();
  await select(input, []);
  await wrapper.get('section').trigger('keydown', {key:'s', metaKey:true});
  expect(wrapper.emitted('save')[0][0].coverUrls).toEqual([original]);
});
