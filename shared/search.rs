use std::sync::OnceLock;
fn near_word(a:&str,b:&str)->bool {
    if !(4..=32).contains(&a.len()) || !(4..=32).contains(&b.len()) || !a.bytes().all(|c|c.is_ascii_lowercase()) || !b.bytes().all(|c|c.is_ascii_lowercase()) || a.len().abs_diff(b.len())>1 {return false;}
    let (a,b)=(a.as_bytes(),b.as_bytes());
    if a.len()==b.len() {let diff:Vec<_>=(0..a.len()).filter(|&i|a[i]!=b[i]).collect();return diff.len()<=1 || (diff.len()==2 && diff[1]==diff[0]+1 && a[diff[0]]==b[diff[1]] && a[diff[1]]==b[diff[0]]);}
    let (short,long)=if a.len()<b.len(){(a,b)}else{(b,a)};
    let i=(0..short.len()).find(|&i|short[i]!=long[i]).unwrap_or(short.len());short[i..]==long[i+1..]
}
pub fn terms(query:&str)->Vec<String> {
    let needle=query.trim().to_lowercase();if needle.is_empty(){return vec![];}
    static ALIASES:OnceLock<Vec<Vec<String>>>=OnceLock::new();
    let aliases=ALIASES.get_or_init(||serde_json::from_str(include_str!("search-aliases.json")).expect("checked search aliases"));
    let exact=aliases.iter().any(|g|g.contains(&needle));let mut result=vec![needle.clone()];
    for group in aliases {if if exact {group.contains(&needle)}else{group.iter().any(|term|near_word(&needle,term))} {for term in group {if !result.contains(term)&&result.len()<32 {result.push(term.clone());}}}}
    result
}
pub fn patterns(query:&str)->Vec<String> {terms(query).iter().map(|s|format!("%{}%",s.replace('\\',"\\\\").replace('%',"\\%").replace('_',"\\_"))).collect()}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn multilingual_typo_and_literal_search(){assert!(terms("photograpy").contains(&"摄影".into()));assert!(terms("图片").contains(&"image".into()));assert!(terms("imaeg").contains(&"图片".into()));assert_eq!(terms("zzzzzz"),vec!["zzzzzz"]);assert_eq!(patterns("100%_"),vec!["%100\\%\\_%"]);assert!(terms("").is_empty());assert!(!near_word("图像","图象"));}
}
