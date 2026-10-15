//! Download provenance and explicit updates. All replacement writes and backups share one transaction.
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

type Result<T> = std::result::Result<T,String>;
#[derive(Deserialize)]
#[serde(tag="action",rename_all="snake_case")]
pub enum Request {
    List,
    Preview { kind:String, local_id:String },
    Import { kind:String, remote_id:String, version:i64, remote:Value, payload:Value },
    Apply { kind:String, local_id:String, remote_id:String, version:i64, remote:Value, payload:Value, expected_digest:String, replace:bool },
    Backups,
    RestoreCopy { id:String },
}
fn db(e:rusqlite::Error)->String {e.to_string()}
pub(crate) fn initialize(c:&Connection)->Result<()> {
    c.execute_batch("CREATE TABLE IF NOT EXISTS downloaded_content (
      local_id TEXT PRIMARY KEY, kind TEXT NOT NULL, remote_id TEXT NOT NULL, version INTEGER NOT NULL,
      baseline_json TEXT NOT NULL, remote_json TEXT NOT NULL, member_map_json TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS content_update_backups (
      id TEXT PRIMARY KEY, local_id TEXT NOT NULL, kind TEXT NOT NULL, saved_at TEXT NOT NULL, payload_json TEXT NOT NULL);").map_err(db)
}
fn row(c:&Connection,kind:&str,id:&str)->Result<Value> {
    let sql=match kind {
        "prompt"=>"SELECT json_object('id',id,'title',title,'summary',summary,'content',content,'category_id',category_id,'model',model,'collection_id',collection_id,'author',author,'source',source,'remote_id',remote_id,'deleted_at',deleted_at) FROM prompts WHERE id=?1",
        "collection"=>"SELECT json_object('id',id,'title',title,'description',description,'category_id',category_id,'cover_type',cover_type,'cover_json',cover_json,'deleted_at',deleted_at) FROM collections WHERE id=?1",
        _=>return Err("无效的内容类型".into()),
    };
    let raw:String=c.query_row(sql,[id],|r|r.get(0)).map_err(db)?;
    let mut value:Value=serde_json::from_str(&raw).map_err(|e|e.to_string())?;
    if kind=="prompt" {value["assets"]=json!(super::assets::read(c,id)?);}
    Ok(value)
}
fn snapshot(c:&Connection,kind:&str,id:&str)->Result<Value> {
    let root=row(c,kind,id)?;
    let mut members=vec![];
    if kind=="collection" {
        let mut statement=c.prepare("SELECT id FROM prompts WHERE collection_id=?1 AND deleted_at IS NULL ORDER BY id").map_err(db)?;
        let ids=statement.query_map([id],|r|r.get::<_,String>(0)).map_err(db)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db)?;
        for id in ids {members.push(row(c,"prompt",&id)?);}
    }
    Ok(json!({"root":root,"members":members}))
}
fn digest(v:&Value)->String {format!("{:x}",Sha256::digest(v.to_string().as_bytes()))}
fn provenance(c:&Connection,id:&str)->Result<Option<Value>> {
    let raw:Option<String>=c.query_row("SELECT json_object('kind',kind,'remote_id',remote_id,'version',version,'baseline',json(baseline_json),'remote',json(remote_json),'member_map',json(member_map_json)) FROM downloaded_content WHERE local_id=?1",[id],|r|r.get(0)).optional().map_err(db)?;
    raw.map(|s|serde_json::from_str(&s).map_err(|e|e.to_string())).transpose()
}
fn preview(c:&Connection,kind:&str,id:&str)->Result<Value> {
    let local=snapshot(c,kind,id)?;
    let origin=provenance(c,id)?;
    let modified=origin.as_ref().map(|v|v["baseline"]!=local);
    Ok(json!({"local_id":id,"kind":kind,"local_digest":digest(&local),"local":local,"origin":origin,"modified":modified}))
}
fn category(c:&Connection,v:&Value)->Result<Option<String>> {
    let Some(id)=v.as_str() else {return Ok(None)};
    let exists:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM categories WHERE id=?1 AND deleted_at IS NULL)",[id],|r|r.get(0)).map_err(db)?;
    Ok(exists.then(||id.into()))
}
fn write(c:&Connection,kind:&str,id:&str,p:&Value,collection:Option<&str>,remote:&str)->Result<()> {
    super::sync::validate_payload(kind,p)?;
    let title=p["title"].as_str().filter(|s|!s.trim().is_empty()).ok_or("内容缺少标题")?;
    let cat=category(c,&p["category_id"])?;
    let now=super::now_millis();
    if kind=="prompt" {
        let content=p["content"].as_str().ok_or("提示词正文格式错误")?;
        let assets:Vec<super::assets::Asset>=serde_json::from_value(p.get("assets").cloned().unwrap_or(json!([]))).map_err(|_|"附件格式错误")?;
        super::assets::validate(&assets)?;
        c.execute("INSERT INTO prompts(id,title,content,category_id,model,collection_id,source,remote_id,created_at,updated_at,author,summary) VALUES(?1,?2,?3,?4,?5,?6,?9,?7,?8,?8,?10,?11)
          ON CONFLICT(id) DO UPDATE SET title=excluded.title,content=excluded.content,category_id=excluded.category_id,model=excluded.model,updated_at=excluded.updated_at,version=prompts.version+1",
          params![id,title,content,cat,p["model"].as_str(),collection,remote,now,if remote.is_empty(){"local"}else{"downloaded"},p["author"].as_str(),p["summary"].as_str()]).map_err(db)?;
        super::assets::replace(c,id,&assets)?;
    }else {
        let cover_type=p["cover_type"].as_str().unwrap_or("none");
        if !matches!(cover_type,"none"|"single"|"grid") {return Err("封面格式错误".into())}
        let cover=p["cover_json"].as_str().unwrap_or("[]");
        serde_json::from_str::<Vec<String>>(cover).map_err(|_|"封面格式错误")?;
        c.execute("INSERT INTO collections(id,title,description,category_id,cover_type,cover_json,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?7)
          ON CONFLICT(id) DO UPDATE SET title=excluded.title,description=excluded.description,category_id=excluded.category_id,cover_type=excluded.cover_type,cover_json=excluded.cover_json,updated_at=excluded.updated_at",
          params![id,title,p["description"].as_str(),cat,cover_type,cover,now]).map_err(db)?;
    }
    Ok(())
}
fn install(c:&Connection,kind:&str,id:&str,remote_id:&str,version:i64,remote:&Value,payload:&Value,old:Option<&Value>)->Result<Value> {
    if !matches!(kind,"prompt"|"collection") || remote_id.is_empty() || version<1 || !remote.is_object() {return Err("下载版本信息无效".into())}
    let prompts=payload["prompts"].as_array().ok_or("下载内容无效")?;
    let mut map=serde_json::Map::new();
    if kind=="prompt" {
        if prompts.len()!=1 {return Err("提示词下载数量无效".into())}
        write(c,"prompt",id,&prompts[0],None,remote_id)?;
    }else {
        let collections=payload["collections"].as_array().ok_or("合集内容无效")?;
        if collections.len()!=1 || prompts.is_empty() {return Err("合集内容无效".into())}
        write(c,"collection",id,&collections[0],None,remote_id)?;
        for prompt in prompts {
            let member=prompt["remote_member_id"].as_str().filter(|id|!id.is_empty());
            if old.is_some() && member.is_none() {return Err("旧合集成员没有稳定标识，请另存新版".into())}
            if member.is_some_and(|key|map.contains_key(key)) {return Err("合集成员标识重复".into())}
            let existing=member.and_then(|key|old.and_then(|o|o["member_map"][key].as_str()));
            let local_id=existing.map(str::to_string).unwrap_or_else(||uuid::Uuid::new_v4().to_string());
            if let Some(existing)=existing {
                let current=row(c,"prompt",existing)?;
                if current["collection_id"]!=id || !current["deleted_at"].is_null() {return Err("本地成员已移出或删除，请保留本地并另存新版".into())}
            }
            write(c,"prompt",&local_id,prompt,Some(id),remote_id)?;
            if let Some(member)=member {map.insert(member.into(),json!(local_id));}
        }
        if let Some(old)=old {
            for (key,local_id) in old["member_map"].as_object().ok_or("合集来源关联无效")? {
                if map.contains_key(key) {continue}
                let local_id=local_id.as_str().ok_or("成员关联无效")?;
                let current=row(c,"prompt",local_id)?;
                let baseline=old["baseline"]["members"].as_array().and_then(|rows|rows.iter().find(|r|r["id"]==local_id));
                // Removed upstream members survive when changed locally; local-only additions are never touched.
                if baseline==Some(&current) {
                    c.execute("UPDATE prompts SET deleted_at=?1,updated_at=?1 WHERE id=?2",params![super::now_millis(),local_id]).map_err(db)?;
                }
            }
        }
    }
    let baseline=snapshot(c,kind,id)?;
    c.execute("INSERT INTO downloaded_content(local_id,kind,remote_id,version,baseline_json,remote_json,member_map_json) VALUES(?1,?2,?3,?4,?5,?6,?7)
        ON CONFLICT(local_id) DO UPDATE SET version=excluded.version,baseline_json=excluded.baseline_json,remote_json=excluded.remote_json,member_map_json=excluded.member_map_json",
        params![id,kind,remote_id,version,baseline.to_string(),remote.to_string(),Value::Object(map).to_string()]).map_err(db)?;
    Ok(json!({"local_id":id,"kind":kind,"version":version}))
}
pub fn run(dir:&Path,request:Request)->Result<Value> {
    let mut c=Connection::open(dir.join("promptark.sqlite")).map_err(db)?;
    let tx=c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
    let result=match request {
        Request::List=>{
            let mut statement=tx.prepare("SELECT d.local_id,d.kind,d.remote_id,d.version,CASE WHEN d.kind='prompt' THEN p.title ELSE c.title END
              FROM downloaded_content d LEFT JOIN prompts p ON p.id=d.local_id LEFT JOIN collections c ON c.id=d.local_id
              WHERE (d.kind='prompt' AND p.id IS NOT NULL AND p.deleted_at IS NULL) OR (d.kind='collection' AND c.id IS NOT NULL AND c.deleted_at IS NULL)
              UNION ALL SELECT p.id,'prompt',p.remote_id,NULL,p.title FROM prompts p WHERE p.remote_id IS NOT NULL AND p.remote_id<>'' AND p.collection_id IS NULL AND p.deleted_at IS NULL
              AND NOT EXISTS(SELECT 1 FROM downloaded_content d WHERE d.local_id=p.id)
              UNION ALL SELECT c.id,'collection',min(p.remote_id),NULL,c.title FROM collections c JOIN prompts p ON p.collection_id=c.id
              WHERE c.deleted_at IS NULL AND p.deleted_at IS NULL AND NOT EXISTS(SELECT 1 FROM downloaded_content d WHERE d.local_id=c.id)
              GROUP BY c.id HAVING count(DISTINCT p.remote_id)=1 AND count(*)=count(NULLIF(p.remote_id,''))
              ORDER BY 5,1").map_err(db)?;
            let rows=statement.query_map([],|r|Ok(json!({"local_id":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"remote_id":r.get::<_,String>(2)?,"version":r.get::<_,Option<i64>>(3)?,"title":r.get::<_,String>(4)?}))).map_err(db)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db)?;
            json!(rows)
        },
        Request::Preview{kind,local_id}=>preview(&tx,&kind,&local_id)?,
        Request::Import{kind,remote_id,version,remote,payload}=>install(&tx,&kind,&uuid::Uuid::new_v4().to_string(),&remote_id,version,&remote,&payload,None)?,
        Request::Apply{kind,local_id,remote_id,version,remote,payload,expected_digest,replace}=>{
            let current=snapshot(&tx,&kind,&local_id)?;
            if digest(&current)!=expected_digest {return Err("预览后本地内容已变化，请重新检查更新".into())}
            let origin=provenance(&tx,&local_id)?;
            if replace {
                let old=origin.as_ref().ok_or("旧下载没有版本基准，请另存新版")?;
                if old["remote_id"]!=remote_id || old["kind"]!=kind || version<=old["version"].as_i64().unwrap_or(i64::MAX) {return Err("版本或来源已变化，请重新检查".into())}
                if !current["root"]["deleted_at"].is_null() {return Err("内容已删除，请另存新版".into())}
                if kind=="collection" && (old["member_map"].as_object().is_none_or(|m|m.is_empty()) || old["remote"]["members"].as_array().is_none_or(|members|members.iter().any(|m|m["member_id"].as_str().is_none_or(str::is_empty)))) {return Err("旧合集成员关联未知，请另存新版".into())}
                tx.execute("INSERT INTO content_update_backups(id,local_id,kind,saved_at,payload_json) VALUES(?1,?2,?3,?4,?5)",params![uuid::Uuid::new_v4().to_string(),local_id,kind,super::now_millis(),current.to_string()]).map_err(db)?;
                install(&tx,&kind,&local_id,&remote_id,version,&remote,&payload,Some(old))?
            }else {install(&tx,&kind,&uuid::Uuid::new_v4().to_string(),&remote_id,version,&remote,&payload,None)?}
        },
        Request::Backups=>{
            let mut statement=tx.prepare("SELECT id,kind,saved_at,json_extract(payload_json,'$.root.title') FROM content_update_backups ORDER BY CAST(saved_at AS INTEGER) DESC LIMIT 100").map_err(db)?;
            let rows=statement.query_map([],|r|Ok(json!({"id":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"saved_at":r.get::<_,String>(2)?,"title":r.get::<_,String>(3)?}))).map_err(db)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db)?;json!(rows)
        },
        Request::RestoreCopy{id}=>{
            let (kind,raw):(String,String)=tx.query_row("SELECT kind,payload_json FROM content_update_backups WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(db)?;
            let data:Value=serde_json::from_str(&raw).map_err(|e|e.to_string())?;
            let local_id=uuid::Uuid::new_v4().to_string();
            write(&tx,&kind,&local_id,&data["root"],None,"")?;
            if kind=="collection" {for member in data["members"].as_array().ok_or("备份格式无效")? {write(&tx,"prompt",&uuid::Uuid::new_v4().to_string(),member,Some(&local_id),"")?;}}
            json!({"local_id":local_id,"kind":kind})
        },
    };
    tx.commit().map_err(db)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_database as library;
    fn payload(text:&str)->Value {json!({"prompts":[{"title":"Title","content":text,"assets":[]}]})}
    fn download(dir:&Path)->String {
        run(dir,Request::Import{kind:"prompt".into(),remote_id:"remote".into(),version:1,remote:json!({"content":"v1"}),payload:payload("v1")}).unwrap()["local_id"].as_str().unwrap().into()
    }
    fn apply(id:&str,digest:&str,text:&str,replace:bool)->Request {
        Request::Apply{kind:"prompt".into(),local_id:id.into(),remote_id:"remote".into(),version:2,remote:json!({"content":text}),payload:payload(text),expected_digest:digest.into(),replace}
    }
    #[test]
    fn previews_detect_edits_and_atomic_backups_restore_without_overwriting() {
        let dir=tempfile::tempdir().unwrap();library::initialize_in_dir(dir.path()).unwrap();let id=download(dir.path());
        let c=Connection::open(dir.path().join("promptark.sqlite")).unwrap();
        let before=preview(&c,"prompt",&id).unwrap();assert_eq!(before["modified"],false);
        c.execute("UPDATE prompts SET content='my edit' WHERE id=?1",[&id]).unwrap();
        assert!(run(dir.path(),apply(&id,before["local_digest"].as_str().unwrap(),"v2",true)).unwrap_err().contains("已变化"));
        let changed=preview(&c,"prompt",&id).unwrap();assert_eq!(changed["modified"],true);
        run(dir.path(),apply(&id,changed["local_digest"].as_str().unwrap(),"v2",true)).unwrap();
        assert_eq!(row(&c,"prompt",&id).unwrap()["content"],"v2");
        let backups=run(dir.path(),Request::Backups).unwrap();
        let restored=run(dir.path(),Request::RestoreCopy{id:backups[0]["id"].as_str().unwrap().into()}).unwrap();
        assert_eq!(row(&c,"prompt",restored["local_id"].as_str().unwrap()).unwrap()["content"],"my edit");
        assert_eq!(row(&c,"prompt",&id).unwrap()["content"],"v2");
    }
    #[test]
    fn unknown_legacy_copies_cannot_be_replaced_and_failed_write_rolls_back_backup() {
        let dir=tempfile::tempdir().unwrap();library::initialize_in_dir(dir.path()).unwrap();
        let legacy=library::create_prompt_in_dir(dir.path(),"Legacy","private",None).unwrap();
        let c=Connection::open(dir.path().join("promptark.sqlite")).unwrap();
        let before=preview(&c,"prompt",&legacy.id).unwrap();
        assert!(run(dir.path(),apply(&legacy.id,before["local_digest"].as_str().unwrap(),"new",true)).is_err());
        run(dir.path(),apply(&legacy.id,before["local_digest"].as_str().unwrap(),"new",false)).unwrap();
        assert_eq!(row(&c,"prompt",&legacy.id).unwrap()["content"],"private");
        let id=download(dir.path());let before=preview(&c,"prompt",&id).unwrap();
        c.execute_batch("CREATE TRIGGER fail_update BEFORE UPDATE ON prompts BEGIN SELECT RAISE(ABORT,'simulated write failure'); END;").unwrap();
        assert!(run(dir.path(),apply(&id,before["local_digest"].as_str().unwrap(),"new",true)).is_err());
        assert_eq!(row(&c,"prompt",&id).unwrap()["content"],"v1");
        assert_eq!(run(dir.path(),Request::Backups).unwrap(),json!([]));
        assert_eq!(provenance(&c,&id).unwrap().unwrap()["version"],1);
    }
    #[test]
    fn collections_match_stable_ids_and_keep_edited_removed_members() {
        let dir=tempfile::tempdir().unwrap();library::initialize_in_dir(dir.path()).unwrap();
        let file=json!({"collections":[{"title":"Collection"}],"prompts":[{"remote_member_id":"a","title":"Same","content":"a"},{"remote_member_id":"b","title":"Same","content":"b"}]});
        let result=run(dir.path(),Request::Import{kind:"collection".into(),remote_id:"r".into(),version:1,remote:json!({"members":[{"member_id":"a"},{"member_id":"b"}]}),payload:file.clone()}).unwrap();
        let id=result["local_id"].as_str().unwrap();let c=Connection::open(dir.path().join("promptark.sqlite")).unwrap();
        let origin=provenance(&c,id).unwrap().unwrap();let a=origin["member_map"]["a"].as_str().unwrap();let b=origin["member_map"]["b"].as_str().unwrap();
        c.execute("UPDATE prompts SET content='edited b' WHERE id=?1",[b]).unwrap();
        let before=preview(&c,"collection",id).unwrap();
        let mut updated=file;updated["prompts"]=json!([{"remote_member_id":"c","title":"New","content":"c"},{"remote_member_id":"a","title":"Renamed","content":"a2"}]);
        run(dir.path(),Request::Apply{kind:"collection".into(),local_id:id.into(),remote_id:"r".into(),version:2,remote:json!({}),payload:updated,expected_digest:before["local_digest"].as_str().unwrap().into(),replace:true}).unwrap();
        assert_eq!(row(&c,"prompt",a).unwrap()["content"],"a2");
        assert_eq!(row(&c,"prompt",b).unwrap()["content"],"edited b");assert!(row(&c,"prompt",b).unwrap()["deleted_at"].is_null());
        assert_eq!(snapshot(&c,"collection",id).unwrap()["members"].as_array().unwrap().len(),3);
    }
}
