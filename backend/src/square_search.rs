use axum::{extract::{Query, State}, http::{HeaderMap, StatusCode}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Search {
    q: Option<String>, category_id: Option<String>, model: Option<String>,
    limit: Option<i64>, offset: Option<i64>,
}

pub async fn search(State(state): State<AppState>, headers: HeaderMap, Query(input): Query<Search>) -> Result<Json<Value>, StatusCode> {
    let limit = input.limit.unwrap_or(20);
    let offset = input.offset.unwrap_or(0);
    let query = input.q.unwrap_or_default().trim().to_owned();
    if !(1..=100).contains(&limit) || !(0..=100_000).contains(&offset) || query.len() > 1200
        || input.model.as_ref().is_some_and(|s| s.len() > 200)
        || input.category_id.as_ref().is_some_and(|s| s.len() > 200) { return Err(StatusCode::BAD_REQUEST); }
    if !state.square_public().await? { crate::require_user(&state, &headers).await?; }
    let model = input.model.filter(|s| !s.is_empty());
    let category = input.category_id.filter(|s| !s.is_empty());
    let categories = if category.is_some() {
        if let Some(pg) = &state.db { pg.catalog_entries("categories", false).await? } else { crate::admin_catalog::seed_categories() }
    } else { vec![] };
    let ids: Vec<String> = category.iter().cloned().chain(categories.iter().filter(|c| c.parent_id.as_ref() == category.as_ref()).map(|c| c.id.clone())).collect();
    let mut items: Vec<Value> = if let Some(pg) = &state.db {
        let pattern = format!("%{}%", query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
        let sql = format!(r#"SELECT jsonb_build_object('id',id,'title',title,'kind',kind,'excerpt',excerpt,'category_id',category_id,'model',model) FROM {}
            WHERE visibility='online' AND (title ILIKE $1 ESCAPE '\' OR excerpt ILIKE $1 ESCAPE '\')
            AND ($2::text IS NULL OR model=$2) AND ($3::text[]='{{}}' OR category_id=ANY($3))
            ORDER BY title,id LIMIT $4 OFFSET $5"#, pg.t("square_items"));
        tokio::time::timeout(std::time::Duration::from_secs(3), sqlx::query_scalar(&sql).bind(pattern).bind(&model).bind(&ids).bind(limit+1).bind(offset).fetch_all(&pg.pool))
            .await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    } else {
        let needle = query.to_lowercase();
        let mut rows = state.all_items().await?;
        rows.retain(|item| (item.title.to_lowercase().contains(&needle) || item.excerpt.as_deref().unwrap_or("").to_lowercase().contains(&needle))
            && model.as_ref().is_none_or(|m| item.model.as_ref() == Some(m))
            && (ids.is_empty() || item.category_id.as_ref().is_some_and(|id| ids.contains(id))));
        rows.sort_by(|a,b| a.title.cmp(&b.title).then(a.id.cmp(&b.id)));
        rows.into_iter().skip(offset as usize).take(limit as usize+1).map(|p| json!({"id":p.id,"title":p.title,"kind":p.kind,"excerpt":p.excerpt,"category_id":p.category_id,"model":p.model})).collect()
    };
    let more = items.len() > limit as usize; items.truncate(limit as usize);
    if !state.square_public().await? { crate::require_user(&state, &headers).await?; }
    Ok(Json(json!({"items":items,"limit":limit,"offset":offset,"next_offset":if more && offset+limit <= 100_000 {Some(offset+limit)} else {None}})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin_security_tests::{state, request};
    #[tokio::test]
    async fn paged_public_search_filters_and_hides_unavailable_content() {
        let state = state().await; let pg = state.db.as_ref().unwrap();
        for i in 0..5 {
            sqlx::query(&format!("INSERT INTO {} (id,title,kind,excerpt,content,category_id,model,visibility) VALUES($1,$2,'prompt','100% literal_','body','cat-image-0','Flux',$3)",pg.t("square_items")))
                .bind(format!("qa-{i}")).bind(format!("QA {i}")).bind(if i==4 {"offline"} else {"online"}).execute(&pg.pool).await.unwrap();
        }
        let path = "/v1/square/search?q=100%25&category_id=cat-image&model=Flux&limit=2";
        let (code, first) = request(&state,"GET",path,"",json!(null)).await;
        assert_eq!(code,StatusCode::OK); assert_eq!(first["items"].as_array().unwrap().len(),2); assert_eq!(first["next_offset"],2);
        let (_, last) = request(&state,"GET",&format!("{path}&offset=2"),"",json!(null)).await;
        assert_eq!(last["items"].as_array().unwrap().len(),2); assert!(last["next_offset"].is_null());
        assert_ne!(first["items"][0]["id"],last["items"][0]["id"]);
        assert!(first["items"][0].get("content").is_none());
        let (_, empty) = request(&state,"GET","/v1/square/search?q=100%25&model=other","",json!(null)).await;
        assert_eq!(empty["items"],json!([]));
        for params in ["limit=0","limit=101","offset=-1","offset=100001","limit=1.2","unknown=x"] {
            assert_eq!(request(&state,"GET",&format!("/v1/square/search?{params}"),"",json!(null)).await.0,StatusCode::BAD_REQUEST);
        }
        state.set_square_public(false).await.unwrap();
        assert_eq!(request(&state,"GET",path,"",json!(null)).await.0,StatusCode::UNAUTHORIZED);
    }
}
