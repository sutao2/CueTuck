import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import { resetMemoryLibrary, listLocalPrompts, updateLocalPrompt, deleteLocalPrompt } from './library.js';
import { resetSquare, setSquareContentTransport, downloadSquareItem, completeSquareImages } from './square.js';
import { listPromptAssets } from './assets.js';
import { downloadReferenceImages } from './referenceAssets.js';
const url='https://cms-assets.youmind.com/test.png';
const png=Uint8Array.from([137,80,78,71,13,10,26,10]);
const item={id:'image-prompt',title:'配图',content:'原始正文',reference:{images:[url]}};
beforeEach(()=>{resetMemoryLibrary();resetSquare();setSquareContentTransport(async()=>item);});
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
  fetcher.mockImplementation(async()=>new Response(new Uint8Array(5*1024*1024+1)));
  await expect(downloadReferenceImages(item)).rejects.toThrow('5 MiB');
});
