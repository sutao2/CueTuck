import { mount, flushPromises } from '@vue/test-utils';
import { it, expect } from 'vitest';
import WorkbenchShell from './WorkbenchShell.vue';
import { resetMemoryLibrary, listLocalPrompts } from '../platform/library.js';
import { resetMemorySession } from '../platform/session.js';
import { resetSquare, setCatalogTransport } from '../platform/square.js';

it('opens an editable example without writing a prompt and allows saving the same draft', async () => {
  resetMemoryLibrary(); resetMemorySession(); resetSquare(); setCatalogTransport(async () => ({ categories: [], models: [] }));
  const w = mount(WorkbenchShell); await flushPromises();
  await w.get('[data-testid="try-example"]').trigger('click');
  expect(await listLocalPrompts()).toHaveLength(0);
  expect(w.get('[data-testid="prompt-editor"] textarea').element.value).toContain('{{收件人}}');
  await w.get('[data-testid="toggle-trial"]').trigger('click');
  expect(w.get('[data-testid="prompt-editor"]').text()).toContain('收件人');
  await w.get('[data-testid="prompt-editor"] .modal-footer .primary-button').trigger('click'); await flushPromises();
  expect(await listLocalPrompts()).toHaveLength(1);
  expect(w.get('.card-primary').text()).toBe('填写并复制');
  w.unmount();
});
