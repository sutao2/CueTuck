import { beforeEach, expect, it, vi } from 'vitest';
const { download }=vi.hoisted(()=>({download:vi.fn()}));
vi.mock('./privateMedia.js',()=>({downloadPublishedAsset:download}));
beforeEach(()=>{vi.resetModules();download.mockReset();});
it('coalesces matching downloads, scopes credentials, and retries failure',async()=>{
 const {publishedImage}=await import('./publishedImageCache.js');
 download.mockResolvedValue({data:'abc'});const ref={id:'one'};
 await Promise.all([publishedImage('item',ref,'a'),publishedImage('item',ref,'a')]);
 await publishedImage('item',ref,'a');expect(download).toHaveBeenCalledTimes(1);
 await publishedImage('item',ref,'b');expect(download).toHaveBeenCalledTimes(2);
 download.mockRejectedValueOnce(new Error('offline'));await expect(publishedImage('new',ref,'a')).rejects.toThrow('offline');
 await publishedImage('new',ref,'a');expect(download).toHaveBeenCalledTimes(4);
});
it('limits in-flight downloads to three and evicts oversized cache entries',async()=>{
 const {publishedImage}=await import('./publishedImageCache.js');let active=0,max=0;
 download.mockImplementation(async()=>{active++;max=Math.max(max,active);await new Promise(resolve=>setTimeout(resolve,2));active--;return {data:'x'.repeat(3*1024*1024)};});
 await Promise.all(Array.from({length:6},(_,i)=>publishedImage('item',{id:i},null)));
 expect(max).toBe(3);await publishedImage('item',{id:0},null);expect(download).toHaveBeenCalledTimes(7);
});
