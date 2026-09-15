import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import { resetMemoryLibrary, listLocalPrompts, updateLocalPrompt, deleteLocalPrompt } from './library.js';
import { resetSquare, setDownloadStatsTransport, setSquareContentTransport, downloadSquareItem, completeSquareImages } from './square.js';
import { listPromptAssets } from './assets.js';
import { downloadReferenceImages, clearReferenceImageCache } from './referenceAssets.js';
const url='https://cms-assets.youmind.com/test.png';
const png=Uint8Array.from([137,80,78,71,13,10,26,10]);
const item={id:'image-prompt',title:'配图',content:'原始正文',reference:{images:[url]}};
beforeEach(()=>{clearReferenceImageCache();resetMemoryLibrary();resetSquare(); setDownloadStatsTransport(async () => ({ download_count: 1 }));setSquareContentTransport(async()=>item);});
afterEach(()=>vi.unstubAllGlobals());
it('downloads reference bytes atomically and without credentials or attribution in text',async()=>{
  const fetcher=vi.fn(async()=>new Response(png));vi.stubGlobal('fetch',fetcher);
  await downloadSquareItem(item.id);
  const [row]=await listLocalPrompts();expect(row.content).toBe('原始正文');expect(row.image_count).toBe(1);
  expect((await listPromptAssets(row.id))[0]).toMatchObject({name:'参考图-1.png',mime:'image/png'});
  expect(fetcher.mock.calls[0]).toEqual([url,expect.objectContaining({credentials:'omit',referrerPolicy:'no-referrer',redirect:'error'})]);
  await downloadSquareItem(item.id);expect(fetcher).toHaveBeenCalledTimes(1);
});
it('does not import partial content when a reference fails; retry succeeds',async()=>{
  setSquareContentTransport(async()=>({...item,reference:{images:[url,url+'?second']}}));
  const fetcher=vi.fn().mockResolvedValueOnce(new Response(png)).mockResolvedValueOnce(new Response('<html>error</html>'));
  vi.stubGlobal('fetch',fetcher);await expect(downloadSquareItem(item.id)).rejects.toThrow('参考图 2 下载失败');expect(await listLocalPrompts()).toHaveLength(0);
  fetcher.mockImplementation(async()=>new Response(png));await downloadSquareItem(item.id);expect(await listLocalPrompts()).toHaveLength(1);
});
it('supplements old downloads without overwriting edits, duplicating files or creating another prompt',async()=>{
  setSquareContentTransport(async()=>({...item,reference:null}));await downloadSquareItem(item.id);
  const [row]=await listLocalPrompts();await updateLocalPrompt({...row,title:'本地编辑',content:'不要覆盖'});
  setSquareContentTransport(async()=>item);vi.stubGlobal('fetch',vi.fn(async()=>new Response(png)));
  await completeSquareImages(item.id);await completeSquareImages(item.id);
  expect(await listLocalPrompts()).toEqual([expect.objectContaining({id:row.id,title:'本地编辑',content:'不要覆盖',image_count:1})]);
  expect(await listPromptAssets(row.id)).toHaveLength(1);
});
it('keeps an existing copy intact on failure and rejects a deleted target',async()=>{
  setSquareContentTransport(async()=>({...item,reference:null}));await downloadSquareItem(item.id);const [row]=await listLocalPrompts();
  setSquareContentTransport(async()=>item);vi.stubGlobal('fetch',vi.fn(async()=>new Response('failed',{status:503})));
  await expect(completeSquareImages(item.id)).rejects.toThrow('下载失败');expect(await listPromptAssets(row.id)).toHaveLength(0);
  await deleteLocalPrompt(row.id);await expect(completeSquareImages(item.id)).rejects.toThrow('本地副本不存在');
});
it('deduplicates and bounds allowed URLs, rejects oversized bytes and never loads other hosts',async()=>{
  const fetcher=vi.fn(async()=>new Response(png));vi.stubGlobal('fetch',fetcher);
  await downloadReferenceImages({reference:{images:[url,url,'http://cms-assets.youmind.com/a','https://cms-assets.youmind.com:8443/a','https://private.test/a','https://user:pass@cms-assets.youmind.com/a']}});
  expect(fetcher).toHaveBeenCalledTimes(1);
  clearReferenceImageCache();
  fetcher.mockImplementation(async()=>new Response(new Uint8Array(5*1024*1024+1)));
  await expect(downloadReferenceImages(item)).rejects.toThrow('5 MiB');
});

it('downloads at most three images concurrently, reports completed counts and preserves source order', async () => {
  const urls = Array.from({length:6}, (_, i) => `${url}?image=${i}`), pending = new Map(), progress = [];
  let active = 0, peak = 0;
  const fetcher = vi.fn((address) => new Promise(resolve => {
    active++; peak = Math.max(peak, active);
    pending.set(address, () => { active--; resolve(new Response(Uint8Array.from([...png, urls.indexOf(address)]))); });
  }));
  vi.stubGlobal('fetch', fetcher);
  const task = downloadReferenceImages({reference:{images:urls}}, value => progress.push(value.completed));
  expect(fetcher).toHaveBeenCalledTimes(3);
  for (const index of [2,0,1]) {
    pending.get(urls[index])();
    await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(4 + [2,0,1].indexOf(index)));
  }
  for (const index of [5,4,3]) pending.get(urls[index])();
  const assets = await task;
  expect(peak).toBe(3);
  expect(assets.map(asset => asset.name)).toEqual(urls.map((_, i) => `参考图-${i+1}.png`));
  expect(assets.map(asset => atob(asset.data).charCodeAt(8))).toEqual([0,1,2,3,4,5]);
  expect(progress).toEqual([0,1,2,3,4,5,6]);
});

it('aborts in-flight images and stops queued downloads after a failure without importing or recording stats', async () => {
  const urls = Array.from({length:6}, (_, i) => `${url}?image=${i}`);
  setSquareContentTransport(async () => ({...item,reference:{images:urls}}));
  const stats = vi.fn(); setDownloadStatsTransport(stats);
  let aborted = 0;
  const fetcher = vi.fn((address, {signal}) => address === urls[0] ? Promise.resolve(new Response('broken')) : new Promise((_, reject) => {
    signal.addEventListener('abort', () => { aborted++; reject(Error('aborted')); }, {once:true});
  }));
  vi.stubGlobal('fetch', fetcher);
  await expect(downloadSquareItem(item.id)).rejects.toThrow('参考图 1 下载失败');
  expect(fetcher).toHaveBeenCalledTimes(3);
  expect(aborted).toBe(2);
  expect(await listLocalPrompts()).toHaveLength(0);
  expect(stats).not.toHaveBeenCalled();
});

it('reuses validated successful images on retry, expires the cache and creates fresh asset ids',async()=>{
  vi.useFakeTimers();
  try {
    const second=url+'?retry';
    const fetcher=vi.fn(async(address)=>address===second?new Response('error',{status:503}):new Response(png));vi.stubGlobal('fetch',fetcher);
    await expect(downloadReferenceImages({reference:{images:[url,second]}})).rejects.toThrow();
    fetcher.mockImplementation(async()=>new Response(png));
    const first=await downloadReferenceImages({reference:{images:[url,second]}});
    expect(fetcher.mock.calls.filter(([address])=>address===url)).toHaveLength(1);
    const again=await downloadReferenceImages({reference:{images:[url,second]}});
    expect(again[0].id).not.toBe(first[0].id);expect(fetcher).toHaveBeenCalledTimes(3);
    vi.advanceTimersByTime(300001);await downloadReferenceImages({reference:{images:[url]}});expect(fetcher).toHaveBeenCalledTimes(4);
  } finally { vi.useRealTimers(); }
});
