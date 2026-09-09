use axum::{extract::{Query, State}, http::{HeaderMap, StatusCode}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Browse {
    sort: Option<String>, q: Option<String>, category_id: Option<String>, model: Option<String>,
    limit: Option<i64>, offset: Option<i64>,
}

pub async fn browse(State(state): State<AppState>, headers: HeaderMap, Query(input): Query<Browse>) -> Result<Json<Value>, StatusCode> {
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
    let email = crate::optional_access_email(&state, &headers).await;
    if !state.square_public().await? || sort == "favorites" { crate::require_user(&state, &headers).await?; }
    let model = input.model.filter(|s| !s.is_empty());
    let category = input.category_id.filter(|s| !s.is_empty());
    let categories = if category.is_some() {
        if let Some(pg) = &state.db { pg.catalog_entries("categories", false).await? } else { crate::admin_catalog::seed_categories() }
    } else { vec![] };
    let ids: Vec<String> = category.iter().cloned().chain(categories.iter().filter(|c| c.parent_id.as_ref() == category.as_ref()).map(|c| c.id.clone())).collect();
    let (total, mut items): (i64, Vec<Value>) = if let Some(pg) = &state.db {
        let pattern = (!query.is_empty()).then(|| format!("%{}%", query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")));
        let favorite = format!("EXISTS(SELECT 1 FROM {} f WHERE f.email=$4 AND f.item_id=s.id)", pg.t("favorites"));
        let filter = format!(r#"s.visibility='online'
            AND ($1::text IS NULL OR s.title ILIKE $1 ESCAPE '\' OR s.excerpt ILIKE $1 ESCAPE '\' OR s.reference->>'author' ILIKE $1 ESCAPE '\')
            AND ($2::text IS NULL OR s.model=$2) AND ($3::text[]='{{}}' OR s.category_id=ANY($3))
            AND (NOT $5::boolean OR {favorite})"#);
        let order = match sort {
            "latest" => "s.listed_at DESC NULLS LAST,s.id",
            "hot" => "s.download_count DESC,s.title,s.id",
            _ => "s.recommended DESC,s.sort_index,s.id",
        };
        let count_sql = format!("SELECT count(*) FROM {} s WHERE {filter}", pg.t("square_items"));
        // Never select content/members, or return the complete reference document.
        let page_sql = format!("SELECT jsonb_build_object('id',s.id,'title',left(s.title,160),'kind',s.kind,
            'excerpt',left(s.excerpt,240),'model',s.model,'category_id',s.category_id,'member_count',s.member_count,
            'is_favorite',{favorite},'reference',CASE WHEN jsonb_typeof(s.reference->'images'->0)='string'
            THEN jsonb_build_object('images',jsonb_build_array(left(s.reference->'images'->>0,2048))) ELSE NULL END)
            FROM {} s WHERE {filter} ORDER BY {order} LIMIT $6 OFFSET $7", pg.t("square_items"));
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let mut tx = pg.pool.begin().await?;
            sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY").execute(&mut *tx).await?;
            let total: i64 = sqlx::query_scalar(&count_sql).bind(&pattern).bind(&model).bind(&ids).bind(&email).bind(sort == "favorites").fetch_one(&mut *tx).await?;
            let items: Vec<Value> = sqlx::query_scalar(&page_sql).bind(&pattern).bind(&model).bind(&ids).bind(&email).bind(sort == "favorites").bind(limit + 1).bind(offset).fetch_all(&mut *tx).await?;
            tx.commit().await?;
            Ok::<_, sqlx::Error>((total, items))
        }).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    } else {
        let favorites = if let Some(email) = &email { state.favorite_ids(email).await? } else { vec![] };
        let needle = query.to_lowercase();
        let mut rows = state.all_items().await?;
        rows.retain(|p| (needle.is_empty() || p.title.to_lowercase().contains(&needle) || p.excerpt.as_deref().unwrap_or("").to_lowercase().contains(&needle)
                || p.reference.as_ref().and_then(|r|r["author"].as_str()).unwrap_or("").to_lowercase().contains(&needle))
            && model.as_ref().is_none_or(|m|p.model.as_ref()==Some(m))
            && (ids.is_empty() || p.category_id.as_ref().is_some_and(|id|ids.contains(id)))
            && (sort != "favorites" || favorites.contains(&p.id)));
        let counts = state.download_counts().await?;
        rows.sort_by(|a,b| match sort {
            "hot" => counts.get(&b.id).unwrap_or(&0).cmp(counts.get(&a.id).unwrap_or(&0)).then(a.title.cmp(&b.title)).then(a.id.cmp(&b.id)),
            "latest" => b.id.cmp(&a.id), _ => a.id.cmp(&b.id),
        });
        let total = rows.len() as i64;
        (total, rows.into_iter().skip(offset as usize).take(limit as usize+1).map(|p| {
            let image = p.reference.as_ref().and_then(|r|r["images"][0].as_str()).map(|s|s.chars().take(2048).collect::<String>());
            json!({"id":p.id,"title":p.title.chars().take(160).collect::<String>(),"kind":p.kind,"excerpt":p.excerpt.map(|s|s.chars().take(240).collect::<String>()),"model":p.model,"category_id":p.category_id,"member_count":p.member_count,"is_favorite":favorites.contains(&p.id),"reference":image.map(|url|json!({"images":[url]}))})
        }).collect())
    };
    let more = items.len() > limit as usize;
    items.truncate(limit as usize);
    if !state.square_public().await? || sort == "favorites" { crate::require_user(&state, &headers).await?; }
    Ok(Json(json!({"items":items,"total":total,"limit":limit,"offset":offset,"next_offset":if more && offset+limit <= 100_000 {Some(offset+limit)} else {None}})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_security_tests::{state, request};
    #[tokio::test]
    async fn bounded_browse_filters_counts_and_omits_heavy_fields() {
        let state = state().await; let pg = state.db.as_ref().unwrap();
        for i in 0..110 {
            sqlx::query(&format!("INSERT INTO {} (id,title,kind,excerpt,content,category_id,model,visibility,reference,download_count) VALUES($1,'QA same','prompt','100% literal_',$2,'cat-image-0','Flux',$3,$4,$5)",pg.t("square_items")))
                .bind(format!("qa-{i:03}")).bind("large body ".repeat(10000)).bind(if i==109 {"offline"} else {"online"})
                .bind(json!({"author":"QA author","images":["https://cms-assets.youmind.com/a.jpg","https://cms-assets.youmind.com/b.jpg"],"large":"x".repeat(20000)})).bind(i as i64).execute(&pg.pool).await.unwrap();
        }
        let path = "/v1/square/browse?q=100%25&category_id=cat-image&model=Flux";
        let (code, first) = request(&state,"GET",path,"",json!(null)).await;
        assert_eq!(code,StatusCode::OK); assert_eq!(first["total"],109); assert_eq!(first["items"].as_array().unwrap().len(),48); assert_eq!(first["next_offset"],48);
        assert!(first.to_string().len()<250_000); assert!(!first.to_string().contains("large body"));
        assert_eq!(first["items"][0]["reference"]["images"].as_array().unwrap().len(),1);
        let (_, second) = request(&state,"GET",&format!("{path}&offset=48"),"",json!(null)).await;
        assert_ne!(first["items"][0]["id"],second["items"][0]["id"]);
        let (_, last) = request(&state,"GET",&format!("{path}&offset=96"),"",json!(null)).await;
        assert_eq!(last["items"].as_array().unwrap().len(),13); assert!(last["next_offset"].is_null());
        let (_, empty) = request(&state,"GET",&format!("{path}&offset=110"),"",json!(null)).await;
        assert_eq!(empty["total"],109); assert_eq!(empty["items"],json!([]));
        let (_, hot) = request(&state,"GET",&format!("{path}&sort=hot"),"",json!(null)).await;
        assert_eq!(hot["items"][0]["id"],"qa-108");
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
        let (_, isolated) = request(&state,"GET","/v1/square/browse?sort=favorites",&other.access_token,json!(null)).await;
        assert_eq!(isolated["total"],0);
        state.set_square_public(false).await.unwrap();
        assert_eq!(request(&state,"GET",path,"",json!(null)).await.0,StatusCode::UNAUTHORIZED);
        sqlx::query(&format!("DROP SCHEMA {} CASCADE",pg.schema)).execute(&pg.pool).await.unwrap();
    }
}
