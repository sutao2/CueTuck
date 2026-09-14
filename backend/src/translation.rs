use axum::{extract::{Path,State},http::{HeaderMap,StatusCode},Json};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use crate::{AppState,postgres::Pg};
fn db_error(_:sqlx::Error)->StatusCode{StatusCode::INTERNAL_SERVER_ERROR}
#[derive(Clone,Serialize,Deserialize)]
#[serde(default,deny_unknown_fields)]
pub struct Config{pub revision:i64,pub endpoint:String,pub model:String,pub enabled:bool,pub daily_tokens:i64, pub encrypted_key:String}
impl Default for Config{fn default()->Self{Self{revision:0,endpoint:String::new(),model:String::new(),enabled:false,daily_tokens:2_000_000,encrypted_key:String::new()}}}
impl Config{fn view(&self)->Value{json!({"revision":self.revision,"endpoint":self.endpoint,"model":self.model,"enabled":self.enabled,"daily_tokens":self.daily_tokens,"has_key":!self.encrypted_key.is_empty()})}}
impl Pg{
 pub async fn init_translation(&self)->Result<(),sqlx::Error>{
  sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (id BOOLEAN PRIMARY KEY DEFAULT true CHECK(id),data JSONB NOT NULL)",self.t("translation_config"))).execute(&self.pool).await?;
  sqlx::query(&format!("INSERT INTO {}(id,data) VALUES(true,$1) ON CONFLICT DO NOTHING",self.t("translation_config"))).bind(json!(Config::default())).execute(&self.pool).await?;
  sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (item_id TEXT NOT NULL REFERENCES {}(id) ON DELETE CASCADE,target TEXT NOT NULL CHECK(target IN ('zh','en')),status TEXT NOT NULL DEFAULT 'queued',attempts INTEGER NOT NULL DEFAULT 0,claim TEXT,lease_until TIMESTAMPTZ,next_at TIMESTAMPTZ NOT NULL DEFAULT now(),data JSONB,error TEXT,actor TEXT NOT NULL,created_at TIMESTAMPTZ NOT NULL DEFAULT now(),PRIMARY KEY(item_id,target))",self.t("prompt_translations"),self.t("square_items"))).execute(&self.pool).await?;
  sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (day DATE PRIMARY KEY,tokens BIGINT NOT NULL DEFAULT 0)",self.t("translation_usage"))).execute(&self.pool).await?;
  sqlx::query(&format!("CREATE TABLE IF NOT EXISTS {} (actor TEXT NOT NULL,day DATE NOT NULL,requests INTEGER NOT NULL CHECK(requests BETWEEN 0 AND 20),PRIMARY KEY(actor,day))",self.t("translation_personal_usage"))).execute(&self.pool).await?;
  // Invalidate at the source write, not by reading every full body while browsing.
  sqlx::query(&format!("CREATE OR REPLACE FUNCTION {}() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.title IS DISTINCT FROM OLD.title OR NEW.content IS DISTINCT FROM OLD.content OR NEW.members IS DISTINCT FROM OLD.members THEN DELETE FROM {} WHERE item_id=OLD.id; END IF; RETURN NEW; END $$",self.t("invalidate_translation"),self.t("prompt_translations"))).execute(&self.pool).await?;
  sqlx::query(&format!("DROP TRIGGER IF EXISTS invalidate_translation ON {}",self.t("square_items"))).execute(&self.pool).await?;
  sqlx::query(&format!("CREATE TRIGGER invalidate_translation BEFORE UPDATE OF title,content,members ON {} FOR EACH ROW EXECUTE FUNCTION {}()",self.t("square_items"),self.t("invalidate_translation"))).execute(&self.pool).await?;
  Ok(())
 }
 pub async fn has_translation_secret(&self)->Result<bool,StatusCode>{Ok(!self.translation_config().await?.encrypted_key.is_empty())}
 async fn translation_config(&self)->Result<Config,StatusCode>{let v:Value=sqlx::query_scalar(&format!("SELECT data FROM {} WHERE id",self.t("translation_config"))).fetch_one(&self.pool).await.map_err(db_error)?;serde_json::from_value(v).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)}
 pub async fn translation_versions(&self,id:&str)->Result<Value,StatusCode>{
  let rows:Vec<(String,String,Option<Value>,Option<String>)>=sqlx::query_as(&format!("SELECT target,status,data,error FROM {} WHERE item_id=$1",self.t("prompt_translations"))).bind(id).fetch_all(&self.pool).await.map_err(db_error)?;
  Ok(Value::Object(rows.into_iter().map(|(target,status,data,error)|(target,json!({"status":status,"version":data,"error":error}))).collect()))
 }
}
pub async fn get(State(state):State<AppState>,headers:HeaderMap)->Result<Json<Value>,StatusCode>{
 crate::require_configuration_admin(&state,&headers).await?;let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
 let config=pg.translation_config().await?;
 let counts:Vec<(String,i64)>=sqlx::query_as(&format!("SELECT status,count(*) FROM {} GROUP BY status",pg.t("prompt_translations"))).fetch_all(&pg.pool).await.map_err(db_error)?;
 let usage:i64=sqlx::query_scalar(&format!("SELECT COALESCE((SELECT tokens FROM {} WHERE day=CURRENT_DATE),0)",pg.t("translation_usage"))).fetch_one(&pg.pool).await.map_err(db_error)?;
 let zh_ready:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM {} WHERE target='zh' AND status='ready'",pg.t("prompt_translations"))).fetch_one(&pg.pool).await.map_err(db_error)?;
 let total:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM {} WHERE visibility='online'",pg.t("square_items"))).fetch_one(&pg.pool).await.map_err(db_error)?;
 let failures:Vec<Value>=sqlx::query_scalar(&format!("SELECT jsonb_build_object('id',t.item_id,'title',s.title,'target',t.target,'error',t.error) FROM {} t JOIN {} s ON s.id=t.item_id WHERE t.status='failed' ORDER BY t.created_at DESC LIMIT 20",pg.t("prompt_translations"),pg.t("square_items"))).fetch_all(&pg.pool).await.map_err(db_error)?;
 Ok(Json(json!({"config":config.view(),"counts":counts.into_iter().map(|(s,n)|(s,json!(n))).collect::<serde_json::Map<_,_>>(),"used_tokens":usage,"zh_ready":zh_ready,"total":total,"failures":failures})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Save{revision:i64,endpoint:String,model:String,enabled:bool,daily_tokens:i64,#[serde(default)]key:String,current_password:String}
pub async fn save(State(state):State<AppState>,headers:HeaderMap,Json(input):Json<Save>)->Result<Json<Value>,StatusCode>{
 let actor=crate::require_configuration_admin(&state,&headers).await?;state.limit_account_auth(&actor).await?;
 let endpoint=input.endpoint.trim().trim_end_matches('/').to_owned();
 if crate::outbound::validate_url(&endpoint).is_err()||!endpoint.ends_with("/chat/completions")||input.model.trim().is_empty()||input.model.len()>200||input.key.len()>4096||input.current_password.len()>512||!(10_000..=100_000_000).contains(&input.daily_tokens){return Err(StatusCode::BAD_REQUEST)}
 let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;let mut tx=pg.pool.begin().await.map_err(db_error)?;
 crate::admin_ai::owner_lock(pg,&mut tx,&actor,&crate::bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?).await?;
 let hash:Option<String>=sqlx::query_scalar(&format!("SELECT password_hash FROM {} WHERE email=$1",pg.t("accounts"))).bind(&actor).fetch_one(&mut *tx).await.map_err(db_error)?;
 if !hash.is_some_and(|h|crate::verify_password(&input.current_password,&h)){return Err(StatusCode::FORBIDDEN)}
 let old:Value=sqlx::query_scalar(&format!("SELECT data FROM {} WHERE id FOR UPDATE",pg.t("translation_config"))).fetch_one(&mut *tx).await.map_err(db_error)?;
 let old:Config=serde_json::from_value(old).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
 if old.revision!=input.revision{return Err(StatusCode::CONFLICT)}
 let encrypted_key=if !input.key.is_empty(){crate::oauth_admin::seal(&state.oauth_config.key,"translation",&input.key)?}else if endpoint==old.endpoint{old.encrypted_key}else{String::new()};
 if encrypted_key.is_empty(){return Err(StatusCode::BAD_REQUEST)}
 let config=Config{revision:old.revision+1,endpoint,model:input.model.trim().into(),enabled:input.enabled,daily_tokens:input.daily_tokens,encrypted_key};
 sqlx::query(&format!("UPDATE {} SET data=$1 WHERE id",pg.t("translation_config"))).bind(json!(config)).execute(&mut *tx).await.map_err(db_error)?;
 crate::admin_risk::audit(pg,&mut tx,&actor,"translation_config",json!({"revision":config.revision})).await?;
 tx.commit().await.map_err(db_error)?;Ok(Json(config.view()))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Discover{endpoint:String,#[serde(default)]key:String,#[serde(default)]model_id:Option<String>,#[serde(default)]translation:bool}
pub async fn discover(State(state):State<AppState>,headers:HeaderMap,Json(input):Json<Discover>)->Result<Json<Value>,StatusCode>{
 let actor=crate::require_configuration_admin(&state,&headers).await?;state.limit_account_auth(&actor).await?;
 let endpoint=input.endpoint.trim().trim_end_matches('/');if !endpoint.ends_with("/chat/completions")||input.key.len()>4096{return Err(StatusCode::BAD_REQUEST)}
 let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
 let key=if !input.key.is_empty(){input.key}else if input.translation{let old=pg.translation_config().await?;if old.endpoint!=endpoint{return Err(StatusCode::BAD_REQUEST)}crate::oauth_admin::unseal(&state.oauth_config.key,"translation",&old.encrypted_key)?}else{crate::admin_ai::discovery_key(&state,input.model_id.as_deref().unwrap_or(""),endpoint).await?};
 let url=format!("{}/models",endpoint.trim_end_matches("/chat/completions"));
 let result=async{let client=crate::outbound::client(&url,15).await?;let value=crate::outbound::bounded_json(client.get(&url).bearer_auth(key).send().await.map_err(|_|"模型目录连接失败".to_string())?).await?;
 let rows=value["data"].as_array().ok_or("未返回兼容模型目录")?;if rows.len()>2000{return Err("模型目录过大".into())}let mut ids:Vec<String>=rows.iter().filter_map(|v|v["id"].as_str()).filter(|s|!s.trim().is_empty()&&s.len()<=200).map(str::to_owned).collect();ids.sort();ids.dedup();if ids.is_empty(){return Err("当前 Key 未返回可用模型".into())}Ok::<_,String>(ids)}.await;
 Ok(Json(match result{Ok(ids)=>json!({"models":ids}),Err(error)=>json!({"error":error})}))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Action{action:String}
pub async fn action(State(state):State<AppState>,headers:HeaderMap,Json(input):Json<Action>)->Result<Json<Value>,StatusCode>{
 let actor=crate::require_configuration_admin(&state,&headers).await?;state.limit_account_auth(&actor).await?;let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
 let mut tx=pg.pool.begin().await.map_err(db_error)?;crate::admin_ai::owner_lock(pg,&mut tx,&actor,&crate::bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?).await?;
 let count=match input.action.as_str(){
 "queue"=>sqlx::query(&format!("INSERT INTO {}(item_id,target,actor) SELECT id,'zh',$1 FROM {} WHERE visibility='online' ON CONFLICT DO NOTHING",pg.t("prompt_translations"),pg.t("square_items"))).bind(&actor).execute(&mut *tx).await.map_err(db_error)?.rows_affected(),
 "retry"=>sqlx::query(&format!("UPDATE {} SET status='queued',attempts=0,error=NULL,next_at=now() WHERE status='failed'",pg.t("prompt_translations"))).execute(&mut *tx).await.map_err(db_error)?.rows_affected(),
 "pause"|"resume"=>sqlx::query(&format!("UPDATE {} SET data=jsonb_set(jsonb_set(data,'{{enabled}}',$1),'{{revision}}',to_jsonb((data->>'revision')::bigint+1)) WHERE id",pg.t("translation_config"))).bind(json!(input.action=="resume")).execute(&mut *tx).await.map_err(db_error)?.rows_affected(),
 _=>return Err(StatusCode::BAD_REQUEST)};
 crate::admin_risk::audit(pg,&mut tx,&actor,"translation_job",json!({"action":input.action,"affected":count})).await?;tx.commit().await.map_err(db_error)?;
 Ok(Json(json!({"affected":count})))
}
pub async fn versions(State(state):State<AppState>,headers:HeaderMap,Path(id):Path<String>)->Result<Json<Value>,StatusCode>{
 if !state.square_public().await?{crate::require_user(&state,&headers).await?;}
 state.get_item(&id).await?.ok_or(StatusCode::NOT_FOUND)?;
 let result=if let Some(pg)=&state.db{pg.translation_versions(&id).await?}else{json!({})};Ok(Json(result))
}
pub async fn enqueue(State(state):State<AppState>,headers:HeaderMap,Path((id,target)):Path<(String,String)>)->Result<Json<Value>,StatusCode>{
 let actor=crate::require_user(&state,&headers).await?;if target!="zh"&&target!="en"{return Err(StatusCode::BAD_REQUEST)}
 let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;state.get_item(&id).await?.ok_or(StatusCode::NOT_FOUND)?;
 let versions=pg.translation_versions(&id).await?;if versions[&target]["status"]=="ready"{return Ok(Json(versions))}
 state.limit_account_auth(&actor).await?;
 if !pg.translation_config().await?.enabled{return Err(StatusCode::SERVICE_UNAVAILABLE)}
 let mut tx=pg.pool.begin().await.map_err(db_error)?;
 sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))").bind(format!("{}:translation-user:{actor}",pg.schema)).execute(&mut *tx).await.map_err(db_error)?;
 let changed=sqlx::query(&format!("INSERT INTO {}(item_id,target,actor) VALUES($1,$2,$3) ON CONFLICT(item_id,target) DO UPDATE SET status='queued',attempts=0,error=NULL,next_at=now() WHERE {}.status='failed'",pg.t("prompt_translations"),pg.t("prompt_translations"))).bind(&id).bind(&target).bind(&actor).execute(&mut *tx).await.map_err(db_error)?.rows_affected();
 if changed>0 {
  let allowed=sqlx::query(&format!("INSERT INTO {}(actor,day,requests) VALUES($1,CURRENT_DATE,1) ON CONFLICT(actor,day) DO UPDATE SET requests={}.requests+1 WHERE {}.requests<20",pg.t("translation_personal_usage"),pg.t("translation_personal_usage"),pg.t("translation_personal_usage"))).bind(&actor).execute(&mut *tx).await.map_err(db_error)?.rows_affected();
  if allowed==0{return Err(StatusCode::TOO_MANY_REQUESTS)}
 }
 tx.commit().await.map_err(db_error)?;
 Ok(Json(pg.translation_versions(&id).await?))
}
impl AppState{pub fn start_translation_worker(&self){let state=self.clone();tokio::spawn(async move{loop{let _=run_one(&state).await;tokio::time::sleep(std::time::Duration::from_secs(1)).await;}});}}
async fn run_one(state:&AppState)->Result<(),StatusCode>{
 let Some(pg)=&state.db else{return Ok(())};
 // Transaction-scoped lock on a dedicated connection prevents overlapping paid calls across replicas.
 let mut guard=pg.pool.begin().await.map_err(db_error)?;
 let locked:bool=sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(hashtext($1))").bind(format!("{}:translation-worker",pg.schema)).fetch_one(&mut *guard).await.map_err(db_error)?;if !locked{return Ok(())}
 let config=pg.translation_config().await?;if !config.enabled||config.model.is_empty(){return Ok(())}
 sqlx::query(&format!("UPDATE {} SET status='failed',error='上次翻译中断，请重试',claim=NULL WHERE status='running' AND lease_until<now() AND attempts>=3",pg.t("prompt_translations"))).execute(&pg.pool).await.map_err(db_error)?;
 let row:Option<(String,String)>=sqlx::query_as(&format!("SELECT t.item_id,t.target FROM {} t JOIN {} s ON s.id=t.item_id WHERE s.visibility='online' AND ((t.status='queued' AND t.next_at<=now()) OR (t.status='running' AND t.lease_until<now())) AND t.attempts<3 ORDER BY CASE WHEN t.target='en' THEN 0 ELSE 1 END,t.created_at,t.item_id LIMIT 1",pg.t("prompt_translations"),pg.t("square_items"))).fetch_optional(&pg.pool).await.map_err(db_error)?;
 let Some((id,target))=row else{return Ok(())};let Some(item)=state.get_item(&id).await? else{return Ok(())};
 let source=json!({"title":item.title,"content":item.content.unwrap_or_default(),"members":item.members});
 let source_text=serde_json::to_string(&source).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
 let claim=uuid::Uuid::new_v4().to_string();
 // Reserve a conservative ceiling before spending. Failed/aborted calls retain the reservation.
 let reserve=crate::prompt_translation::reservation(&source_text);
 let mut tx=pg.pool.begin().await.map_err(db_error)?;
 let usage_day:String=sqlx::query_scalar("SELECT CURRENT_DATE::text").fetch_one(&mut *tx).await.map_err(db_error)?;
 let budget=sqlx::query(&format!("INSERT INTO {}(day,tokens) SELECT CURRENT_DATE,$1 WHERE $1<=$2 ON CONFLICT(day) DO UPDATE SET tokens={}.tokens+$1 WHERE {}.tokens+$1<=$2",pg.t("translation_usage"),pg.t("translation_usage"),pg.t("translation_usage"))).bind(reserve).bind(config.daily_tokens).execute(&mut *tx).await.map_err(db_error)?.rows_affected();if budget==0{return Ok(())}
 let changed=sqlx::query(&format!("UPDATE {} SET status='running',claim=$3,attempts=attempts+1,lease_until=now()+interval '20 minutes' WHERE item_id=$1 AND target=$2 AND status IN ('queued','running') AND EXISTS(SELECT 1 FROM {} s WHERE s.id=$1 AND s.title=$4 AND COALESCE(s.content,'')=$5 AND COALESCE(s.members,'[]'::jsonb)=$6)",pg.t("prompt_translations"),pg.t("square_items"))).bind(&id).bind(&target).bind(&claim).bind(source["title"].as_str().unwrap_or("")).bind(source["content"].as_str().unwrap_or("")).bind(&source["members"]).execute(&mut *tx).await.map_err(db_error)?.rows_affected();if changed==0{return Ok(())}tx.commit().await.map_err(db_error)?;
 let result=tokio::time::timeout(std::time::Duration::from_secs(900),async{
 let key=crate::oauth_admin::unseal(&state.oauth_config.key,"translation",&config.encrypted_key).map_err(|_|"无法解密翻译密钥".to_string())?;
 let client=crate::outbound::client(&config.endpoint,30).await?;
 let translated=crate::prompt_translation::translate(&client,&config.endpoint,&key,&config.model,&source_text,&target).await?;
 let data:Value=serde_json::from_str(&translated.text).map_err(|_|"译文结构无效".to_string())?;
 Ok::<_,String>((data,translated.input_tokens+translated.output_tokens,translated.usage_complete))
 }).await.unwrap_or_else(|_|Err("翻译超时，原文已保留".into()));
 let mut tx=pg.pool.begin().await.map_err(db_error)?;
 match result{
 Ok((mut data,tokens,usage_complete))=>{data["excerpt"]=json!(data["content"].as_str().unwrap_or("").chars().take(240).collect::<String>());data["source"]=source.clone();data["source_hash"]=json!(crate::prompt_translation::fingerprint(&source_text));data["model"]=json!(config.model);data["rule"]=json!(crate::prompt_translation::RULE);
 sqlx::query(&format!("UPDATE {} SET status='ready',data=$4,error=NULL,claim=NULL WHERE item_id=$1 AND target=$2 AND claim=$3",pg.t("prompt_translations"))).bind(&id).bind(&target).bind(&claim).bind(data).execute(&mut *tx).await.map_err(db_error)?;
 if usage_complete && tokens<reserve as u64{sqlx::query(&format!("UPDATE {} SET tokens=greatest(0,tokens-$1) WHERE day=$2::date",pg.t("translation_usage"))).bind(reserve-tokens as i64).bind(&usage_day).execute(&mut *tx).await.map_err(db_error)?;}
 },Err(error)=>{sqlx::query(&format!("UPDATE {} SET status=CASE WHEN attempts>=3 THEN 'failed' ELSE 'queued' END,error=$4,claim=NULL,next_at=now()+interval '1 minute' WHERE item_id=$1 AND target=$2 AND claim=$3",pg.t("prompt_translations"))).bind(&id).bind(&target).bind(&claim).bind(error).execute(&mut *tx).await.map_err(db_error)?;}}
 tx.commit().await.map_err(db_error)?;guard.commit().await.map_err(db_error)?;Ok(())
}
#[cfg(test)]mod tests{
 use super::*;use crate::admin_security_tests::{state,request};
 #[tokio::test]async fn admin_batch_does_not_consume_personal_translation_quota(){
  let state=state().await;let pg=state.db.as_ref().unwrap();
  pg.upsert_account("quota-owner@example.com",Some("test-password"),"owner").await.unwrap();
  let session=state.issue_session("quota-owner@example.com".into()).await.unwrap();
  sqlx::query(&format!("INSERT INTO {}(id,title,kind,content,visibility) SELECT 'quota-'||n,'Sample','prompt','中文正文','online' FROM generate_series(1,21) n",pg.t("square_items"))).execute(&pg.pool).await.unwrap();
  sqlx::query(&format!("UPDATE {} SET data=jsonb_set(data,'{{enabled}}','true') WHERE id",pg.t("translation_config"))).execute(&pg.pool).await.unwrap();
  assert_eq!(request(&state,"POST","/v1/admin/translation/actions",&session.access_token,json!({"action":"queue"})).await.0,StatusCode::OK);
  let code=request(&state,"POST","/v1/square/items/quota-1/translations/en",&session.access_token,json!(null)).await.0;
  assert_eq!(code,StatusCode::OK,"Batch work must not exhaust the administrator's personal translation quota");
  let used:i32=sqlx::query_scalar(&format!("SELECT requests FROM {} WHERE actor='quota-owner@example.com' AND day=CURRENT_DATE",pg.t("translation_personal_usage"))).fetch_one(&pg.pool).await.unwrap();assert_eq!(used,1);
  sqlx::query(&format!("UPDATE {} SET requests=20",pg.t("translation_personal_usage"))).execute(&pg.pool).await.unwrap();
  assert_eq!(request(&state,"POST","/v1/square/items/quota-1/translations/en",&session.access_token,json!(null)).await.0,StatusCode::OK);
  assert_eq!(request(&state,"POST","/v1/square/items/quota-2/translations/en",&session.access_token,json!(null)).await.0,StatusCode::TOO_MANY_REQUESTS);
  assert!(pg.translation_versions("quota-2").await.unwrap()["en"].is_null());
  sqlx::query(&format!("DROP SCHEMA {} CASCADE",pg.schema)).execute(&pg.pool).await.unwrap();
 }
 #[tokio::test]async fn versions_are_private_until_online_and_invalidated_by_source_changes(){
  let state=state().await;let pg=state.db.as_ref().unwrap();
  sqlx::query(&format!("INSERT INTO {}(id,title,kind,content,visibility) VALUES('translate-test','Hello','prompt','Hello {{name}}','online')",pg.t("square_items"))).execute(&pg.pool).await.unwrap();
  sqlx::query(&format!("INSERT INTO {}(item_id,target,status,data,actor) VALUES('translate-test','zh','ready',$1,'test')",pg.t("prompt_translations"))).bind(json!({"title":"你好","content":"你好 {name}"})).execute(&pg.pool).await.unwrap();
  let (code,versions)=request(&state,"GET","/v1/square/items/translate-test/translations","",json!(null)).await;assert_eq!(code,StatusCode::OK);assert_eq!(versions["zh"]["version"]["title"],"你好");
  let (code,browse)=request(&state,"GET","/v1/square/browse?q=%E4%BD%A0%E5%A5%BD","",json!(null)).await;assert_eq!(code,StatusCode::OK);assert_eq!(browse["items"][0]["title"],"你好");assert_eq!(browse["total"],1);
  let (_,original)=request(&state,"GET","/v1/square/browse?q=%E4%BD%A0%E5%A5%BD&content_language=original","",json!(null)).await;assert_eq!(original["items"][0]["title"],"Hello");
  assert_eq!(request(&state,"POST","/v1/square/items/translate-test/translations/en","",json!(null)).await.0,StatusCode::UNAUTHORIZED);
  assert_eq!(request(&state,"GET","/v1/admin/translation","",json!(null)).await.0,StatusCode::UNAUTHORIZED);
  sqlx::query(&format!("UPDATE {} SET download_count=1 WHERE id='translate-test'",pg.t("square_items"))).execute(&pg.pool).await.unwrap();assert!(!pg.translation_versions("translate-test").await.unwrap()["zh"].is_null());
  sqlx::query(&format!("UPDATE {} SET content='Changed source' WHERE id='translate-test'",pg.t("square_items"))).execute(&pg.pool).await.unwrap();assert_eq!(pg.translation_versions("translate-test").await.unwrap(),json!({}));
  sqlx::query(&format!("UPDATE {} SET visibility='offline' WHERE id='translate-test'",pg.t("square_items"))).execute(&pg.pool).await.unwrap();assert_eq!(request(&state,"GET","/v1/square/items/translate-test/translations","",json!(null)).await.0,StatusCode::NOT_FOUND);
  sqlx::query(&format!("DROP SCHEMA {} CASCADE",pg.schema)).execute(&pg.pool).await.unwrap();
 }
 #[tokio::test]async fn owner_controls_queue_and_models_and_discovery_rejects_unknown_credentials(){
  let state=state().await;let pg=state.db.as_ref().unwrap();pg.upsert_account("translation-owner@example.com",Some("test-password"),"owner").await.unwrap();pg.upsert_account("translation-user@example.com",Some("test-password"),"user").await.unwrap();
  let owner=state.issue_session("translation-owner@example.com".into()).await.unwrap();let user=state.issue_session("translation-user@example.com".into()).await.unwrap();
  assert_eq!(request(&state,"GET","/v1/admin/translation",&user.access_token,json!(null)).await.0,StatusCode::FORBIDDEN);
  let (code,view)=request(&state,"GET","/v1/admin/translation",&owner.access_token,json!(null)).await;assert_eq!(code,StatusCode::OK);assert!(!view.to_string().contains("encrypted_key"));
  let cfg=json!({"revision":0,"endpoint":"https://example.com/v1/chat/completions","model":"qwen-mt-flash","key":"fixture-private","enabled":false,"daily_tokens":100000,"current_password":"test-password"});
  assert_eq!(request(&state,"PUT","/v1/admin/translation",&owner.access_token,cfg.clone()).await.0,StatusCode::OK);
  assert_eq!(request(&state,"PUT","/v1/admin/translation",&owner.access_token,cfg).await.0,StatusCode::CONFLICT);
  assert_eq!(request(&state,"POST","/v1/admin/ai/models/discover",&owner.access_token,json!({"endpoint":"https://other.example/v1/chat/completions","translation":true})).await.0,StatusCode::BAD_REQUEST);
  let (code,first)=request(&state,"POST","/v1/admin/translation/actions",&owner.access_token,json!({"action":"queue"})).await;assert_eq!(code,StatusCode::OK);let (_,again)=request(&state,"POST","/v1/admin/translation/actions",&owner.access_token,json!({"action":"queue"})).await;assert!(first["affected"].is_number());assert_eq!(again["affected"],0);
  sqlx::query(&format!("DROP SCHEMA {} CASCADE",pg.schema)).execute(&pg.pool).await.unwrap();
 }
}

#[cfg(test)]mod live_tests{
 use super::*;
 #[tokio::test]#[ignore="explicit live provider evaluation"]
 async fn live_translation_worker_preserves_templates_and_json(){
  let key=std::env::var("CUETUCK_TRANSLATION_TEST_KEY").expect("test key required");
  let state=crate::admin_security_tests::state().await;let pg=state.db.as_ref().unwrap();
  let config=Config{revision:1,endpoint:"https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".into(),model:"qwen-mt-flash".into(),enabled:true,daily_tokens:1_000_000,encrypted_key:crate::oauth_admin::seal(&state.oauth_config.key,"translation",&key).unwrap()};
  sqlx::query(&format!("UPDATE {} SET data=$1",pg.t("translation_config"))).bind(json!(config)).execute(&pg.pool).await.unwrap();
  let samples=vec!["Write a concise report about {{topic}} for {argument name=\"audience\" default=\"engineers\"}. Keep the code unchanged: `npm run test`. Cite https://example.com/source .".to_owned(),json!({"image_type":"photo","prompt":"A warm studio portrait of {{person}}, soft side lighting.","metadata":{"model":"sample-model","seed":42},"labels":["gentle lighting"]}).to_string(),format!("Describe the following scene in detail. {{subject}}\n{}","Use soft natural lighting with a balanced composition. Preserve the sense of scale, realistic texture and visual hierarchy.\n".repeat(65))];
  for (index,text) in samples.iter().enumerate(){let id=format!("live-translate-{index}");sqlx::query(&format!("INSERT INTO {}(id,title,kind,content,visibility) VALUES($1,'Translation sample','prompt',$2,'online')",pg.t("square_items"))).bind(&id).bind(text).execute(&pg.pool).await.unwrap();sqlx::query(&format!("INSERT INTO {}(item_id,target,actor) VALUES($1,'zh','live-test')",pg.t("prompt_translations"))).bind(&id).execute(&pg.pool).await.unwrap();run_one(&state).await.unwrap();let rows=pg.translation_versions(&id).await.unwrap();assert_eq!(rows["zh"]["status"],"ready","{}",rows["zh"]["error"]);let translated=rows["zh"]["version"]["content"].as_str().unwrap();assert_ne!(translated,text);crate::prompt_translation::validate_pair(text,translated).unwrap();if index==1{let value:Value=serde_json::from_str(translated).unwrap();assert_eq!(value["image_type"],"photo");assert_eq!(value["metadata"]["model"],"sample-model");assert_eq!(value["metadata"]["seed"],42);}println!("live sample {index}: ready, source bytes {}, translated bytes {}",text.len(),translated.len());}
  sqlx::query(&format!("DROP SCHEMA {} CASCADE",pg.schema)).execute(&pg.pool).await.unwrap();
 }
}
