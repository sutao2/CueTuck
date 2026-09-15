use std::{collections::HashMap, sync::{Arc, Mutex}, time::{Duration, Instant}};
use axum::http::StatusCode;
use sha2::{Digest, Sha256};

pub struct Candidate {
    pub id: String,
    pub literal: bool,
    pub featured: bool,
    pub downloads: i64,
    pub listed: Option<f64>,
}

struct Snapshot { ids: Arc<Vec<String>>, created: Instant }
#[derive(Default)]
pub struct Recommendations { snapshots: Mutex<HashMap<String, Snapshot>> }
const TTL: Duration = Duration::from_secs(7200);
const MAX_IDS: usize = 500_000;
const MAX_CANDIDATES: usize = 100_000;

impl Recommendations {
    pub fn get(&self, key: &str) -> Result<Option<Arc<Vec<String>>>, StatusCode> {
        let mut cache = self.snapshots.lock().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        cache.retain(|_, s| s.created.elapsed() < TTL);
        Ok(cache.get(key).map(|s| s.ids.clone()))
    }
    pub fn insert(&self, key: String, rows: Vec<Candidate>, seed: &str) -> Result<Arc<Vec<String>>, StatusCode> {
        if rows.len() > MAX_CANDIDATES { return Err(StatusCode::SERVICE_UNAVAILABLE); }
        let ids = Arc::new(rank(rows, seed, chrono::Utc::now().timestamp() as f64));
        let mut cache = self.snapshots.lock().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        cache.retain(|_, s| s.created.elapsed() < TTL);
        // Concurrent first pages must receive the same snapshot.
        if let Some(snapshot) = cache.get(&key) { return Ok(snapshot.ids.clone()); }
        while cache.len() >= 64 || cache.values().map(|s|s.ids.len()).sum::<usize>() + ids.len() > MAX_IDS {
            if let Some(oldest) = cache.iter().min_by_key(|(_,s)|s.created).map(|(k,_)|k.clone()) { cache.remove(&oldest); } else { break; }
        }
        cache.insert(key, Snapshot { ids: ids.clone(), created: Instant::now() });
        Ok(ids)
    }
}

fn rank(rows: Vec<Candidate>, seed: &str, now: f64) -> Vec<String> {
    let max_popularity = (rows.iter().map(|r|r.downloads.max(0)).max().unwrap_or(0) as f64 + 1.0).ln().max(1.0);
    let mut scored: Vec<_> = rows.into_iter().map(|row| {
        let digest = Sha256::digest(format!("{seed}:{}", row.id).as_bytes());
        let noise = u64::from_be_bytes(digest[..8].try_into().unwrap()) as f64 / u64::MAX as f64;
        let fresh = row.listed.map(|time| (1.0 - (now - time).max(0.0) / (30.0 * 86400.0)).clamp(0.0,1.0)).unwrap_or(0.0);
        let popularity = (row.downloads.max(0) as f64 + 1.0).ln() / max_popularity;
        let score = if row.literal {2.0} else {0.0} + noise * 0.60 + if row.featured {0.20} else {0.0} + fresh * 0.10 + popularity * 0.10;
        (row.id, score)
    }).collect();
    scored.sort_by(|a,b|b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().map(|(id,_)|id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rows() -> Vec<Candidate> { (0..150).map(|i|Candidate {id:format!("id-{i}"),literal:false,featured:false,downloads:i,listed:None}).collect() }
    #[test]
    fn stable_rotation_and_positive_quality_signals() {
        let a = rank(rows(), "a", 0.0); let b = rank(rows(), "b", 0.0);
        assert_eq!(a, rank(rows(), "a", 0.0)); assert_ne!(a[..48], b[..48]);
        assert_eq!(a.iter().collect::<std::collections::HashSet<_>>().len(),150);
        let mut promoted = rows(); promoted[149].featured=true; promoted[149].listed=Some(0.0);
        let improved=rank(promoted,"a",0.0);
        assert!(improved.iter().position(|id|id=="id-149") <= a.iter().position(|id|id=="id-149"));
    }
    #[test]
    fn snapshots_freeze_changes_expire_and_remain_bounded() {
        let cache=Recommendations::default();let first=cache.insert("one".into(),rows(),"a").unwrap();
        assert_eq!(cache.insert("one".into(),vec![],"b").unwrap(),first);
        cache.snapshots.lock().unwrap().get_mut("one").unwrap().created=Instant::now()-TTL;
        assert!(cache.get("one").unwrap().is_none());
        for i in 0..70 {cache.insert(i.to_string(),rows(),"a").unwrap();}
        assert_eq!(cache.snapshots.lock().unwrap().len(),64);
    }
}
