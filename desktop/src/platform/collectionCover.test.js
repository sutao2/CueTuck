import { expect, it } from 'vitest';
import { collectionCoverAssets } from '../lib/cover.js';
import { validateAssets } from './assets.js';
import { validateCollectionAssets } from './privateMedia.js';
it('retains all nine covers in order and rejects local paths without reading them', () => {
  const urls = Array.from({length:9}, (_,i) => `data:image/png;base64,${btoa('\x89PNG\r\n\x1a\n'+i)}`);
  const assets = collectionCoverAssets({ cover_type:'grid', cover_json:JSON.stringify(urls) });
  expect(assets).toHaveLength(9);
  expect(assets.map(a => `data:${a.mime};base64,${a.data}`)).toEqual(urls);
  expect(new Set(assets.map(a=>a.id)).size).toBe(9);
  expect(() => collectionCoverAssets({ cover_type:'single', cover_json:'["/private/cover.png"]' })).toThrow('重新选择');
});
it('rejects unknown, duplicated, shared, non-image and invalid layout cover references', () => {
  const file={ id:crypto.randomUUID(),media_id:`media.${crypto.randomUUID()}`,size:4,sha256:'a'.repeat(64),name:'cover.png',mime:'image/png' };
  const members=[{title:'成员',content:'正文'}];
  expect(()=>validateCollectionAssets(members,[file],{layout:'grid',asset_ids:[file.id]})).not.toThrow();
  for(const cover of [{layout:'grid',asset_ids:['missing']},{layout:'grid',asset_ids:[file.id,file.id]},{layout:'bad',asset_ids:[file.id]},{layout:'single',asset_ids:[]}]) {
    expect(()=>validateCollectionAssets(members,[file],cover)).toThrow();
  }
  expect(()=>validateCollectionAssets([{...members[0],asset_ids:[file.id]}],[file],{layout:'grid',asset_ids:[file.id]})).toThrow();
  expect(()=>validateCollectionAssets(members,[{...file,mime:'text/plain'}],{layout:'grid',asset_ids:[file.id]})).toThrow();
});

it('recognizes mislabeled saved covers by bytes and preserves nine mixed images', () => {
  const formats = [ ['image/jpeg', 'jpg', '\xff\xd8\xffphoto'], ['image/webp', 'webp', 'RIFF1234WEBPphoto'], ['image/gif', 'gif', 'GIF89aphoto'], ['image/png', 'png', '\x89PNG\r\n\x1a\nphoto'] ];
  const urls = Array.from({length:9}, (_,i) => `data:image/png;base64,${btoa(formats[i % 4][2])}`);
  const assets = collectionCoverAssets({cover_type:'grid',cover_json:JSON.stringify(urls)});
  expect(() => validateAssets(assets)).not.toThrow();
  expect(assets.map(a => a.data)).toEqual(urls.map(u => u.split(',')[1]));
  assets.forEach((a,i) => expect(a).toMatchObject({mime:formats[i % 4][0],name:`cover-${i+1}.${formats[i % 4][1]}`}));
});
it('rejects non-image bytes and invalid encoding before preparing publication', () => {
  for (const data of [btoa('<svg></svg>'), 'bad===', btoa('not an image')]) {
    expect(() => collectionCoverAssets({cover_type:'single',cover_json:JSON.stringify([`data:image/png;base64,${data}`])})).toThrow();
  }
});
