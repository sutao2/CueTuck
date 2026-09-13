import { beforeEach, expect, it, vi } from 'vitest';
import { resetUpdates, setUpdateTransport, setInstallTransport, updateState, refreshUpdate, downloadUpdate, installUpdate } from './updates.js';
import { resetMemoryLibrary } from './library.js';
beforeEach(() => { resetUpdates(); resetMemoryLibrary(); });
it('automatically downloads but never installs; coalesces busy checks and keeps ready state', async () => {
 let finish;const check=vi.fn(()=>new Promise(resolve=>{finish=resolve;}));setUpdateTransport(check);
 const request=refreshUpdate();await Promise.resolve();await Promise.resolve();await refreshUpdate();expect(check).toHaveBeenCalledTimes(1);
 finish({available:true,version:'9.0.0',notes:'notes'});await request;
 const download=vi.fn(async()=>({ready:true,size:1000}));setInstallTransport(download);await downloadUpdate('preview');
 expect(updateState).toMatchObject({phase:'ready',downloaded:1000,total:1000});await refreshUpdate();expect(check).toHaveBeenCalledTimes(1);
});
it('retains retryable available state when download or signature verification fails', async () => {
 setUpdateTransport(async()=>({available:true,version:'9.0.0'}));setInstallTransport(async()=>{throw Error('签名验证失败');});
 await refreshUpdate({autoDownload:true});expect(updateState.phase).toBe('available');expect(updateState.error).toContain('签名');
 await installUpdate();expect(updateState.phase).toBe('available');
 setInstallTransport(async()=>({ready:true,size:10}));await downloadUpdate();expect(updateState.phase).toBe('ready');
});
it('distinguishes network failure from current and clears stale version on a successful empty check', async () => {
 setUpdateTransport(async()=>{throw Error('断网');});await refreshUpdate();expect(updateState.phase).toBe('error');
 setUpdateTransport(async()=>({available:false}));await refreshUpdate();expect(updateState).toMatchObject({phase:'current',version:'',error:''});
});
