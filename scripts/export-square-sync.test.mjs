import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {randomUUID} from 'node:crypto';
import {syncSql} from './export-square-sync.mjs';

test('real PostgreSQL: literal content, legacy catalog, replay, conflict and rollback', {skip:process.env.PROMPTARK_LOCAL_SYNC_TEST !== '1'}, () => {
  const db='sync_test_'+randomUUID().replaceAll('-','');
  const run=(args,input)=>spawnSync('docker',['exec','-i','backend-db-1',...args],{input,encoding:'utf8'});
  const sql=input=>run(['psql','-X','-qAt','-v','ON_ERROR_STOP=1','-U','pl','-d',db],input);
  assert.equal(run(['createdb','-U','pl',db]).status,0);
  try {
    assert.equal(sql(`CREATE TABLE catalog(kind text,id text,data jsonb,deleted boolean,PRIMARY KEY(kind,id));
      CREATE TABLE catalog_redirects(kind text,source text,target text,PRIMARY KEY(kind,source));
      CREATE TABLE square_items(id text PRIMARY KEY,content text,category_id text);
      CREATE TABLE accounts(email text,password_hash text);
      INSERT INTO accounts VALUES('owner','keep');
      INSERT INTO catalog VALUES('categories','cat-test','{"name":"测试","region":""}',false);`).status,0);
    const content="写作\n'); DROP TABLE accounts; -- \\ {{变量}}";
    const data={catalog:[{kind:'categories',id:'cat-test',data:{name:'测试'},deleted:false}],catalog_redirects:[],square_items:[{id:'one',content,category_id:'cat-test'}]};
    assert.equal(sql(syncSql(data)).status,0);
    assert.equal(sql(syncSql(data)).status,0);
    assert.equal(sql('SELECT count(*) FROM square_items').stdout.trim(),'1');
    assert.equal(JSON.parse(sql('SELECT to_json(content) FROM square_items').stdout),content);
    assert.equal(sql('SELECT password_hash FROM accounts').stdout.trim(),'keep');
    assert.notEqual(sql(syncSql({...data,square_items:[{...data.square_items[0],content:'conflict'}]})).status,0);
    assert.notEqual(sql(syncSql({...data,square_items:[{id:'two',content:'new',category_id:'missing'}]})).status,0);
    assert.equal(sql('SELECT count(*) FROM square_items').stdout.trim(),'1');
  } finally {assert.equal(run(['dropdb','-U','pl',db]).status,0);}
});

test('rejects incomplete data and duplicate identities before producing SQL', () => {
  assert.throws(()=>syncSql({}),/Missing/);
  assert.throws(()=>syncSql({catalog:[],catalog_redirects:[],square_items:[{id:'same'},{id:'same'}]}),/Duplicate/);
});
