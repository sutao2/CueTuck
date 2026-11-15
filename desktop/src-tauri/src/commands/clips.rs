// Separate from the prompt database: its sync, backups and MCP never see clips.
use rusqlite::{params, Connection, TransactionBehavior};
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Search {
        query: String,
        limit: Option<u32>,
    },
    Save {
        id: Option<String>,
        kind: String,
        title: String,
        value: Option<String>,
    },
    CopyValue {
        id: String,
    },
    Delete {
        id: String,
    },
}

fn credential(id: &str) -> Result<keyring::Entry, String> {
    if !cfg!(any(target_os = "macos", target_os = "windows")) {
        return Err("此平台不支持安全保存密码".into());
    }
    keyring::Entry::new("CueTuck.quick-clips", id).map_err(|_| "无法访问系统凭据库".into())
}

fn db(_: rusqlite::Error) -> String {
    "无法读写本机片段库".into()
}

fn run(
    c: &mut Connection,
    request: Request,
    entry: impl Fn(&str) -> Result<keyring::Entry, String>,
) -> Result<Value, String> {
    c.execute_batch(
        "CREATE TABLE IF NOT EXISTS clips (
        id TEXT PRIMARY KEY, kind TEXT NOT NULL, title TEXT NOT NULL, body TEXT,
        CHECK ((kind='password') = (body IS NULL)))",
    )
    .map_err(db)?;
    match request {
        Request::Search { query, limit } => {
            let mut stmt=c.prepare("SELECT id,kind,title,body FROM clips WHERE instr(lower(title),lower(?1))>0 OR instr(lower(body),lower(?1))>0 ORDER BY CASE WHEN lower(title)=lower(?1) THEN 0 WHEN instr(lower(title),lower(?1))=1 THEN 1 WHEN instr(lower(title),lower(?1))>0 THEN 2 ELSE 3 END, rowid DESC LIMIT ?2").map_err(db)?;
            let rows=stmt.query_map(params![query.trim(),limit.map(i64::from).unwrap_or(-1)],|r| Ok(json!({
                "id":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"title":r.get::<_,String>(2)?,"content":r.get::<_,Option<String>>(3)?
            }))).map_err(db)?.collect::<Result<Vec<_>,_>>().map_err(db)?;
            Ok(json!(rows))
        }
        Request::Save {
            id,
            kind,
            title,
            value,
        } => {
            if !matches!(
                kind.as_str(),
                "text" | "url" | "account" | "template" | "password"
            ) || title.trim().is_empty()
                || title.len() > 500
                || value
                    .as_ref()
                    .is_some_and(|s| s.is_empty() || s.len() > 262144)
            {
                return Err("请检查片段的名称、类型和内容".into());
            }
            let secret = kind == "password";
            if value.is_none() && !(secret && id.is_some()) {
                return Err("请填写片段内容".into());
            }
            let tx = c
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(db)?;
            let body = if secret { None } else { value.as_deref() };
            let id = if let Some(id) = id {
                if tx
                    .execute(
                        "UPDATE clips SET title=?1,body=?2 WHERE id=?3 AND kind=?4",
                        params![title.trim(), body, id, kind],
                    )
                    .map_err(db)?
                    != 1
                {
                    return Err("片段已删除或类型已改变".into());
                }
                id
            } else {
                let id = uuid::Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO clips VALUES (?1,?2,?3,?4)",
                    params![id, kind, title.trim(), body],
                )
                .map_err(db)?;
                id
            };
            if secret {
                if let Some(value) = value {
                    entry(&id)?
                        .set_password(&value)
                        .map_err(|_| "密码未保存，请检查系统凭据库授权")?;
                }
            }
            tx.commit().map_err(db)?;
            Ok(json!(id))
        }
        Request::CopyValue { id } => {
            let body: Option<String> = c
                .query_row("SELECT body FROM clips WHERE id=?1", [&id], |r| r.get(0))
                .map_err(db)?;
            Ok(json!(match body {
                Some(value) => value,
                None => entry(&id)?
                    .get_password()
                    .map_err(|_| "无法读取密码，请检查系统凭据库授权")?,
            }))
        }
        Request::Delete { id } => {
            let tx = c
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(db)?;
            let kind: String = tx
                .query_row("SELECT kind FROM clips WHERE id=?1", [&id], |r| r.get(0))
                .map_err(db)?;
            tx.execute("DELETE FROM clips WHERE id=?1", [&id])
                .map_err(db)?;
            if kind == "password" {
                match entry(&id)?.delete_credential() {
                    Ok(()) | Err(keyring::Error::NoEntry) => (),
                    Err(_) => return Err("密码未删除，请检查系统凭据库授权".into()),
                }
            }
            tx.commit().map_err(db)?;
            Ok(Value::Null)
        }
    }
}

#[tauri::command]
pub async fn quick_clip(app: AppHandle, request: Request) -> Result<Value, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|_| "无法读取本机片段目录")?;
    tauri::async_runtime::spawn_blocking(move || {
        std::fs::create_dir_all(&dir).map_err(|_| "无法创建本机片段目录")?;
        let mut c = Connection::open(dir.join("quick-clips.sqlite")).map_err(db)?;
        run(&mut c, request, credential)
    })
    .await
    .map_err(|_| "片段操作未完成".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persisted_templates_search_and_copy_verbatim() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clips.sqlite");
        let mut c = Connection::open(&path).unwrap();
        let text = "  https://example.invalid/{{name}}\n";
        let id = run(
            &mut c,
            Request::Save {
                id: None,
                kind: "template".into(),
                title: "Site".into(),
                value: Some(text.into()),
            },
            |_| panic!(),
        )
        .unwrap();
        drop(c);
        let mut c = Connection::open(&path).unwrap();
        let rows = run(
            &mut c,
            Request::Search {
                query: "EXAMPLE".into(),
                limit: None,
            },
            |_| panic!(),
        )
        .unwrap();
        assert_eq!(rows[0]["id"], id);
        assert_eq!(
            run(
                &mut c,
                Request::CopyValue {
                    id: id.as_str().unwrap().into()
                },
                |_| panic!()
            )
            .unwrap(),
            text
        );
    }
    #[test]
    fn credentials_fail_closed_and_search_never_reads_a_secret() {
        let mut c = Connection::open_in_memory().unwrap();
        let save = || Request::Save {
            id: None,
            kind: "password".into(),
            title: "Work".into(),
            value: Some("demo secret".into()),
        };
        assert!(run(&mut c, save(), |_| Err("denied".into())).is_err());
        let search = |q: &str| Request::Search {
            query: q.into(),
            limit: None,
        };
        assert_eq!(run(&mut c, search(""), |_| panic!()).unwrap(), json!([]));
        let id = run(&mut c, save(), |id| {
            keyring::mock::default_credential_builder()
                .build(None, "test", id)
                .map(keyring::Entry::new_with_credential)
                .map_err(|_| "mock".into())
        })
        .unwrap();
        assert_eq!(
            run(&mut c, search("secret"), |_| panic!()).unwrap(),
            json!([])
        );
        let rows = run(&mut c, search("Work"), |_| panic!()).unwrap();
        assert_eq!(rows[0]["id"], id);
        assert!(rows[0]["content"].is_null());
    }
}
