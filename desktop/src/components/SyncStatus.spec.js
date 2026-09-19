import { mount, flushPromises } from '@vue/test-utils';
import { it, expect } from 'vitest';
import SyncStatus from './SyncStatus.vue';
import { resetMemoryLibrary, setLocalSetting } from '../platform/library.js';
import { saveSyncResult } from '../platform/syncStatus.js';

it('shows only the current account queue and sync result, and clears both on sign out', async () => {
  resetMemoryLibrary();
  await setLocalSetting('sync_queue', JSON.stringify([{ email: 'a@test', kind: 'favorite' }, { email: 'b@test', kind: 'publish' }]));
  await saveSyncResult('a@test', '个人库已同步');
  const w = mount(SyncStatus, { props: { session: { loggedIn: true, email: 'a@test' }, expanded: true } });
  await flushPromises();
  expect(w.text()).toContain('1 项待发送'); expect(w.text()).toContain('个人库已同步');
  await w.setProps({ session: { loggedIn: true, email: 'b@test' } }); await flushPromises();
  expect(w.text()).not.toContain('个人库已同步');
  await saveSyncResult('b@test', '同步未完成：离线'); await flushPromises();
  expect(w.text()).toContain('同步未完成：离线');
  await w.setProps({ session: { loggedIn: false } }); await flushPromises();
  expect(w.find('[data-testid="sync-status"]').exists()).toBe(false);
  w.unmount();
});
