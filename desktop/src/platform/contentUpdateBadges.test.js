import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, expect, it, vi } from 'vitest';
const service = vi.hoisted(() => ({ versions: vi.fn(), setting: vi.fn(), token: null, listener: null }));
vi.mock('./contentVersions.js', () => ({ contentVersions: service.versions }));
vi.mock('./library.js', () => ({ getLocalSetting: service.setting }));
vi.mock('./session.js', () => ({ getSession: () => ({ accessToken: service.token }), subscribeSession: fn => { service.listener = fn; return () => { service.listener = null; }; } }));
import { useContentUpdateBadges, skillUpdateRows } from './contentUpdateBadges.js';
let wrapper, api, rows, loader;
const prompt = { kind:'prompt',remote_id:'remote',local_id:'local',version:1 };
async function start() {
  loader = vi.fn(async () => rows);
  wrapper = mount({ setup() { api = useContentUpdateBadges(loader); return {}; }, template:'<div />' });
  await flushPromises();
}
beforeEach(() => { vi.useFakeTimers(); rows=[{...prompt}]; service.token=null; service.setting.mockResolvedValue(null); service.versions.mockResolvedValue({current_version:2}); });
afterEach(() => { wrapper?.unmount(); vi.useRealTimers(); vi.clearAllMocks(); });
it('compares prompts, collections and individual Skill installs; skips unknown baselines and deduplicates sources', async () => {
  rows.push({...prompt,local_id:'current',version:2}, {...prompt,local_id:'unknown',version:null}, {kind:'collection',remote_id:'album',local_id:'album',version:1},
    ...skillUpdateRows([{root_id:'a',path:'/a',community:{id:'skill',version:1}}, {root_id:'b',path:'/b',community:{id:'skill',version:2}}, {root_id:'c',path:'/c'}]));
  await start();
  expect(api.badges.value).toEqual({'prompt:local':2,'collection:album':2,'a:/a':2});
  expect(service.versions.mock.calls).toEqual([['prompt','remote'],['collection','album'],['skill','skill']]);
  expect(loader).toHaveBeenCalledTimes(1);
  rows[0].version=2; await api.refresh();
  expect(api.badges.value['prompt:local']).toBeUndefined();
  expect(service.versions).toHaveBeenCalledTimes(3);
});
it('throttles focus checks and discovers a new version after five minutes without mutations', async () => {
  service.versions.mockResolvedValue({current_version:1}); await start();
  service.versions.mockResolvedValue({current_version:2});
  window.dispatchEvent(new Event('focus')); await flushPromises();
  expect(api.badges.value).toEqual({}); expect(service.versions).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(300000); await flushPromises();
  expect(api.badges.value).toEqual({'prompt:local':2});
  expect(service.versions).toHaveBeenLastCalledWith('prompt','remote');
});
it('limits requests to three concurrently', async () => {
  rows=Array.from({length:7},(_,i)=>({...prompt,remote_id:`r${i}`,local_id:`l${i}`}));
  const pending=[]; service.versions.mockImplementation(()=>new Promise(resolve=>pending.push(resolve)));
  await start(); expect(pending).toHaveLength(3);
  pending[0]({current_version:2}); await flushPromises(); expect(pending).toHaveLength(4);
  wrapper.unmount(); pending.slice(1).forEach(resolve=>resolve({current_version:2})); await flushPromises();
  expect(api.badges.value).toEqual({}); expect(pending).toHaveLength(4);
});
it('discards prior-account responses and retries with the current session', async () => {
  let finish; service.versions.mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve;})); await start();
  service.token='new-account'; service.versions.mockRejectedValue(Error('not visible')); service.listener();
  finish({current_version:9}); await flushPromises();
  expect(api.badges.value).toEqual({}); expect(service.versions).toHaveBeenCalledTimes(2);
});
it('clears expired unavailable versions and can retry after failure', async () => {
  await start(); service.versions.mockRejectedValue(Error('offline'));
  await vi.advanceTimersByTimeAsync(300000); await flushPromises(); expect(api.badges.value).toEqual({});
  service.versions.mockResolvedValue({current_version:3}); window.dispatchEvent(new Event('online')); await flushPromises();
  expect(api.badges.value).toEqual({'prompt:local':3});
});
it('respects disabled square access and does not request versions', async () => {
  service.setting.mockResolvedValue('0'); await start(); expect(service.versions).not.toHaveBeenCalled(); expect(loader).not.toHaveBeenCalled();
});
it('does not check while hidden or offline and cleans up listeners', async () => {
  const online=vi.spyOn(navigator,'onLine','get').mockReturnValue(false); await start(); expect(service.versions).not.toHaveBeenCalled(); online.mockRestore();
  const visibility=vi.spyOn(document,'visibilityState','get').mockReturnValue('hidden'); await api.refresh(); expect(service.versions).not.toHaveBeenCalled(); visibility.mockRestore();
  document.dispatchEvent(new Event('visibilitychange')); await flushPromises(); expect(service.versions).toHaveBeenCalledTimes(1);
  wrapper.unmount(); expect(service.listener).toBeNull(); await vi.advanceTimersByTimeAsync(300000); window.dispatchEvent(new Event('focus')); await flushPromises(); expect(service.versions).toHaveBeenCalledTimes(1);
});
