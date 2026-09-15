use axum::{extract::{Query, State}, http::{HeaderMap, StatusCode}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use crate::recommendation::Candidate;
use sqlx::Row;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Browse {
    sort: Option<String>, q: Option<String>, category_id: Option<String>, model: Option<String>,
    limit: Option<i64>, offset: Option<i64>, content_language: Option<String>, recommendation: Option<String>, exclude: Option<String>,
}

pub async fn browse(State(state): State<AppState>, headers: HeaderMap, Query(input): Query<Browse>) -> Result<Json<Value>, StatusCode> {
    let translated = match input.content_language.as_deref().unwrap_or("zh") {"zh"=>true,"original"=>false,_=>return Err(StatusCode::BAD_REQUEST)};
    let limit = input.limit.unwrap_or(48);
    let offset = input.offset.unwrap_or(0);
    let query = input.q.unwrap_or_default().trim().to_owned();
    let sort = match input.sort.as_deref().unwrap_or("recommended") {
        "recommended" | "推荐" => "recommended", "latest" | "最新" => "latest",
        "hot" | "热门" => "hot", "favorites" | "收藏" => "favorites", _ => return Err(StatusCode::BAD_REQUEST),
    };
    if !(1..=48).contains(&limit) || !(0..=100_000).contains(&offset) || query.len() > 1200
        || input.model.as_ref().is_some_and(|s| s.len() > 200)
        || input.category_id.as_ref().is_some_and(|s| s.len() > 200) { return Err(StatusCode::BAD_REQUEST); }
    let seed = input.recommendation.unwrap_or_else(|| format!("hour-{}", chrono::Utc::now().timestamp() / 3600));
    if seed.is_empty() || seed.len() > 64 || !seed.bytes().all(|c|c.is_ascii_alphanumeric() || c==b'-') { return Err(StatusCode::BAD_REQUEST); }
    let mut excluded: Vec<String> = match input.exclude {
        Some(raw) if raw.len() <= 12000 => serde_json::from_str(&raw).map_err(|_|StatusCode::BAD_REQUEST)?,
        Some(_) => return Err(StatusCode::BAD_REQUEST), None => vec![],
    };
    if excluded.len() > 256 || excluded.iter().any(|id|id.is_empty() || id.len()>200) { return Err(StatusCode::BAD_REQUEST); }
    if sort != "recommended" { excluded.clear(); }
    excluded.sort(); excluded.dedup();
    let mut recommendation_next = None;
    let email = crate::optional_access_email(&state, &headers).await;
    if !state.square_public().await? || sort == "favorites" { crate::require_user(&state, &headers).await?; }
    let model = input.model.filter(|s| !s.is_empty());
    let category = input.category_id.filter(|s| !s.is_empty());
    let categories = if category.is_some() {
        if let Some(pg) = &state.db { pg.catalog_entries("categories", false).await? } else { crate::admin_catalog::seed_categories() }
    } else { vec![] };
    let ids: Vec<String> = category.iter().cloned().chain(categories.iter().filter(|c| c.parent_id.as_ref() == category.as_ref()).map(|c| c.id.clone())).collect();
    let key = serde_json::to_string(&(&seed, &query, &model, &ids, translated, &excluded)).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
    let cached = if sort == "recommended" { state.recommendations.get(&key)? } else { None };
    if sort == "recommended" && offset > 0 && cached.is_none() { return Err(StatusCode::CONFLICT); }
    let (total, mut items, category_counts): (i64, Vec<Value>, Option<serde_json::Map<String, Value>>) = if let Some(pg) = &state.db {
        let pattern = (!query.is_empty()).then(|| format!("%{}%", query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")));
        let favorite = format!("EXISTS(SELECT 1 FROM {} f WHERE f.email=$4 AND f.item_id=s.id)", pg.t("favorites"));
        let translated_join = format!("LEFT JOIN {} tr ON tr.item_id=s.id AND tr.target='zh' AND tr.status='ready'", pg.t("prompt_translations"));
        let title = if translated {"COALESCE(tr.data->>'title',s.title)"} else {"s.title"};
        let excerpt = if translated {"COALESCE(tr.data->>'excerpt',s.excerpt)"} else {"s.excerpt"};
        let filter = format!(r#"s.visibility='online'
            AND ($1::text IS NULL OR s.title ILIKE $1 ESCAPE '\' OR s.excerpt ILIKE $1 ESCAPE '\' OR s.reference->>'author' ILIKE $1 ESCAPE '\' OR tr.data->>'title' ILIKE $1 ESCAPE '\' OR tr.data->>'excerpt' ILIKE $1 ESCAPE '\')
            AND ($2::text IS NULL OR s.model=$2) AND ($3::text[]='{{}}' OR s.category_id=ANY($3))
            AND (NOT $5::boolean OR {favorite}) AND NOT(s.id=ANY($9::text[]))"#);
        let order = match sort {
            "latest" => "s.listed_at DESC NULLS LAST,s.id",
            "hot" => "s.download_count DESC,s.title,s.id",
            "recommended" => "array_position($8::text[],s.id)",
            _ => "s.recommended DESC,s.sort_index,s.id",
        };
        let count_sql = format!("SELECT count(*) FROM {} s {translated_join} WHERE {filter}", pg.t("square_items")).replace("$9", "$6");
        // Never select content/members, or return the complete reference document.
        let page_sql = format!("SELECT jsonb_build_object('id',s.id,'title',left({title},160),'kind',s.kind,
            'excerpt',left({excerpt},240),'content_language',CASE WHEN tr.status='ready' THEN 'zh' ELSE 'original' END,'model',s.model,'category_id',s.category_id,'member_count',s.member_count,
            'is_favorite',{favorite},'download_count',s.download_count,
            'preview_asset',(SELECT a FROM jsonb_array_elements(COALESCE(p.asset_refs,'[]'::jsonb)) a WHERE a->>'mime' IN ('image/png','image/jpeg','image/gif','image/webp') LIMIT 1),
            'image_count',(SELECT count(*) FROM jsonb_array_elements(COALESCE(p.asset_refs,'[]'::jsonb)) a WHERE a->>'mime' IN ('image/png','image/jpeg','image/gif','image/webp')),
            'asset_count',jsonb_array_length(COALESCE(p.asset_refs,'[]'::jsonb)),
            'favorite_count',(SELECT count(*) FROM {} f WHERE f.item_id=s.id),'reference',CASE WHEN jsonb_typeof(s.reference->'images'->0)='string'
            THEN jsonb_build_object('images',jsonb_build_array(left(s.reference->'images'->>0,2048))) ELSE NULL END)
            FROM {} s {translated_join} LEFT JOIN {} p ON p.id=s.id AND p.status='approved' WHERE {filter} AND ($8::text[] IS NULL OR s.id=ANY($8)) ORDER BY {order} LIMIT $6 OFFSET $7", pg.t("favorites"), pg.t("square_items"), pg.t("publications"));
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let mut tx = pg.pool.begin().await?;
            sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY").execute(&mut *tx).await?;
            let snapshot = if sort == "recommended" {
                if let Some(snapshot) = cached.clone() { Some(snapshot) } else {
                    let sql = format!("SELECT s.id,s.recommended,s.download_count,EXTRACT(EPOCH FROM s.listed_at)::float8 AS listed FROM {} s {translated_join} WHERE {filter} LIMIT 100001",pg.t("square_items")).replace("$9", "$6");
                    let rows = sqlx::query(&sql).bind(&pattern).bind(&model).bind(&ids).bind(&email).bind(false).bind(&excluded).fetch_all(&mut *tx).await?;
                    let rows = rows.into_iter().map(|row|Candidate { id:row.get("id"),featured:row.get("recommended"),downloads:row.get("download_count"),listed:row.get("listed") }).collect();
                    Some(state.recommendations.insert(key.clone(),rows,&seed).map_err(|_|sqlx::Error::Protocol("recommendation capacity".into()))?)
                }
            } else { None };
            let total = if let Some(snapshot) = &snapshot { snapshot.len() as i64 } else {
                sqlx::query_scalar(&count_sql).bind(&pattern).bind(&model).bind(&ids).bind(&email).bind(sort == "favorites").bind(&excluded).fetch_one(&mut *tx).await?
            };
            let page_ids = snapshot.as_ref().map(|rows|rows.iter().skip(offset as usize).take(limit as usize).cloned().collect::<Vec<_>>());
            if snapshot.is_some() && offset+limit < total { recommendation_next = Some(offset+limit); }
            let items: Vec<Value> = sqlx::query_scalar(&page_sql).bind(&pattern).bind(&model).bind(&ids).bind(&email).bind(sort == "favorites").bind(if snapshot.is_some() {limit} else {limit+1}).bind(if snapshot.is_some() {0} else {offset}).bind(page_ids).bind(&excluded).fetch_all(&mut *tx).await?;
            let category_counts = if offset == 0 {
                let counts: Vec<(String, i64)> = sqlx::query_as(&format!("SELECT COALESCE(category_id,''),count(*) FROM {} WHERE visibility='online' GROUP BY COALESCE(category_id,'')", pg.t("square_items"))).fetch_all(&mut *tx).await?;
                Some(counts.into_iter().map(|(id,count)| (id,json!(count))).collect())
            } else { None };
            tx.commit().await?;
            Ok::<_, sqlx::Error>((total, items, category_counts))
        }).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    } else {
        let favorites = if let Some(email) = &email { state.favorite_ids(email).await? } else { vec![] };
        let needle = query.to_lowercase();
        let mut rows = state.all_items().await?;
        let category_counts = if offset == 0 {
            let mut counts = serde_json::Map::new();
            for row in &rows {
                let entry = counts.entry(row.category_id.clone().unwrap_or_default()).or_insert(json!(0));
                *entry = json!(entry.as_i64().unwrap_or(0) + 1);
            }
            Some(counts)
        } else { None };
        rows.retain(|p| (needle.is_empty() || p.title.to_lowercase().contains(&needle) || p.excerpt.as_deref().unwrap_or("").to_lowercase().contains(&needle)
                || p.reference.as_ref().and_then(|r|r["author"].as_str()).unwrap_or("").to_lowercase().contains(&needle))
            && model.as_ref().is_none_or(|m|p.model.as_ref()==Some(m))
            && (ids.is_empty() || p.category_id.as_ref().is_some_and(|id|ids.contains(id)))
            && (sort != "favorites" || favorites.contains(&p.id)) && !excluded.contains(&p.id));
        let counts = state.download_counts().await?;
        let favorite_counts = state.memory_favorite_counts()?;
        rows.sort_by(|a,b| match sort {
            "hot" => counts.get(&b.id).unwrap_or(&0).cmp(counts.get(&a.id).unwrap_or(&0)).then(a.title.cmp(&b.title)).then(a.id.cmp(&b.id)),
            "latest" => b.id.cmp(&a.id), _ => a.id.cmp(&b.id),
        });
        let total;
        if sort == "recommended" {
            let snapshot = match cached { Some(ids)=>ids, None=>state.recommendations.insert(key,rows.iter().map(|p|Candidate {id:p.id.clone(),featured:false,downloads:*counts.get(&p.id).unwrap_or(&0),listed:None}).collect(),&seed)? };
            total=snapshot.len() as i64;
            if offset+limit < total {recommendation_next=Some(offset+limit);}
            let mut by_id: std::collections::HashMap<_,_> = rows.into_iter().map(|p|(p.id.clone(),p)).collect();
            rows=snapshot.iter().skip(offset as usize).take(limit as usize).filter_map(|id|by_id.remove(id)).collect();
        } else {total=rows.len() as i64;rows=rows.into_iter().skip(offset as usize).take(limit as usize+1).collect();}
        (total, rows.into_iter().map(|p| {
            let image = p.reference.as_ref().and_then(|r|r["images"][0].as_str()).map(|s|s.chars().take(2048).collect::<String>());
            json!({"id":p.id,"title":p.title.chars().take(160).collect::<String>(),"kind":p.kind,"excerpt":p.excerpt.map(|s|s.chars().take(240).collect::<String>()),"model":p.model,"category_id":p.category_id,"member_count":p.member_count,"is_favorite":favorites.contains(&p.id),"download_count":counts.get(&p.id).copied().unwrap_or(0),"favorite_count":favorite_counts.get(&p.id).copied().unwrap_or(0),"reference":image.map(|url|json!({"images":[url]}))})
        }).collect(), category_counts)
    };
    let more = items.len() > limit as usize;
    items.truncate(limit as usize);
    if !state.square_public().await? || sort == "favorites" { crate::require_user(&state, &headers).await?; }
    let category_total = category_counts.as_ref().map(|counts| counts.values().filter_map(Value::as_i64).sum::<i64>());
    Ok(Json(json!({"items":items,"total":total,"category_counts":category_counts,"category_total":category_total,"limit":limit,"offset":offset,"recommendation":if sort=="recommended" {Some(seed)} else {None},"next_offset":if sort=="recommended" {recommendation_next} else if more && offset+limit <= 100_000 {Some(offset+limit)} else {None}})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_security_tests::{state, request};
    #[tokio::test]
    async fn recommendation_twenty_thousand_candidates_stays_bounded() {
        let state=state().await;let pg=state.db.as_ref().unwrap();
        sqlx::query(&format!("INSERT INTO {} (id,title,kind,excerpt,content,visibility,download_count) SELECT 'bench-'||i,'bench-'||i,'prompt','bench-ranking',repeat('body',100),'online',i FROM generate_series(1,20000) i",pg.t("square_items"))).execute(&pg.pool).await.unwrap();
        let start=std::time::Instant::now();
        let (status,first)=request(&state,"GET","/v1/square/browse?q=bench-ranking&recommendation=bench","",json!(null)).await;
        let cold=start.elapsed();let start=std::time::Instant::now();
        let (second_status,second)=request(&state,"GET","/v1/square/browse?q=bench-ranking&recommendation=bench&offset=48","",json!(null)).await;
        println!("20k recommendation cold={cold:?}, continuation={:?}, bytes={}",start.elapsed(),first.to_string().len());
        assert_eq!(status,StatusCode::OK);assert_eq!(second_status,StatusCode::OK);
        assert_eq!(first["total"],20000);assert_eq!(first["items"].as_array().unwrap().len(),48);assert_eq!(second["items"].as_array().unwrap().len(),48);
        assert!(first.to_string().len()<100_000);assert!(!first.to_string().contains("bodybody"));
        sqlx::query(&format!("DROP SCHEMA {} CASCADE",pg.schema)).execute(&pg.pool).await.unwrap();
    }
    #[tokio::test]
    async fn recommendation_pages_survive_metrics_new_items_and_removals() {
        let state=state().await; let pg=state.db.as_ref().unwrap();
        for i in 0..120 {
            sqlx::query(&format!("INSERT INTO {} (id,title,kind,content,excerpt,visibility,download_count,listed_at) VALUES($1,$1,'prompt','body','rank-fixture','online',$2,now())",pg.t("square_items")))
                .bind(format!("rank-{i:03}")).bind(i as i64).execute(&pg.pool).await.unwrap();
        }
        let path="/v1/square/browse?q=rank-fixture&recommendation=test-a";
        let (code, first)=request(&state,"GET",path,"",json!(null)).await;
        assert_eq!(code,StatusCode::OK);assert_eq!(first["recommendation"],"test-a");assert_eq!(first["total"],120);
        let excluded_path="/v1/square/browse?q=rank-fixture&recommendation=exclude&exclude=%5B%22rank-000%22%2C%22rank-001%22%5D";
        let (excluded_code, excluded)=request(&state,"GET",excluded_path,"",json!(null)).await;
        assert_eq!(excluded_code,StatusCode::OK); assert_eq!(excluded["total"],118);
        assert_eq!(excluded["category_counts"],first["category_counts"]);
        for offset in [0,48,96] {
            let (_, page)=request(&state,"GET",&format!("{excluded_path}&offset={offset}"),"",json!(null)).await;
            assert!(page["items"].as_array().unwrap().iter().all(|item|item["id"]!="rank-000"&&item["id"]!="rank-001"));
        }
        assert_eq!(request(&state,"GET","/v1/square/browse?exclude=bad","",json!(null)).await.0,StatusCode::BAD_REQUEST);
        let (_, second)=request(&state,"GET",&format!("{path}&offset=48"),"",json!(null)).await;
        let ids=|value:&Value|value["items"].as_array().unwrap().iter().map(|i|i["id"].as_str().unwrap().to_string()).collect::<Vec<_>>();
        let first_ids=ids(&first);let second_ids=ids(&second);
        assert!(first_ids.iter().all(|id|!second_ids.contains(id)));
        sqlx::query(&format!("UPDATE {} SET download_count=1000000 WHERE id=$1",pg.t("square_items"))).bind(&second_ids[1]).execute(&pg.pool).await.unwrap();
        sqlx::query(&format!("UPDATE {} SET visibility='offline' WHERE id=$1 OR id=$2",pg.t("square_items"))).bind(&first_ids[0]).bind(&second_ids[0]).execute(&pg.pool).await.unwrap();
        sqlx::query(&format!("INSERT INTO {} (id,title,kind,excerpt,visibility) VALUES('rank-new','new','prompt','rank-fixture','online')",pg.t("square_items"))).execute(&pg.pool).await.unwrap();
        let (_, again)=request(&state,"GET",&format!("{path}&offset=48"),"",json!(null)).await;
        assert_eq!(ids(&again),second_ids[1..]);assert_eq!(again["total"],120);assert_eq!(again["next_offset"],96);
        let (_, rotated)=request(&state,"GET","/v1/square/browse?q=rank-fixture&recommendation=test-b","",json!(null)).await;
        assert_ne!(ids(&rotated),first_ids);assert_eq!(rotated["total"],119);
        assert_eq!(request(&state,"GET","/v1/square/browse?q=rank-fixture&recommendation=missing&offset=48","",json!(null)).await.0,StatusCode::CONFLICT);
        assert_eq!(request(&state,"GET","/v1/square/browse?recommendation=%27","",json!(null)).await.0,StatusCode::BAD_REQUEST);
        // Even a completely removed slice must advance using snapshot positions.
        sqlx::query(&format!("UPDATE {} SET visibility='offline' WHERE id=ANY($1)",pg.t("square_items"))).bind(&second_ids).execute(&pg.pool).await.unwrap();
        let (_, empty)=request(&state,"GET",&format!("{path}&offset=48"),"",json!(null)).await;
        assert_eq!(empty["items"],json!([]));assert_eq!(empty["next_offset"],96);
        sqlx::query(&format!("DROP SCHEMA {} CASCADE",pg.schema)).execute(&pg.pool).await.unwrap();
    }
    #[tokio::test]
    async fn bounded_browse_filters_counts_and_omits_heavy_fields() {
        let state = state().await; let pg = state.db.as_ref().unwrap();
        for i in 0..110 {
            sqlx::query(&format!("INSERT INTO {} (id,title,kind,excerpt,content,category_id,model,visibility,reference,download_count) VALUES($1,'QA same','prompt','100% literal_',$2,'cat-image-0','Flux',$3,$4,$5)",pg.t("square_items")))
                .bind(format!("qa-{i:03}")).bind("large body ".repeat(10000)).bind(if i==109 {"offline"} else {"online"})
                .bind(json!({"author":"QA author","images":["https://cms-assets.youmind.com/a.jpg","https://cms-assets.youmind.com/b.jpg"],"large":"x".repeat(20000)})).bind(i as i64).execute(&pg.pool).await.unwrap();
        }
        let path = "/v1/square/browse?q=100%25&category_id=cat-image&model=Flux&recommendation=browse-test";
        let (code, first) = request(&state,"GET",path,"",json!(null)).await;
        assert_eq!(code,StatusCode::OK); assert_eq!(first["total"],109); assert_eq!(first["items"].as_array().unwrap().len(),48); assert_eq!(first["next_offset"],48);
        assert_eq!(first["category_counts"]["cat-image-0"],109);
        assert!(first["category_total"].as_i64().unwrap() >= 109);
        assert!(first.to_string().len()<250_000); assert!(!first.to_string().contains("large body"));
        assert_eq!(first["items"][0]["reference"]["images"].as_array().unwrap().len(),1);
        let (_, second) = request(&state,"GET",&format!("{path}&offset=48"),"",json!(null)).await;
        assert_ne!(first["items"][0]["id"],second["items"][0]["id"]);
        assert!(second["category_counts"].is_null());
        let (_, filtered) = request(&state,"GET","/v1/square/browse?q=no-such-prompt&model=unknown","",json!(null)).await;
        assert_eq!(filtered["total"],0);
        assert_eq!(filtered["category_counts"],first["category_counts"]);
        let (_, last) = request(&state,"GET",&format!("{path}&offset=96"),"",json!(null)).await;
        assert_eq!(last["items"].as_array().unwrap().len(),13); assert!(last["next_offset"].is_null());
        let (_, empty) = request(&state,"GET",&format!("{path}&offset=110"),"",json!(null)).await;
        assert_eq!(empty["total"],109); assert_eq!(empty["items"],json!([]));
        let (_, hot) = request(&state,"GET",&format!("{path}&sort=hot"),"",json!(null)).await;
        assert_eq!(hot["items"][0]["id"],"qa-108");
        assert_eq!(hot["items"][0]["download_count"],108);
        assert_eq!(hot["items"][0]["favorite_count"],0);
        sqlx::query(&format!("UPDATE {} SET listed_at=CASE WHEN id IN ('qa-002','qa-003') THEN TIMESTAMPTZ '2030-01-01' ELSE NULL END",pg.t("square_items"))).execute(&pg.pool).await.unwrap();
        let (_, latest) = request(&state,"GET",&format!("{path}&sort=latest"),"",json!(null)).await;
        assert_eq!(latest["items"][0]["id"],"qa-002"); assert_eq!(latest["items"][1]["id"],"qa-003");
        assert!(latest["recommendation"].is_null());

        for query in ["limit=0","limit=49","offset=-1","offset=100001","sort=bad","extra=x"] {
            assert_eq!(request(&state,"GET",&format!("/v1/square/browse?{query}"),"",json!(null)).await.0,StatusCode::BAD_REQUEST);
        }
        assert_eq!(request(&state,"GET","/v1/square/browse?sort=favorites","",json!(null)).await.0,StatusCode::UNAUTHORIZED);
        pg.upsert_account("browse@example.com", Some("test-password"), "user").await.unwrap();
        pg.upsert_account("other@example.com", Some("test-password"), "user").await.unwrap();
        let session = state.issue_session("browse@example.com".into()).await.unwrap();
        let other = state.issue_session("other@example.com".into()).await.unwrap();
        assert_eq!(request(&state,"PUT","/v1/favorites/qa-003",&session.access_token,json!(null)).await.0,StatusCode::OK);
        let (_, saved) = request(&state,"GET","/v1/square/browse?sort=favorites",&session.access_token,json!(null)).await;
        assert_eq!(saved["total"],1); assert_eq!(saved["items"][0]["id"],"qa-003"); assert_eq!(saved["items"][0]["is_favorite"],true);
        assert_eq!(saved["items"][0]["favorite_count"],1);
        request(&state,"PUT","/v1/favorites/qa-003",&session.access_token,json!(null)).await;
        request(&state,"PUT","/v1/favorites/qa-003",&other.access_token,json!(null)).await;
        let (_, counted) = request(&state,"GET","/v1/square/browse?sort=favorites",&session.access_token,json!(null)).await;
        assert_eq!(counted["items"][0]["favorite_count"],2);
        request(&state,"DELETE","/v1/favorites/qa-003",&other.access_token,json!(null)).await;
        let (_, uncounted) = request(&state,"GET","/v1/square/browse?sort=favorites",&session.access_token,json!(null)).await;
        assert_eq!(uncounted["items"][0]["favorite_count"],1);
        let (_, isolated) = request(&state,"GET","/v1/square/browse?sort=favorites",&other.access_token,json!(null)).await;
        assert_eq!(isolated["total"],0);
        assert_eq!(isolated["category_counts"],first["category_counts"]);
        sqlx::query(&format!("UPDATE {} SET visibility='offline'",pg.t("square_items"))).execute(&pg.pool).await.unwrap();
        let (_, empty_counts) = request(&state,"GET","/v1/square/browse","",json!(null)).await;
        assert_eq!(empty_counts["category_total"],0); assert_eq!(empty_counts["category_counts"],json!({}));
        state.set_square_public(false).await.unwrap();
        assert_eq!(request(&state,"GET",path,"",json!(null)).await.0,StatusCode::UNAUTHORIZED);
        sqlx::query(&format!("DROP SCHEMA {} CASCADE",pg.schema)).execute(&pg.pool).await.unwrap();
    }
}
