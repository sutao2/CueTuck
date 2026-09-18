//! Reviews immutable Skill bundles as data. Scripts are never executed.
use crate::{admin_ai::{self, Run}, admin_moderation::Policy, admin_risk::audit, ai_jobs, postgres::Pg, skill_bundle::Bundle, AppState};
use axum::http::StatusCode;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use sqlx::Row;
fn db(_:sqlx::Error)->StatusCode { StatusCode::INTERNAL_SERVER_ERROR }

fn input(bundle:&Bundle,title:&str,description:&str)->Result<(String,Vec<String>),String> {
    let decoded=bundle.validate()?;
    let mut files=vec![]; let mut images=vec![]; let mut image_bytes=0;
    for (file,bytes) in bundle.files.iter().zip(decoded) {
        let ext=file.path.rsplit('.').next().unwrap_or("").to_lowercase();
        let mime=match ext.as_str(){"png"=>Some("image/png"),"jpg"|"jpeg"=>Some("image/jpeg"),"webp"=>Some("image/webp"),_=>None};
        if let Some(mime)=mime {
            crate::media::validate_file(file.path.rsplit('/').next().unwrap_or(&file.path),mime,&bytes).map_err(|_|"Skill 图片校验失败，转人工")?;
            image_bytes+=bytes.len();
            if images.len()>=12 || image_bytes>20*1024*1024 {return Err("Skill 图片超出 12 张 / 20 MiB，转人工".into())}
            images.push(format!("data:{mime};base64,{}",STANDARD.encode(bytes)));
            files.push(json!({"path":file.path,"image_index":images.len(),"executable":file.executable}));
        } else {
            if matches!(ext.as_str(),"pdf"|"gif"|"zip"|"docx"|"xlsx"|"pptx"|"exe"|"dll"|"wasm") {return Err("Skill 包含未支持的文件类型，转人工".into())}
            let text=std::str::from_utf8(&bytes).map_err(|_|"Skill 包含未支持的二进制文件，转人工")?;
            if text.chars().any(|c|c.is_control() && !matches!(c,'\n'|'\r'|'\t')) {return Err("Skill 包含非文本文件，转人工".into())}
            files.push(json!({"path":file.path,"text":text,"executable":file.executable}));
        }
    }
    let text=json!({"kind":"skill","title":title,"description":description,"files":files}).to_string();
    if text.len()>128_000 {return Err("Skill 全部文本超过 128 KB，转人工；未截断送审".into())}
    Ok((text,images))
}

pub(crate) async fn run_one(state:&AppState)->Result<bool,StatusCode> {
    let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let Some((id,claim))=pg.claim_review_job(true).await? else {return Ok(false)};
    if screen(state,&id,&claim).await.is_err() {
        let mut tx=pg.pool.begin().await.map_err(db)?;
        ai_jobs::finish_review(pg,&mut tx,&id,&claim,Some("Skill 审核依赖不可用，可有限重试"),true).await?;
        tx.commit().await.map_err(db)?;
    }
    Ok(true)
}
async fn screen(state:&AppState,id:&str,claim:&str)->Result<(),StatusCode> {
    let pg=state.db.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let row=sqlx::query(&format!("SELECT title,description,category,bundle,moderation FROM {} WHERE id=$1",pg.t("skill_publications"))).bind(id).fetch_one(&pg.pool).await.map_err(db)?;
    let initial:Value=row.get("moderation");
    let config=pg.ai_config().await?;
    let policy:Value=sqlx::query_scalar(&format!("SELECT data FROM {} WHERE id=1",pg.t("moderation_policy"))).fetch_one(&pg.pool).await.map_err(db)?;
    let mut runs=vec![]; let mut error=None;
    let bundle:Bundle=serde_json::from_value(row.get("bundle")).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
    if policy["enabled"]!=true || policy["ai_decides"]!=true || initial["policy_revision"]!=policy["revision"] {
        error=Some("审核策略已变化，转人工".into());
    } else {
        match input(&bundle,row.get("title"),row.get("description")) {
            Err(reason)=>error=Some(reason),
            Ok((text,images))=>{
                let category:String=row.get("category");
                let work=async {
                    for skill in config.skills.iter().filter(|s|s.enabled && s.kinds.iter().any(|k|k=="skill") && (s.categories.is_empty() || s.categories.contains(&category))) {
                        runs.push(admin_ai::run_review(&config,skill,&state.oauth_config.key,&text,&images,true).await);
                    }
                };
                if tokio::time::timeout(std::time::Duration::from_secs(25),work).await.is_err() {error=Some("Skill 审核超过 25 秒，转人工".into())}
                if runs.is_empty() && error.is_none() {error=Some("没有匹配的 Skill 文件包审核路由，转人工".into())}
            }
        }
    }
    finish(pg,id,claim,&initial,config.revision,&runs,error).await
}

pub(crate) async fn finish(pg:&Pg,id:&str,claim:&str,initial:&Value,revision:i64,runs:&[Run],mut error:Option<String>)->Result<(),StatusCode> {
    let mut tx=pg.pool.begin().await.map_err(db)?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))").bind(format!("{}:ai-config",pg.schema)).execute(&mut *tx).await.map_err(db)?;
    let current:i64=sqlx::query_scalar(&format!("SELECT max(revision) FROM {}",pg.t("ai_configuration"))).fetch_one(&mut *tx).await.map_err(db)?;
    let policy:Value=sqlx::query_scalar(&format!("SELECT data FROM {} WHERE id=1 FOR SHARE",pg.t("moderation_policy"))).fetch_one(&mut *tx).await.map_err(db)?;
    let policy:Policy=serde_json::from_value(policy).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
    if current!=revision || initial["policy_revision"]!=policy.revision || !policy.enabled || !policy.ai_decides || !policy.auto_approve || !policy.require_ai || !policy.check_images {error=Some("审核期间策略或模型版本变化，转人工".into())}
    ai_jobs::lock_review_claim(pg,&mut tx,id,claim,true).await?;
    let status:String=sqlx::query_scalar(&format!("SELECT status FROM {} WHERE id=$1 FOR UPDATE",pg.t("skill_publications"))).bind(id).fetch_one(&mut *tx).await.map_err(db)?;
    if status!="pending" {
        ai_jobs::finish_review(pg,&mut tx,id,claim,None,true).await?;
        tx.commit().await.map_err(db)?;return Ok(());
    }
    let decision=if error.is_none(){admin_ai::direct_decision(runs,revision)}else{None};
    let reasons=if decision==Some("rejected"){admin_ai::rejection_reasons(runs)}else{vec![]};
    let reason=if decision==Some("approved"){"AI 审核通过".into()}else{reasons.join("；")};
    let moderation=json!({"policy_revision":policy.revision,"decision":decision.unwrap_or("manual"),"ai":{"revision":revision,"runs":runs,"error":error},"reasons":reasons});
    if let Some(status)=decision {
        let history=json!([{"status":status,"reason":reason,"source":"ai","at":chrono::Utc::now()}]);
        sqlx::query(&format!("UPDATE {} SET status=$2,reason=$3,history=history||$4::jsonb WHERE id=$1",pg.t("skill_publications"))).bind(id).bind(status).bind(&reason).bind(history).execute(&mut *tx).await.map_err(db)?;
    }
    sqlx::query(&format!("UPDATE {} SET moderation=$2 WHERE id=$1",pg.t("skill_publications"))).bind(id).bind(&moderation).execute(&mut *tx).await.map_err(db)?;
    audit(pg,&mut tx,"system:ai","skill_ai_reviewed",json!({"id":id,"status":decision.unwrap_or("pending"),"revision":revision})).await?;
    let failed=runs.iter().any(|r|r.error.is_some()) || error.as_deref().is_some_and(|s|s.contains("超过 25 秒"));
    ai_jobs::finish_review(pg,&mut tx,id,claim,failed.then_some("模型链路失败，将有限重试；可人工处置"),true).await?;
    tx.commit().await.map_err(db)?;Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_files_are_reviewed_and_unsupported_content_is_not_silently_omitted() {
        let mut b=Bundle{name:"review".into(),files:vec![crate::skill_bundle::BundleFile{path:"SKILL.md".into(),content:STANDARD.encode("# Review"),executable:false},crate::skill_bundle::BundleFile{path:"scripts/check.sh".into(),content:STANDARD.encode("echo check"),executable:true}]};
        let (text,images)=input(&b,"Review","test").unwrap();assert!(text.contains("echo check"));assert!(images.is_empty());
        b.files[1].content=STANDARD.encode([0,1,2]);assert!(input(&b,"Review","test").is_err());
        b.files[1].content=STANDARD.encode("x".repeat(128001));assert!(input(&b,"Review","test").is_err());
    }
    #[test]
    fn nested_skill_images_are_checked_and_image_limit_never_truncates() {
        let mut b=Bundle{name:"review".into(),files:vec![crate::skill_bundle::BundleFile{path:"SKILL.md".into(),content:STANDARD.encode("# Review"),executable:false}]};
        for i in 0..12 {b.files.push(crate::skill_bundle::BundleFile{path:format!("images/cover{i}.png"),content:STANDARD.encode(b"\x89PNG\r\n\x1a\nfixture"),executable:false});}
        let (text,images)=input(&b,"Review","test").unwrap();assert_eq!(images.len(),12);assert!(text.contains("images/cover11.png"));
        b.files.push(crate::skill_bundle::BundleFile{path:"images/extra.png".into(),content:STANDARD.encode(b"\x89PNG\r\n\x1a\nfixture"),executable:false});
        assert!(input(&b,"Review","test").is_err());
    }

    #[tokio::test]
    async fn skill_decisions_are_durable_atomic_and_never_overwrite_humans() {
        use crate::admin_security_tests::{state,request};
        use crate::ai_transport::Verdict;
        let s=state().await;let pg=s.db.as_ref().unwrap();
        pg.upsert_account("skill-review@test.local",Some("test-password"),"owner").await.unwrap();
        pg.put_profile("skill-review@test.local",Some("作者"),None).await.unwrap();
        let token=s.issue_session("skill-review@test.local".into()).await.unwrap().access_token;
        let policy=Policy{enabled:true,ai_decides:true,auto_approve:true,..Default::default()};
        sqlx::query(&format!("UPDATE {} SET data=$1 WHERE id=1",pg.t("moderation_policy"))).bind(json!(policy)).execute(&pg.pool).await.unwrap();
        for scenario in ["approve","reject","human","policy","model","failed","audit"] {
            let body=json!({"request_id":uuid::Uuid::new_v4().to_string(),"title":"代码审查","description":"审阅变更","category":"development","license":"MIT","confirm_public":true,"bundle":{"name":"review","files":[{"path":"SKILL.md","content":STANDARD.encode("# Review"),"executable":false}]}});
            let (status,p)=request(&s,"POST","/v1/skills",&token,body.clone()).await;assert_eq!(status,StatusCode::OK,"{p}");
            let id=p["id"].as_str().unwrap();
            assert_eq!(request(&s,"POST","/v1/skills",&token,body).await.1["id"],id);
            pg.apply_schema(false).await.unwrap(); // Queue and foreign key survive migration.
            let (job,old)=pg.claim_review_job(true).await.unwrap().unwrap();assert_eq!(job,id);
            sqlx::query(&format!("UPDATE {} SET lease_until=now()-interval '1 second' WHERE publication_id=$1",pg.t("skill_ai_jobs"))).bind(id).execute(&pg.pool).await.unwrap();
            let (_,claim)=pg.claim_review_job(true).await.unwrap().unwrap();assert_ne!(claim,old);
            let initial=json!({"policy_revision":0});
            let runs=vec![Run{revision:0,skill_id:"general".into(),verdict:Some(Verdict{risk_score:90,decision:if scenario=="reject"{"reject".into()}else{"approve".into()},reasons:vec!["存在明确风险，请修改".into()],matched_rules:vec![]}),models:vec![],error:(scenario=="failed").then(||"timeout".into())}];
            assert_eq!(finish(pg,id,&old,&initial,0,&runs,None).await.err(),Some(StatusCode::CONFLICT));
            if scenario=="human" {assert_eq!(request(&s,"POST",&format!("/v1/admin/skills/{id}/review"),&token,json!({"expected_status":"pending","status":"rejected","reason":"人工决定"})).await.0,StatusCode::OK);}
            if scenario=="policy" {sqlx::query(&format!("UPDATE {} SET data=jsonb_set(data,'{{revision}}','1')",pg.t("moderation_policy"))).execute(&pg.pool).await.unwrap();}
            if scenario=="audit" {sqlx::query(&format!("ALTER TABLE {} ADD CONSTRAINT fail_skill_ai CHECK(action<>'skill_ai_reviewed') NOT VALID",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();}
            let result=finish(pg,id,&claim,&initial,if scenario=="model"{1}else{0},&runs,None).await;
            if scenario=="audit" {assert!(result.is_err());} else {result.unwrap();}
            let value:Value=sqlx::query_scalar(&format!("SELECT jsonb_build_object('status',status,'reason',reason,'history',history) FROM {} WHERE id=$1",pg.t("skill_publications"))).bind(id).fetch_one(&pg.pool).await.unwrap();
            assert_eq!(value["status"],match scenario{"approve"=>"approved","reject"|"human"=>"rejected",_=>"pending"});
            if scenario=="human" {assert_eq!(value["reason"],"人工决定");}
            if scenario=="reject" {assert!(value["reason"].as_str().unwrap().contains("明确风险"));}
            if scenario=="policy" {sqlx::query(&format!("UPDATE {} SET data=$1",pg.t("moderation_policy"))).bind(json!(policy)).execute(&pg.pool).await.unwrap();}
            if scenario=="failed" {
                assert_eq!(request(&s,"POST",&format!("/v1/admin/ai/jobs/{id}/retry?kind=skill"),&token,json!({})).await.0,StatusCode::OK);
                let (_,third)=pg.claim_review_job(true).await.unwrap().unwrap();
                finish(pg,id,&third,&initial,0,&runs,None).await.unwrap();
                assert!(pg.claim_review_job(true).await.unwrap().is_none());
            }
        }
        let (_,jobs)=request(&s,"GET","/v1/admin/ai/jobs",&token,json!({})).await;
        assert!(jobs["items"].as_array().unwrap().iter().all(|j|j["kind"]=="skill"));
        assert_eq!(jobs["total"],7);
    }

}
