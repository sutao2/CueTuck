import { expect, it } from 'vitest';
import { rankLauncherResults } from './launcherRanking.js';
const now = 1_800_000_000_000, day = 86400000;
const ids = rows => rows.map(row => row.id);
it('ranks exact, prefix and title matches above any personal boost', () => {
  const rows = [
    { id:'body', title:'常用', content:'SQL', use_count:1000, last_used_at:String(now) },
    { id:'contains', title:'生成 SQL' }, { id:'prefix', title:'SQL 查询' }, { id:'exact', title:' sql ' },
  ];
  expect(ids(rankLauncherResults(rows, ' SQL ', ['body'], now))).toEqual(['exact','prefix','contains','body']);
  expect(rows[0].id).toBe('body');
});
it('promotes recent, frequent and favorite results and decays old usage', () => {
  const rows = [{id:'plain',title:'测试'}, {id:'old',title:'测试',last_used_at:String(now-40*day)},
    {id:'favorite',title:'测试'}, {id:'frequent',title:'测试',use_count:100},
    {id:'recent',title:'测试',last_used_at:String(now/1000)}];
  expect(ids(rankLauncherResults(rows,'测试',['favorite'],now))).toEqual(['recent','frequent','favorite','old','plain']);
  expect(rankLauncherResults(rows,'测试',['favorite'],now+40*day)[0].id).toBe('frequent');
});
it('keeps deterministic ties and tolerates missing or invalid legacy timestamps', () => {
  const rows = [{id:'b',title:'测试',last_used_at:'bad'}, {id:'a',title:'测试'}, {id:'c',title:'测试',last_used_at:'0'}];
  expect(ids(rankLauncherResults(rows,'测试',[],now))).toEqual(['a','b','c']);
  expect(ids(rankLauncherResults([...rows].reverse(),'测试',[],now))).toEqual(['a','b','c']);
  expect(rankLauncherResults(rows,'  ')).toEqual([]);
});
it('orders ten thousand candidates before result limiting', () => {
  const rows = Array.from({length:10000},(_,i)=>({id:String(i),title:`测试 ${i}`,use_count:i%20}));
  rows.push({id:'exact',title:'测试'});
  const start=performance.now(); const result=rankLauncherResults(rows,'测试',[],now);
  console.info(`10k launcher ranking: ${(performance.now()-start).toFixed(2)}ms`);
  expect(result[0].id).toBe('exact');expect(new Set(ids(result)).size).toBe(10001);
});
