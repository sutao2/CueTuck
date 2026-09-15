import test from 'node:test';
import assert from 'node:assert/strict';
import { browsePrompts, categoryCount, safeImage, skillCatalog, skillDescription } from './live-data.mjs';
const json = data => ({ok:true,json:async()=>data});
test('browse carries remote filters and preserves server totals beyond the loaded page', async()=>{
 let url;
 const result=await browsePrompts('',{query:'图像 & 代码',category:'cat-image',model:'GPT',offset:24},{fetcher:async u=>{url=new URL(u,'https://prompt.likh.cn');return json({items:[{id:'real'}],total:22392,next_offset:48});}});
 assert.equal(url.pathname,'/v1/square/browse');assert.equal(url.searchParams.get('q'),'图像 & 代码');assert.equal(url.searchParams.get('limit'),'24');assert.equal(url.searchParams.get('offset'),'24');assert.equal(url.searchParams.get('category_id'),'cat-image');assert.equal(result.total,22392);
});
test('failed and malformed pages do not silently turn into sample content',async()=>{
 await assert.rejects(browsePrompts('',{},{fetcher:async()=>({ok:false,status:500})}),/500/);
 await assert.rejects(browsePrompts('',{offset:24},{fetcher:async()=>json({items:[],total:9,next_offset:24})}),/分页/);
});
test('category parents sum direct and child counts exactly once',()=>{
 assert.equal(categoryCount('image',[{id:'photo',parent_id:'image'}],{image:12,photo:3,office:99}),15);
 assert.equal(categoryCount('image',[],null),null);
});
test('image references only allow the reviewed image host or the public asset endpoint',()=>{
 assert.equal(safeImage({id:'a',reference:{images:['javascript:alert(1)','https://evil.example/x']}}),'');
 assert.equal(safeImage({id:'a/b',preview_asset:{id:'file'}}),'/v1/square/items/a%2Fb/assets/file');
});
test('Skills use the resolved commit and only real SKILL.md files',async()=>{
 const calls=[],sha='a'.repeat(40);
 const data=await skillCatalog('anthropics/skills',{fetcher:async url=>{calls.push(url);return json(calls.length===1?{sha}:{tree:[{type:'blob',path:'skills/pdf/SKILL.md'},{type:'blob',path:'README.md'},{type:'tree',path:'SKILL.md'}],truncated:false});}});
 assert.equal(data.commit,sha);assert.equal(data.entries.length,1);assert.equal(data.entries[0].directory,'skills/pdf');assert(calls[1].includes(`/git/trees/${sha}?recursive=1`));
 await assert.rejects(skillCatalog('owner/repo?token=bad'),/来源/);
});
test('metadata descriptions are treated as text, including multiline YAML',()=>{
 assert.equal(skillDescription('---\nname: test\ndescription: >\n  First line\n  second line\n---\n# body'),'First line second line');
 assert.equal(skillDescription('---\ndescription: "<script>text only</script>"\n---'),'<script>text only</script>');
});
