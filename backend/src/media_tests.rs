use super::*;
use axum::response::IntoResponse;
use axum::{
    body::{to_bytes, Body, Bytes},
    extract::{Path, State},
    http::Request,
    routing::put,
};
use serde_json::{json, Value};
use sha2::Digest;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use tower::ServiceExt;

#[derive(Clone, Default)]
struct Store {
    objects: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    gets: Arc<AtomicUsize>,
    deletes: Arc<AtomicUsize>,
    mode: Arc<AtomicUsize>,
}
struct Server(tokio::task::JoinHandle<()>);

#[tokio::test]
async fn bucket_scan_paginates_decodes_keys_and_never_removes_unknown_objects(){
    let (state,a,_b,admin,store,_server)=fixture().await;
    let (_,known)=upload(&state,&a,"known.txt","text/plain",b"test",false).await;
    for n in 0..3 {store.objects.lock().unwrap().insert(format!("private-test/promptark/unknown-{n}"),b"test".to_vec());}
    let mut cursor=String::new();let mut unknown=vec![];let mut checked=0;
    loop {
        let (status,page)=crate::admin_security_tests::request(&state,"GET",&format!("/v1/admin/media/scan?side=bucket&cursor={cursor}"),&admin,json!({})).await;
        assert_eq!(status,StatusCode::OK,"{page}");checked+=page["checked"].as_u64().unwrap();unknown.extend(page["items"].as_array().unwrap().clone());
        let Some(next)=page["next_cursor"].as_str() else {break};cursor=next.to_string();
    }
    assert_eq!(checked,4);assert_eq!(unknown.len(),3);assert_eq!(store.deletes.load(Ordering::SeqCst),0);
    store.objects.lock().unwrap().remove(&format!("private-test/promptark/{}",known["id"].as_str().unwrap()));
    let (_,page)=crate::admin_security_tests::request(&state,"GET","/v1/admin/media/scan?side=database",&admin,json!({})).await;assert_eq!(page["items"][0]["status"],"missing");
    store.mode.store(3,Ordering::SeqCst);assert_eq!(crate::admin_security_tests::request(&state,"GET","/v1/admin/media/scan?side=bucket",&admin,json!({})).await.0,StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn interrupted_upload_cleanup_is_aged_explicit_and_scan_is_read_only(){
    let (state,a,_b,admin,store,_server)=fixture().await;let pg=state.db.as_ref().unwrap();
    let (_,file)=upload(&state,&a,"interrupted.txt","text/plain",b"test",false).await;let id=file["id"].as_str().unwrap();
    sqlx::query(&format!("UPDATE {} SET ready=FALSE WHERE id=$1",pg.t("media_objects"))).bind(id).execute(&pg.pool).await.unwrap();
    assert_eq!(reclaim(&state,&admin,id).await.0,StatusCode::CONFLICT);
    let (status,scan)=crate::admin_security_tests::request(&state,"GET","/v1/admin/media/scan?side=database",&admin,json!({})).await;
    assert_eq!(status,StatusCode::OK);assert_eq!(scan["items"][0]["status"],"incomplete");assert_eq!(store.deletes.load(Ordering::SeqCst),0);
    assert_eq!(crate::admin_security_tests::request(&state,"GET","/v1/admin/media/scan?side=database",&a,json!({})).await.0,StatusCode::FORBIDDEN);
    age_media(&state,id).await;let (_,list)=crate::admin_security_tests::request(&state,"GET","/v1/admin/media/orphans",&admin,json!({})).await;assert_eq!(list["items"][0]["ready"],false);
    assert_eq!(reclaim(&state,&admin,id).await.0,StatusCode::OK);assert_eq!(store.deletes.load(Ordering::SeqCst),1);
}

#[tokio::test]
async fn vision_reads_only_selected_owned_verified_images(){
    let (state,a,_b,_admin,store,_server)=fixture().await;
    let (_,file)=upload(&state,&a,"preview.png","image/png",b"\x89PNG\r\n\x1a\nfixture",false).await;
    let reference=json!({"id":Uuid::new_v4().to_string(),"media_id":file["id"],"name":file["name"],"mime":file["mime"],"size":file["size"],"sha256":file["sha256"]});
    let (_,p)=crate::admin_security_tests::request(&state,"POST","/v1/publications",&a,json!({"source_id":"vision","title":"Image","content":"Check image","asset_refs":[reference]})).await;
    let publication:Publication=serde_json::from_value(p).unwrap();
    let images=media::moderation_images(&state,&publication).await.unwrap();assert_eq!(images.len(),1);assert!(images[0].starts_with("data:image/png;base64,"));
    let payload=crate::ai_transport::user_content("check",&images);assert_eq!(payload[1]["image_url"]["url"],images[0]);
    store.mode.store(1,Ordering::SeqCst);assert!(media::moderation_images(&state,&publication).await.is_err());
    let mut forged=publication;forged.asset_refs[0].id=Uuid::new_v4().to_string();assert!(media::moderation_images(&state,&forged).await.is_err());
}

async fn public_fixture() -> (AppState, String, String, String, Value, String, Store, Server) {
    let (state, a, b, admin, store, server) = fixture().await;
    let (_, media) = upload(&state, &a, "notes.txt", "text/plain", b"selected public notes", false).await;
    let reference = json!({"id":Uuid::new_v4().to_string(),"media_id":media["id"],"name":media["name"],"mime":media["mime"],"size":media["size"],"sha256":media["sha256"]});
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!("UPDATE {} SET data=$1 WHERE id=1", pg.t("moderation_policy")))
        .bind(json!({"enabled":true,"auto_approve":true,"require_ai":false,"check_images":false,"check_structure":false,"check_sensitive":false,"check_duplicates":false}))
        .execute(&pg.pool).await.unwrap();
    let (status, publication) = crate::admin_security_tests::request(&state,"POST","/v1/publications",&a,json!({"source_id":"file-source","title":"公开资料","content":"已经确认公开的正文","asset_refs":[reference]})).await;
    assert_eq!(status, StatusCode::OK, "{publication}");
    assert_eq!(publication["status"], "pending", "files must never auto-approve");
    let id = publication["id"].as_str().unwrap().to_owned();
    (state,a,b,admin,reference,id,store,server)
}

#[tokio::test]
async fn publication_files_require_selected_scope_review_and_online_visibility() {
    let (state,a,b,admin,reference,id,store,_server) = public_fixture().await;
    let public = format!("/v1/square/items/{id}/assets/{}",reference["id"].as_str().unwrap());
    let review = format!("/v1/admin/publications/{id}/assets/{}",reference["id"].as_str().unwrap());
    assert_eq!(get(&state,"",&public).await.status(),StatusCode::NOT_FOUND);
    assert_eq!(get(&state,&a,&review).await.status(),StatusCode::FORBIDDEN);
    assert_eq!(get(&state,&b,&review).await.status(),StatusCode::FORBIDDEN);
    let response = get(&state,&admin,&review).await;
    assert_eq!(response.status(),StatusCode::OK);
    assert_eq!(response.headers()["cache-control"],"no-store");
    assert_eq!(get(&state,&admin,&format!("/v1/media/{}/content",reference["media_id"].as_str().unwrap())).await.status(),StatusCode::NOT_FOUND);
    assert_eq!(get(&state,&admin,&format!("/v1/admin/publications/{id}/assets/{}",Uuid::new_v4())).await.status(),StatusCode::NOT_FOUND);
    let (status,_) = crate::admin_security_tests::request(&state,"POST",&format!("/v1/admin/publications/{id}/approve"),&admin,json!({})).await;
    assert_eq!(status,StatusCode::OK);
    assert_eq!(get(&state,"",&public).await.status(),StatusCode::OK);
    state.set_square_public(false).await.unwrap();
    assert_eq!(get(&state,"",&public).await.status(),StatusCode::UNAUTHORIZED);
    assert_eq!(get(&state,&b,&public).await.status(),StatusCode::OK);
    state.set_square_public(true).await.unwrap();
    let (status, content) = crate::admin_security_tests::request(&state,"GET",&format!("/v1/square/items/{id}/content"),"",json!(null)).await;
    assert_eq!(status,StatusCode::OK); assert_eq!(content["asset_refs"],json!([reference]));
    store.mode.store(1,Ordering::SeqCst);
    assert_eq!(get(&state,"",&public).await.status(),StatusCode::BAD_GATEWAY);
    store.mode.store(0,Ordering::SeqCst);
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!("UPDATE {} SET role='user' WHERE email='owner@example.com'",pg.t("accounts"))).execute(&pg.pool).await.unwrap();
    assert_eq!(get(&state,&admin,&review).await.status(),StatusCode::FORBIDDEN);
    for visibility in ["offline","trashed"] {
        sqlx::query(&format!("UPDATE {} SET visibility=$2 WHERE id=$1",pg.t("square_items"))).bind(&id).bind(visibility).execute(&pg.pool).await.unwrap();
        assert_eq!(get(&state,"",&public).await.status(),StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn publication_files_reject_foreign_forged_and_unassigned_references() {
    let (state,a,b,_admin,reference,_id,_store,_server) = public_fixture().await;
    let body = json!({"source_id":"another","title":"测试","content":"正文","asset_refs":[reference]});
    assert_eq!(crate::admin_security_tests::request(&state,"POST","/v1/publications",&b,body.clone()).await.0,StatusCode::NOT_FOUND);
    let mut forged = body.clone(); forged["asset_refs"][0]["name"] = json!("forged.txt");
    assert_eq!(crate::admin_security_tests::request(&state,"POST","/v1/publications",&a,forged).await.0,StatusCode::NOT_FOUND);
    let mut collection = body; collection["kind"] = json!("collection"); collection["members"] = json!([{"title":"member","content":"body"}]);
    assert_eq!(crate::admin_security_tests::request(&state,"POST","/v1/publications",&a,collection).await.0,StatusCode::BAD_REQUEST);
}
#[tokio::test]
async fn collection_files_validate_member_scope_and_keep_manual_review() {
    let (state,a,b,admin,reference,_id,_store,_server) = public_fixture().await;
    let body = json!({"kind":"collection","source_id":"collection-source","title":"合集文件","content":"摘要","asset_refs":[reference],
        "members":[{"title":"第一篇","content":"正文一","asset_ids":[reference["id"]]},{"title":"第二篇","content":"正文二"}]});
    // Invalid association requests must fail before storing any snapshot.
    let pg = state.db.as_ref().unwrap();
    let before: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {}",pg.t("publications"))).fetch_one(&pg.pool).await.unwrap();
    for ids in [json!([]),json!(["unknown"]),json!([reference["id"],reference["id"]])] {
        let mut bad = body.clone(); bad["members"][0]["asset_ids"] = ids;
        assert_eq!(crate::admin_security_tests::request(&state,"POST","/v1/publications",&a,bad).await.0,StatusCode::BAD_REQUEST);
    }
    let mut duplicate = body.clone(); duplicate["members"][1]["asset_ids"] = json!([reference["id"]]);
    assert_eq!(crate::admin_security_tests::request(&state,"POST","/v1/publications",&a,duplicate).await.0,StatusCode::BAD_REQUEST);
    assert_eq!(crate::admin_security_tests::request(&state,"POST","/v1/publications",&b,body.clone()).await.0,StatusCode::NOT_FOUND);
    let mut forged = body.clone(); forged["asset_refs"][0]["sha256"] = json!("0".repeat(64));
    assert_eq!(crate::admin_security_tests::request(&state,"POST","/v1/publications",&a,forged).await.0,StatusCode::NOT_FOUND);
    let after: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {}",pg.t("publications"))).fetch_one(&pg.pool).await.unwrap();
    assert_eq!(before,after);
    let (status, publication) = crate::admin_security_tests::request(&state,"POST","/v1/publications",&a,body.clone()).await;
    assert_eq!(status,StatusCode::OK); assert_eq!(publication["status"],"pending"); assert_eq!(publication["members"][0]["asset_ids"],body["members"][0]["asset_ids"]);
    let id = publication["id"].as_str().unwrap();
    let public = format!("/v1/square/items/{id}/assets/{}",reference["id"].as_str().unwrap());
    assert_eq!(get(&state,"",&public).await.status(),StatusCode::NOT_FOUND);
    assert_eq!(get(&state,&admin,&format!("/v1/admin/publications/{id}/assets/{}",reference["id"].as_str().unwrap())).await.status(),StatusCode::OK);
    assert_eq!(crate::admin_security_tests::request(&state,"POST",&format!("/v1/admin/publications/{id}/approve"),&admin,json!({})).await.0,StatusCode::OK);
    let (_,content) = crate::admin_security_tests::request(&state,"GET",&format!("/v1/square/items/{id}/content"),"",json!(null)).await;
    assert_eq!(content["members"],publication["members"]); assert_eq!(content["asset_refs"],body["asset_refs"]);
    assert_eq!(get(&state,"",&public).await.status(),StatusCode::OK);
    sqlx::query(&format!("UPDATE {} SET visibility='offline' WHERE id=$1",pg.t("square_items"))).bind(id).execute(&pg.pool).await.unwrap();
    assert_eq!(get(&state,"",&public).await.status(),StatusCode::NOT_FOUND);
}
impl Drop for Server {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn fixture() -> (AppState, String, String, String, Store, Server) {
    let store = Store::default();
    let router = Router::new()
        .route("/private-test/",axum::routing::get(|State(s):State<Store>,axum::extract::Query(q):axum::extract::Query<HashMap<String,String>>|async move{
            if s.mode.load(Ordering::SeqCst)==3{return (StatusCode::BAD_GATEWAY,String::new())}
            let mut keys:Vec<_>=s.objects.lock().unwrap().keys().filter_map(|k|k.strip_prefix("private-test/").map(str::to_owned)).collect();keys.sort();
            let start=q.get("continuation-token").and_then(|v|v.parse::<usize>().ok()).unwrap_or(0);
            let entries=keys.iter().skip(start).take(2).map(|k|format!("<Contents><Key>{}</Key><ETag>fixture</ETag><Size>4</Size><LastModified>2026-01-01T00:00:00Z</LastModified></Contents>",urlencoding::encode(k))).collect::<String>();
            let next=if start+2<keys.len(){format!("<NextContinuationToken>{}</NextContinuationToken>",start+2)}else{String::new()};
            (StatusCode::OK,format!("<ListBucketResult>{entries}{next}</ListBucketResult>"))
        }))
        .route(
            "/*key",
            put(
                |State(s): State<Store>, Path(key): Path<String>, body: Bytes| async move {
                    if s.mode.load(Ordering::SeqCst) == 3 {
                        return StatusCode::BAD_GATEWAY;
                    }
                    s.objects.lock().unwrap().insert(key, body.to_vec());
                    StatusCode::OK
                },
            )
            .get(
                |State(s): State<Store>, Path(key): Path<String>| async move {
                    s.gets.fetch_add(1, Ordering::SeqCst);
                    if s.mode.load(Ordering::SeqCst) == 4 {
                        return axum::response::Redirect::temporary("http://127.0.0.1:1/blocked")
                            .into_response();
                    }
                    match s.objects.lock().unwrap().get(&key).cloned() {
                        Some(mut bytes) => {
                            if s.mode.load(Ordering::SeqCst) == 1 {
                                bytes[0] ^= 1;
                            }
                            if s.mode.load(Ordering::SeqCst) == 2 {
                                bytes.push(0);
                            }
                            (StatusCode::OK, bytes).into_response()
                        }
                        None => StatusCode::NOT_FOUND.into_response(),
                    }
                },
            )
            .delete(
                |State(s): State<Store>, Path(key): Path<String>| async move {
                    s.deletes.fetch_add(1,Ordering::SeqCst);
                    if s.mode.load(Ordering::SeqCst) == 5 { return StatusCode::BAD_GATEWAY; }
                    if s.objects.lock().unwrap().remove(&key).is_some() { StatusCode::NO_CONTENT } else { StatusCode::NOT_FOUND }
                },
            ),
        )
        .layer(axum::extract::DefaultBodyLimit::max(6 * 1024 * 1024))
        .with_state(store.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let server = Server(tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    }));
    let mut state = crate::admin_security_tests::state().await;
    state.media = Some(media::MediaConfig {
        endpoint,
        access_key: "test".into(),
        secret_key: "test".into(),
        bucket: "private-test".into(),
    });
    let pg = state.db.as_ref().unwrap();
    for (email, role) in [
        ("a@example.com", "user"),
        ("b@example.com", "user"),
        ("owner@example.com", "owner"),
    ] {
        pg.upsert_account(email, None, role).await.unwrap();
    }
    let a = state
        .issue_session("a@example.com".into())
        .await
        .unwrap()
        .access_token;
    let b = state
        .issue_session("b@example.com".into())
        .await
        .unwrap()
        .access_token;
    let admin = state
        .issue_session("owner@example.com".into())
        .await
        .unwrap()
        .access_token;
    (state, a, b, admin, store, server)
}
async fn age_media(state: &AppState, id: &str) {
    let pg=state.db.as_ref().unwrap();
    sqlx::query(&format!("UPDATE {} SET last_used_at=now()-interval '8 days' WHERE id=$1",pg.t("media_objects"))).bind(id).execute(&pg.pool).await.unwrap();
}
fn media_ref(media: &Value) -> Value { json!({"id":Uuid::new_v4().to_string(),"media_id":media["id"],"name":media["name"],"mime":media["mime"],"size":media["size"],"sha256":media["sha256"]}) }
async fn reclaim(state: &AppState, admin: &str, id: &str) -> (StatusCode,Value) {
    crate::admin_security_tests::request(state,"POST",&format!("/v1/admin/media/orphans/{id}/purge"),admin,json!({"confirm":true})).await
}

#[tokio::test]
async fn reclaim_excludes_recent_legacy_and_all_historical_references() {
    let (state,a,b,admin,store,_server)=fixture().await;
    let pg=state.db.as_ref().unwrap();
    let mut ids=vec![];
    for name in ["recent.txt","incomplete.txt","legacy.txt","deleted.txt","rejected.txt","orphan.txt"] {
        let (_,media)=upload(&state,&a,name,"text/plain",name.as_bytes(),false).await;
        let id=media["id"].as_str().unwrap().to_owned();
        if name!="recent.txt" { age_media(&state,&id).await; }
        if name=="incomplete.txt" {sqlx::query(&format!("UPDATE {} SET ready=FALSE WHERE id=$1",pg.t("media_objects"))).bind(&id).execute(&pg.pool).await.unwrap();}
        if name=="legacy.txt" {sqlx::query(&format!("UPDATE {} SET object_key='legacy/path' WHERE id=$1",pg.t("media_objects"))).bind(&id).execute(&pg.pool).await.unwrap();}
        if name=="deleted.txt" {pg.put_library_changes("a@example.com",&[library::LibraryChange{id:"deleted".into(),kind:"prompt".into(),payload:json!({"title":"deleted","content":"body","asset_refs":[media_ref(&media)]}),updated_at:"1".into(),deleted_at:Some("1".into())}]).await.unwrap();}
        if name=="rejected.txt" {
            let (_,p)=crate::admin_security_tests::request(&state,"POST","/v1/publications",&a,json!({"source_id":"rejected","title":"rejected","content":"body","asset_refs":[media_ref(&media)]})).await;
            sqlx::query(&format!("UPDATE {} SET status='rejected' WHERE id=$1",pg.t("publications"))).bind(p["id"].as_str().unwrap()).execute(&pg.pool).await.unwrap();
        }
        ids.push(id);
    }
    for token in [&a,&b,""] {assert!(crate::admin_security_tests::request(&state,"GET","/v1/admin/media/orphans",token,json!(null)).await.0.is_client_error());assert!(reclaim(&state,token,&ids[5]).await.0.is_client_error());}
    let (_,list)=crate::admin_security_tests::request(&state,"GET","/v1/admin/media/orphans",&admin,json!(null)).await;
    assert_eq!(list["items"].as_array().unwrap().len(),2);
    assert!(list["items"].as_array().unwrap().iter().any(|r|r["id"]==ids[5]));
    assert!(list["items"].as_array().unwrap().iter().any(|r|r["id"]==ids[1] && r["ready"]==false));
    assert!(!list.to_string().contains("object_key"));assert!(!list.to_string().contains("owner_email"));
    for index in [0,2,3,4] {assert_eq!(reclaim(&state,&admin,&ids[index]).await.0,StatusCode::CONFLICT);}
    age_media(&state,&ids[0]).await;
    sqlx::query(&format!("UPDATE {} SET file_name=NULL WHERE id=$1",pg.t("media_objects"))).bind(&ids[0]).execute(&pg.pool).await.unwrap();
    assert_eq!(reclaim(&state,&admin,&ids[0]).await.0,StatusCode::CONFLICT);
    assert_eq!(store.deletes.load(Ordering::SeqCst),0);
    assert_eq!(reclaim(&state,&admin,&ids[5]).await.0,StatusCode::OK);
    assert_eq!(store.deletes.load(Ordering::SeqCst),1);
    assert_eq!(store.objects.lock().unwrap().len(),5);
    let count:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM {} WHERE action IN ('media_reclaim_started','media_reclaim_completed')",pg.t("security_audit"))).fetch_one(&pg.pool).await.unwrap();assert_eq!(count,2);
}

#[tokio::test]
async fn reclaim_preserves_retry_state_after_storage_and_database_failure() {
    let (state,a,_b,admin,store,_server)=fixture().await;let pg=state.db.as_ref().unwrap();
    let (_,media)=upload(&state,&a,"retry.txt","text/plain",b"retry",false).await;let id=media["id"].as_str().unwrap();age_media(&state,id).await;
    store.mode.store(5,Ordering::SeqCst);
    assert_eq!(reclaim(&state,&admin,id).await.0,StatusCode::BAD_GATEWAY);
    assert_eq!(get(&state,&a,&format!("/v1/media/{id}/content")).await.status(),StatusCode::NOT_FOUND);
    let (_,list)=crate::admin_security_tests::request(&state,"GET","/v1/admin/media/orphans",&admin,json!(null)).await;assert_eq!(list["items"][0]["deleting"],true);
    store.mode.store(0,Ordering::SeqCst);
    sqlx::query(&format!("ALTER TABLE {} ADD CONSTRAINT block_reclaim_audit CHECK (action <> 'media_reclaim_completed') NOT VALID",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    assert!(reclaim(&state,&admin,id).await.0.is_server_error());assert_eq!(store.objects.lock().unwrap().len(),0);
    let count:i64=sqlx::query_scalar(&format!("SELECT count(*) FROM {} WHERE id=$1 AND deleting",pg.t("media_objects"))).bind(id).fetch_one(&pg.pool).await.unwrap();assert_eq!(count,1);
    sqlx::query(&format!("ALTER TABLE {} DROP CONSTRAINT block_reclaim_audit",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    assert_eq!(reclaim(&state,&admin,id).await.0,StatusCode::OK);
    let (_,list)=crate::admin_security_tests::request(&state,"GET","/v1/admin/media/orphans",&admin,json!(null)).await;assert_eq!(list["items"],json!([]));
}

#[tokio::test]
async fn reclaim_lease_and_reference_lock_prevent_dangling_snapshots() {
    let (state,a,_b,admin,store,_server)=fixture().await;let pg=state.db.as_ref().unwrap();
    let (_,media)=upload(&state,&a,"race.txt","text/plain",b"race",false).await;let id=media["id"].as_str().unwrap();age_media(&state,id).await;
    let (_,reused)=upload(&state,&a,"race.txt","text/plain",b"race",false).await;assert_eq!(media["id"],reused["id"]);
    assert_eq!(reclaim(&state,&admin,id).await.0,StatusCode::CONFLICT);age_media(&state,id).await;
    let change=library::LibraryChange{id:"race".into(),kind:"prompt".into(),payload:json!({"title":"race","content":"body","asset_refs":[media_ref(&media)]}),updated_at:"1".into(),deleted_at:None};
    let mut tx=pg.pool.begin().await.unwrap();media_reclaim::reference_lock(pg,&mut tx).await.unwrap();
    let pending=pg.put_library_changes("a@example.com",std::slice::from_ref(&change));tokio::pin!(pending);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(100),&mut pending).await.is_err());
    sqlx::query(&format!("UPDATE {} SET deleting=TRUE WHERE id=$1",pg.t("media_objects"))).bind(id).execute(&mut *tx).await.unwrap();tx.commit().await.unwrap();
    assert!(matches!(pending.await,Err(StatusCode::NOT_FOUND)));
    assert_eq!(crate::admin_security_tests::request(&state,"POST","/v1/publications",&a,json!({"source_id":"race","title":"race","content":"body","asset_refs":[media_ref(&media)]})).await.0,StatusCode::NOT_FOUND);
    assert_eq!(reclaim(&state,&admin,id).await.0,StatusCode::OK);assert_eq!(store.deletes.load(Ordering::SeqCst),1);
}

#[tokio::test]
async fn reclaim_rechecks_references_after_waiting_for_writer_commit() {
    let (state,a,_b,admin,store,_server)=fixture().await;let pg=state.db.as_ref().unwrap();
    let (_,media)=upload(&state,&a,"referenced.txt","text/plain",b"referenced",false).await;let id=media["id"].as_str().unwrap();age_media(&state,id).await;
    let (_,list)=crate::admin_security_tests::request(&state,"GET","/v1/admin/media/orphans",&admin,json!(null)).await;assert_eq!(list["items"][0]["id"],id);
    let mut tx=pg.pool.begin().await.unwrap();media_reclaim::reference_lock(pg,&mut tx).await.unwrap();
    let pending=reclaim(&state,&admin,id);tokio::pin!(pending);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(100),&mut pending).await.is_err());
    sqlx::query(&format!("INSERT INTO {} (owner_email,id,kind,payload,updated_at) VALUES ('a@example.com','concurrent','prompt',$1,'1')",pg.t("library_changes")))
        .bind(json!({"title":"concurrent","content":"body","asset_refs":[media_ref(&media)]}).to_string()).execute(&mut *tx).await.unwrap();tx.commit().await.unwrap();
    assert_eq!(pending.await.0,StatusCode::CONFLICT);assert_eq!(store.deletes.load(Ordering::SeqCst),0);
    assert_eq!(get(&state,&a,&format!("/v1/media/{id}/content")).await.status(),StatusCode::OK);
}

#[tokio::test]
async fn reclaim_requires_confirmation_current_owner_and_durable_claim_audit() {
    let (state,a,b,admin,store,_server)=fixture().await;let pg=state.db.as_ref().unwrap();
    let (_,media)=upload(&state,&a,"permissions.txt","text/plain",b"permissions",false).await;let id=media["id"].as_str().unwrap();age_media(&state,id).await;
    for role in ["reviewer","admin"] {
        sqlx::query(&format!("UPDATE {} SET role=$1 WHERE email='b@example.com'",pg.t("accounts"))).bind(role).execute(&pg.pool).await.unwrap();
        assert_eq!(crate::admin_security_tests::request(&state,"GET","/v1/admin/media/orphans",&b,json!(null)).await.0,StatusCode::FORBIDDEN);
        assert_eq!(reclaim(&state,&b,id).await.0,StatusCode::FORBIDDEN);
    }
    assert_eq!(crate::admin_security_tests::request(&state,"POST",&format!("/v1/admin/media/orphans/{id}/purge"),&admin,json!({"confirm":false})).await.0,StatusCode::BAD_REQUEST);
    sqlx::query(&format!("ALTER TABLE {} ADD CONSTRAINT block_reclaim_claim CHECK (action <> 'media_reclaim_started') NOT VALID",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    assert!(reclaim(&state,&admin,id).await.0.is_server_error());
    let deleting:bool=sqlx::query_scalar(&format!("SELECT deleting FROM {} WHERE id=$1",pg.t("media_objects"))).bind(id).fetch_one(&pg.pool).await.unwrap();assert!(!deleting);
    sqlx::query(&format!("ALTER TABLE {} DROP CONSTRAINT block_reclaim_claim",pg.t("security_audit"))).execute(&pg.pool).await.unwrap();
    let mut tx=pg.pool.begin().await.unwrap();sqlx::query(&format!("SELECT email FROM {} WHERE email='owner@example.com' FOR UPDATE",pg.t("accounts"))).execute(&mut *tx).await.unwrap();
    let pending=reclaim(&state,&admin,id);tokio::pin!(pending);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(100),&mut pending).await.is_err());
    sqlx::query(&format!("UPDATE {} SET role='user' WHERE email='owner@example.com'",pg.t("accounts"))).execute(&mut *tx).await.unwrap();tx.commit().await.unwrap();
    assert_eq!(pending.await.0,StatusCode::FORBIDDEN);assert_eq!(store.deletes.load(Ordering::SeqCst),0);
}

async fn upload(
    state: &AppState,
    token: &str,
    name: &str,
    mime: &str,
    bytes: &[u8],
    repeat: bool,
) -> (StatusCode, Value) {
    let mut body = Vec::new();
    for _ in 0..if repeat { 2 } else { 1 } {
        body.extend_from_slice(format!("--boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{name}\"\r\nContent-Type: {mime}\r\n\r\n").as_bytes());
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(b"--boundary--\r\n");
    let response = app(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/media/upload")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "multipart/form-data; boundary=boundary")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn get(state: &AppState, token: &str, path: &str) -> axum::response::Response {
    app(state.clone())
        .oneshot(
            Request::builder()
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn private_media_roundtrip_survives_restart_and_rechecks_ownership_and_sessions() {
    let (state, a, b, admin, store, _server) = fixture().await;
    let bytes = "私有资料\n测试".as_bytes();
    let (status, value) = upload(&state, &a, "参考.md", "text/plain", bytes, false).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    let path = value["url"].as_str().unwrap();
    assert!(path.starts_with("/v1/media/"));
    assert_eq!(value["size"], bytes.len());
    assert_eq!(
        value["sha256"],
        format!("{:x}", sha2::Sha256::digest(bytes))
    );
    let url_path = format!("/v1/media/{}/url", value["id"].as_str().unwrap());
    for target in [path, &url_path] {
        assert_eq!(
            get(&state, "", target).await.status(),
            StatusCode::UNAUTHORIZED
        );
        for token in [&b, &admin] {
            assert_eq!(
                get(&state, token, target).await.status(),
                StatusCode::NOT_FOUND
            );
        }
    }
    assert_eq!(store.gets.load(Ordering::SeqCst), 0);
    let restarted = AppState {
        db: state.db.clone(),
        media: state.media.clone(),
        ..AppState::default()
    };
    let response = get(&restarted, &a, path).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    assert!(response.headers()["content-disposition"]
        .to_str()
        .unwrap()
        .starts_with("attachment;"));
    assert_eq!(
        to_bytes(response.into_body(), media::MAX_FILE)
            .await
            .unwrap()
            .as_ref(),
        bytes
    );
    let response = get(&state, &a, &url_path).await;
    let returned: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(returned, json!({"url":path}));
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!(
        "UPDATE {} SET disabled=TRUE WHERE email='a@example.com'",
        pg.t("accounts")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        get(&state, &a, path).await.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(store.gets.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn private_upload_rejects_invalid_or_oversized_files_without_storage_writes() {
    let (state, a, _, _, store, _server) = fixture().await;
    assert_eq!(
        upload(&state, "", "a.txt", "text/plain", b"a", false)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let unavailable = AppState {
        media: state.media.clone(),
        ..AppState::default()
    };
    let session = unavailable
        .issue_session("a@example.com".into())
        .await
        .unwrap();
    assert_eq!(
        upload(
            &unavailable,
            &session.access_token,
            "a.txt",
            "text/plain",
            b"a",
            false
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    for (name, mime, data, repeat, status) in [
        (
            "bad.svg",
            "image/svg+xml",
            &b"<svg/>"[..],
            false,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "fake.png",
            "image/png",
            &b"<html>"[..],
            false,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "../a.txt",
            "text/plain",
            &b"a"[..],
            false,
            StatusCode::BAD_REQUEST,
        ),
        (
            "a.txt",
            "text/plain",
            &b"a"[..],
            true,
            StatusCode::BAD_REQUEST,
        ),
    ] {
        assert_eq!(upload(&state, &a, name, mime, data, repeat).await.0, status);
    }
    assert_eq!(
        upload(
            &state,
            &a,
            "big.txt",
            "text/plain",
            &vec![b'a'; media::MAX_FILE + 1],
            false
        )
        .await
        .0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert_eq!(
        upload(
            &state,
            &a,
            "huge.txt",
            "text/plain",
            &vec![b'a'; 7 * 1024 * 1024],
            false
        )
        .await
        .0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert!(store.objects.lock().unwrap().is_empty());
    let pg = state.db.as_ref().unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {}", pg.t("media_objects")))
            .fetch_one(&pg.pool)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn storage_failure_cleans_reservation_and_download_rejects_corruption_or_redirects() {
    let (state, a, _, _, store, _server) = fixture().await;
    store.mode.store(3, Ordering::SeqCst);
    assert_eq!(
        upload(&state, &a, "a.txt", "text/plain", b"test", false)
            .await
            .0,
        StatusCode::BAD_GATEWAY
    );
    let pg = state.db.as_ref().unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {}", pg.t("media_objects")))
            .fetch_one(&pg.pool)
            .await
            .unwrap(),
        0
    );
    // If registering success fails, never acknowledge an upload and clean only its own object.
    sqlx::query(&format!(
        "ALTER TABLE {} ADD CONSTRAINT fail_ready CHECK (NOT ready)",
        pg.t("media_objects")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    store.mode.store(0, Ordering::SeqCst);
    assert_eq!(
        upload(&state, &a, "failed.txt", "text/plain", b"test", false)
            .await
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert!(store.objects.lock().unwrap().is_empty());
    assert_eq!(
        sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM {}", pg.t("media_objects")))
            .fetch_one(&pg.pool)
            .await
            .unwrap(),
        0
    );
    sqlx::query(&format!(
        "ALTER TABLE {} DROP CONSTRAINT fail_ready",
        pg.t("media_objects")
    ))
    .execute(&pg.pool)
    .await
    .unwrap();
    store.mode.store(0, Ordering::SeqCst);
    let (status, file) = upload(&state, &a, "a.txt", "text/plain", b"test", false).await;
    assert_eq!(status, StatusCode::OK);
    for mode in [1, 2, 4] {
        store.mode.store(mode, Ordering::SeqCst);
        assert_eq!(
            get(&state, &a, file["url"].as_str().unwrap())
                .await
                .status(),
            StatusCode::BAD_GATEWAY
        );
    }
}

#[tokio::test]
#[ignore = "requires local private MinIO bucket; creates and removes only one generated test object"]
async fn real_minio_private_attachment_roundtrip() {
    use rusty_s3::S3Action;
    use std::{borrow::Cow, time::Duration};
    let (mut state, a, _, _, _, _server) = fixture().await;
    let config = media::MediaConfig::from_env().unwrap();
    assert!(config.ping().await, "local MinIO bucket required");
    let scan=config.scan_page("").await.expect("real MinIO list-v2 parsing and prefix scan");
    assert!(scan.contents.len()<=100);
    let bucket = rusty_s3::Bucket::new(
        url::Url::parse(&config.endpoint).unwrap(),
        rusty_s3::UrlStyle::Path,
        Cow::Owned(config.bucket.clone()),
        Cow::Borrowed("us-east-1"),
    )
    .unwrap();
    let credentials =
        rusty_s3::Credentials::new(config.access_key.clone(), config.secret_key.clone());
    state.media = Some(config);
    let bytes = "PromptArk 附件完整性测试\n只用于隔离验收".as_bytes();
    let (status, result) = upload(&state, &a, "往返验证.txt", "text/plain", bytes, false).await;
    assert_eq!(status, StatusCode::OK);
    let (_, retry) = upload(&state, &a, "往返验证.txt", "text/plain", bytes, false).await;
    let reference = json!({"id":uuid::Uuid::new_v4().to_string(),"media_id":result["id"],"name":result["name"],"mime":result["mime"],"size":result["size"],"sha256":result["sha256"]});
    let (sync_status, synced) = crate::admin_security_tests::request(&state, "PUT", "/v1/library/changes", &a,
        json!({"items":[{"id":"real-media-sync","kind":"prompt","payload":{"title":"隔离测试","asset_refs":[reference.clone()]},"updated_at":"1"}]})).await;
    let key = format!("promptark/{}", result["id"].as_str().unwrap());
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();
    let anonymous = client
        .get(bucket.object_url(&key).unwrap())
        .send()
        .await
        .unwrap()
        .status();
    let response = get(&state, &a, result["url"].as_str().unwrap()).await;
    let download_status = response.status();
    let downloaded = to_bytes(response.into_body(), media::MAX_FILE)
        .await
        .unwrap();
    let cleanup = bucket.delete_object(Some(&credentials), &key);
    assert!(client
        .delete(cleanup.sign(Duration::from_secs(60)))
        .send()
        .await
        .unwrap()
        .status()
        .is_success());
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!(
        "DELETE FROM {} WHERE id=$1",
        pg.t("media_objects")
    ))
    .bind(result["id"].as_str().unwrap())
    .execute(&pg.pool)
    .await
    .unwrap();
    assert_eq!(
        anonymous,
        StatusCode::FORBIDDEN,
        "bucket must not permit anonymous object access"
    );
    assert_eq!(download_status, StatusCode::OK);
    assert_eq!(downloaded.as_ref(), bytes);
    assert_eq!(retry, result);
    assert_eq!(sync_status, StatusCode::OK);
    assert_eq!(synced["items"][0]["payload"]["asset_refs"][0], reference);
}

#[tokio::test]
async fn concurrent_upload_reservations_enforce_account_quota_and_pending_is_unreadable() {
    let (state, a, _, _, _, _server) = fixture().await;
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!("INSERT INTO {} (id,owner_email,object_key,size) SELECT 'seed.'||n,'a@example.com','seed',0 FROM generate_series(1,255) n", pg.t("media_objects"))).execute(&pg.pool).await.unwrap();
    let file = |id: &str| media::MediaUpload {
        id: id.into(),
        url: String::new(),
        name: format!("{id}.txt"),
        mime: "text/plain".into(),
        size: 1,
        sha256: "a".repeat(64),
    };
    let x = file("x");
    let y = file("y");
    let (first, second) = tokio::join!(
        media::reserve(pg, "a@example.com", "x", &x),
        media::reserve(pg, "a@example.com", "y", &y)
    );
    assert_ne!(first.is_ok(), second.is_ok());
    let id = if first.is_ok() { "x" } else { "y" };
    assert_eq!(
        get(&state, &a, &format!("/v1/media/{id}/content"))
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&state, &a, "/v1/media/seed.1/url").await.status(),
        StatusCode::CONFLICT
    );
    // A separate account is governed by its own byte quota, including pending uploads.
    let mut large = file("large");
    large.size = 100 * 1024 * 1024;
    media::reserve(pg, "b@example.com", "large", &large)
        .await
        .unwrap();
    assert_eq!(
        media::reserve(pg, "b@example.com", "extra", &file("extra")).await,
        Err(StatusCode::CONFLICT)
    );
}

#[tokio::test]
async fn retry_reuses_only_owners_completed_object_and_private_refs_are_validated_atomically() {
    let (state, a, b, _, store, _server) = fixture().await;
    let (status, file) = upload(&state, &a, "notes.txt", "text/plain", b"private notes", false).await;
    assert_eq!(status, StatusCode::OK);
    let (_, retry) = upload(&state, &a, "notes.txt", "text/plain", b"private notes", false).await;
    assert_eq!(retry, file);
    assert_eq!(store.objects.lock().unwrap().len(), 1);
    let (_, other) = upload(&state, &b, "notes.txt", "text/plain", b"private notes", false).await;
    assert_ne!(other["id"], file["id"]);
    let reference = json!({ "id": uuid::Uuid::new_v4().to_string(), "media_id": file["id"], "name": file["name"], "mime": file["mime"], "size": file["size"], "sha256": file["sha256"] });
    let change = json!({"id":"prompt-a","kind":"prompt","payload":{"title":"私有","asset_refs":[reference.clone()]},"updated_at":"100"});
    let request = crate::admin_security_tests::request;
    assert_eq!(request(&state, "PUT", "/v1/library/changes", &a, json!({"items":[change.clone()]})).await.0, StatusCode::OK);
    assert_eq!(request(&state, "PUT", "/v1/library/changes", &b, json!({"items":[change.clone()]})).await.0, StatusCode::NOT_FOUND);
    let mut forged = change.clone(); forged["id"] = json!("forged"); forged["payload"]["asset_refs"][0]["sha256"] = json!("0".repeat(64));
    let clean = json!({"id":"must-rollback","kind":"prompt","payload":{"title":"no"},"updated_at":"101"});
    assert_eq!(request(&state, "PUT", "/v1/library/changes", &a, json!({"items":[clean,forged]})).await.0, StatusCode::NOT_FOUND);
    assert_eq!(state.list_library_changes("a@example.com", "").await.unwrap().len(), 1);
    let legacy = json!({"id":"prompt-a","kind":"prompt","payload":{"title":"正文更新"},"updated_at":"101"});
    let (status, result) = request(&state, "PUT", "/v1/library/changes", &a, json!({"items":[legacy]})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(result["items"][0]["payload"]["asset_refs"][0], reference);
    let mut oversized = change.clone(); oversized["payload"]["asset_refs"] = json!(vec![reference.clone(); 13]);
    assert_eq!(request(&state, "PUT", "/v1/library/changes", &a, json!({"items":[oversized]})).await.0, StatusCode::PAYLOAD_TOO_LARGE);
    let mut duplicate = change.clone(); duplicate["payload"]["asset_refs"] = json!([reference.clone(), reference.clone()]);
    assert_eq!(request(&state, "PUT", "/v1/library/changes", &a, json!({"items":[duplicate]})).await.0, StatusCode::BAD_REQUEST);
    let pg = state.db.as_ref().unwrap();
    sqlx::query(&format!("UPDATE {} SET ready=FALSE WHERE id=$1", pg.t("media_objects"))).bind(file["id"].as_str().unwrap()).execute(&pg.pool).await.unwrap();
    assert_eq!(upload(&state, &a, "notes.txt", "text/plain", b"private notes", false).await.0, StatusCode::CONFLICT);
    assert_eq!(request(&state, "PUT", "/v1/library/changes", &a, json!({"items":[change]})).await.0, StatusCode::NOT_FOUND);
    assert_eq!(store.objects.lock().unwrap().len(), 2);
}
