import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import AttachmentPanel from './AttachmentPanel.vue';
import CreatePromptModal from './CreatePromptModal.vue';
import UsePromptModal from './UsePromptModal.vue';
import * as assetsApi from '../platform/assets.js';
import { createLocalPrompt, resetMemoryLibrary } from '../platform/library.js';
let w;
const textFile = () => ({ id: crypto.randomUUID(), name: 'notes.txt', mime: 'text/plain', data: btoa('<script>not executed</script>') });
beforeEach(resetMemoryLibrary);
afterEach(() => { w?.unmount(); vi.restoreAllMocks(); });

it('previews text as text, removes drafts and does not introduce a modal', async () => {
  const file = textFile(); w = mount(AttachmentPanel, { props: { modelValue: [file] } });
  await w.get('[aria-label="查看 notes.txt"]').trigger('click');
  expect(w.get('pre').text()).toContain('<script>not executed</script>');
  expect(w.find('script').exists()).toBe(false);
  expect(w.find('[aria-modal]').exists()).toBe(false);
  await w.get('[aria-label="移除 notes.txt"]').trigger('click');
  expect(w.emitted('update:modelValue')[0][0]).toEqual([]);
});
it('adds pasted images and rejects a bad batch without emitting partial updates', async () => {
  const read = vi.spyOn(assetsApi, 'readAssetFiles').mockResolvedValue([textFile()]);
  w = mount(AttachmentPanel);
  await w.get('section').trigger('paste', { clipboardData: { files: [new File(['image'], 'image.png', {type:'image/png'})] } }); await flushPromises();
  expect(read).toHaveBeenCalledTimes(1);
  expect(w.emitted('update:modelValue')).toHaveLength(1);
  read.mockRejectedValue(new Error('文件超限'));
  await w.get('section').trigger('drop', { dataTransfer: { files: [new File(['bad'], 'bad.txt')] } }); await flushPromises();
  expect(w.get('[role="alert"]').text()).toContain('文件超限');
  expect(w.emitted('update:modelValue')).toHaveLength(1);
});
it('loads saved attachments and protects attachment-only edits on return', async () => {
  const file = textFile(); const p = await createLocalPrompt({ title: 'Saved', content:'body', assets:[file] });
  w = mount(CreatePromptModal, { props: { prompt:p } }); await flushPromises();
  expect(w.find('[aria-label="查看 notes.txt"]').exists()).toBe(true);
  await w.get('[aria-label="移除 notes.txt"]').trigger('click');
  await w.get('.page-back').trigger('click');
  expect(w.find('[data-testid="discard-editor"]').exists()).toBe(true);
  expect((await assetsApi.listPromptAssets(p.id))).toHaveLength(1);
});
it('blocks save when reading existing attachments fails and permits retry', async () => {
  const read = vi.spyOn(assetsApi, 'listPromptAssets').mockRejectedValue(new Error('read failed'));
  w = mount(CreatePromptModal, { props:{ prompt:{ id:'p',title:'Saved',content:'body',asset_count:1 } } }); await flushPromises();
  expect(w.get('.modal-footer .primary-button').attributes('disabled')).toBeDefined();
  expect(w.get('[role="alert"]').text()).toContain('读取附件失败');
  read.mockResolvedValue([textFile()]);
  await w.get('[role="alert"] button').trigger('click'); await flushPromises();
  expect(w.get('.modal-footer .primary-button').attributes('disabled')).toBeUndefined();
  await w.get('.modal-footer .primary-button').trigger('click');
  expect(w.emitted('save')[0][0].assets).toHaveLength(1);
});
it('leaves ordinary text dragging and pasting to the editor', async () => {
  w = mount(AttachmentPanel);
  const drag = new Event('dragover', { cancelable: true });
  Object.defineProperty(drag, 'dataTransfer', { value: { types: ['text/plain'] } });
  w.element.dispatchEvent(drag);
  expect(drag.defaultPrevented).toBe(false);
  const paste = new Event('paste', { cancelable: true });
  Object.defineProperty(paste, 'clipboardData', { value: { files: [] } });
  w.element.dispatchEvent(paste);
  expect(paste.defaultPrevented).toBe(false);
});
it('loads reference files on demand without including them in copied text', async () => {
  const p = await createLocalPrompt({ title: 'With reference', content: 'Only the prompt body', assets: [textFile()] });
  const read = vi.spyOn(assetsApi, 'listPromptAssets');
  w = mount(UsePromptModal, { props: { prompt: p } });
  expect(read).not.toHaveBeenCalled();
  w.get('details').element.open = true;
  await w.get('details').trigger('toggle'); await flushPromises();
  expect(w.find('[aria-label="查看 notes.txt"]').exists()).toBe(true);
  expect(w.find('.attachment-remove').exists()).toBe(false);
  await w.get('[data-testid="use-next"]').trigger('click');
  expect(w.emitted('copied')[0][0]).toBe('Only the prompt body');
});
