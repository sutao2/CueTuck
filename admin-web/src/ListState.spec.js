import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, expect, it, vi } from 'vitest';
import { clearListStates, listState } from './listState.js';
import { clearAdminSession } from './session.js';
import ReviewManagement from './ReviewManagement.vue';
import { setAdminApiTransport, resetAdminApi } from './adminApi.js';
afterEach(() => { clearListStates(); resetAdminApi(); });
it('restores applied review filters and offset after remount without storing rows or drafts', async () => {
  const transport = vi.fn(() => ({items:[],total:80})); setAdminApiTransport(transport);
  listState('review').save({filters:{q:'needle',author:'',status:'pending',from:'',to:''},offset:25});
  let w=mount(ReviewManagement); await flushPromises();
  expect(transport.mock.calls.at(-1)[0].query).toMatchObject({q:'needle',offset:25});
  w.unmount(); w=mount(ReviewManagement); await flushPromises();
  expect(transport.mock.calls.at(-1)[0].query).toMatchObject({q:'needle',offset:25}); w.unmount();
  expect(Object.keys(listState('review').read()).sort()).toEqual(['filters','offset']);
});
it('logout clears state and old asynchronous writers cannot repopulate it', () => {
  const old=listState('users'); old.save({offset:25}); clearAdminSession(); old.save({offset:50});
  expect(listState('users').read()).toEqual({});
});
