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
        kind: Option<String>,
        category_id: Option<String>,
    },
    Save {
        id: Option<String>,
        previous_kind: Option<String>,
        kind: String,
        title: String,
        value: Option<String>,
        category_id: Option<String>,
    },
    Categories,
    CreateCategory {
        name: String,
    },
    RenameCategory {
        id: String,
        name: String,
    },
    DeleteCategory {
        id: String,
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

fn credential_error(error: keyring::Error) -> String {
    #[cfg(target_os = "macos")]
    if let keyring::Error::NoStorageAccess(cause) | keyring::Error::PlatformFailure(cause) = &error
    {
        if let Some(err) = cause.downcast_ref::<security_framework::base::Error>() {
            return match err.code() {
                -25307 | -25294 => {
                    "当前用户没有默认钥匙串。请在“钥匙串访问”中设置默认登录钥匙串，再重试。".into()
                }
                -128 => "已取消钥匙串授权，可重试。".into(),
                -25308 | -25293 => {
                    "钥匙串已锁定或访问被拒绝。请解锁登录钥匙串并允许访问，再重试。".into()
                }
                code => format!("钥匙串操作失败（系统代码 {code}）"),
            };
        }
    }
    match error {
        keyring::Error::TooLong(_, _) => "密码超过系统凭据库的长度限制".into(),
        keyring::Error::NoEntry => "系统凭据库中找不到此密码，请编辑资料重新保存。".into(),
        keyring::Error::NoStorageAccess(_) => {
            "当前登录会话无法访问系统凭据库，请解锁系统后重试。".into()
        }
        _ => "系统凭据库操作失败，请重试。".into(),
    }
}

fn db(_: rusqlite::Error) -> String {
    "无法读写本机资料库".into()
}

fn category_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 60 {
        return Err("分类名称须为 1 至 60 个字符".into());
    }
    Ok(name)
}

fn run(
    c: &mut Connection,
    request: Request,
    entry: impl Fn(&str) -> Result<keyring::Entry, String>,
) -> Result<Value, String> {
    c.execute_batch(
        "PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS clips (
        id TEXT PRIMARY KEY, kind TEXT NOT NULL, title TEXT NOT NULL, body TEXT,
        CHECK ((kind='password') = (body IS NULL)));
        CREATE TABLE IF NOT EXISTS clip_categories (
            id TEXT PRIMARY KEY, name TEXT NOT NULL COLLATE NOCASE UNIQUE);
        CREATE TABLE IF NOT EXISTS clip_membership (
            clip_id TEXT PRIMARY KEY REFERENCES clips(id) ON DELETE CASCADE,
            category_id TEXT NOT NULL REFERENCES clip_categories(id) ON DELETE CASCADE);
        CREATE INDEX IF NOT EXISTS clip_membership_category ON clip_membership(category_id)",
    )
    .map_err(db)?;
    match request {
        Request::Categories => {
            let tx = c.transaction().map_err(db)?;
            let categories = tx.prepare(
                "SELECT c.id, c.name, count(m.clip_id) FROM clip_categories c
                 LEFT JOIN clip_membership m ON m.category_id=c.id GROUP BY c.id ORDER BY c.name"
            ).map_err(db)?.query_map([], |r| Ok(json!({
                "id":r.get::<_,String>(0)?, "name":r.get::<_,String>(1)?, "count":r.get::<_,i64>(2)?
            }))).map_err(db)?.collect::<Result<Vec<_>,_>>().map_err(db)?;
            let (total, uncategorized): (i64, i64) = tx
                .query_row(
                    "SELECT count(*), count(*)-count(m.category_id) FROM clips c
                 LEFT JOIN clip_membership m ON m.clip_id=c.id",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .map_err(db)?;
            Ok(json!({"categories":categories,"total":total,"uncategorized":uncategorized}))
        }
        Request::CreateCategory { name } => {
            let id = uuid::Uuid::new_v4().to_string();
            let changed = c
                .execute(
                    "INSERT OR IGNORE INTO clip_categories(id,name) VALUES (?1,?2)",
                    params![id, category_name(&name)?],
                )
                .map_err(db)?;
            if changed == 0 {
                return Err("分类名称已存在".into());
            }
            Ok(json!(id))
        }
        Request::RenameCategory { id, name } => {
            let changed = c
                .execute(
                    "UPDATE OR IGNORE clip_categories SET name=?1 WHERE id=?2",
                    params![category_name(&name)?, id],
                )
                .map_err(db)?;
            if changed == 0 {
                return Err("分类不存在或名称已被使用".into());
            }
            Ok(Value::Null)
        }
        Request::DeleteCategory { id } => {
            c.execute("DELETE FROM clip_categories WHERE id=?1", [id])
                .map_err(db)?;
            Ok(Value::Null)
        }
        Request::Search {
            query,
            limit,
            kind,
            category_id,
        } => {
            let mut stmt = c.prepare(
                "SELECT c.id,c.kind,c.title,c.body,m.category_id FROM clips c
                 LEFT JOIN clip_membership m ON m.clip_id=c.id
                 WHERE (?3 IS NULL OR c.kind=?3) AND (?4 IS NULL OR coalesce(m.category_id,'')=?4)
                 AND (instr(lower(title),lower(?1))>0 OR instr(lower(body),lower(?1))>0)
                 ORDER BY CASE WHEN lower(title)=lower(?1) THEN 0 WHEN instr(lower(title),lower(?1))=1 THEN 1
                 WHEN instr(lower(title),lower(?1))>0 THEN 2 ELSE 3 END, c.rowid DESC LIMIT ?2"
            ).map_err(db)?;
            let rows = stmt.query_map(params![query.trim(),limit.map(i64::from).unwrap_or(-1),kind,category_id], |r| Ok(json!({
                "id":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"title":r.get::<_,String>(2)?,
                "content":r.get::<_,Option<String>>(3)?,"category_id":r.get::<_,Option<String>>(4)?
            }))).map_err(db)?.collect::<Result<Vec<_>,_>>().map_err(db)?;
            Ok(json!(rows))
        }
        Request::Save {
            id,
            previous_kind,
            kind,
            title,
            value,
            category_id,
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
                return Err("请检查资料的名称、类型和内容".into());
            }
            let secret = kind == "password";
            let expected_kind = previous_kind.as_deref().unwrap_or(&kind);
            let was_password = id.is_some() && expected_kind == "password";
            if value.is_none() && !(secret && was_password) {
                return Err("请填写资料内容".into());
            }
            let tx = c
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(db)?;
            let body = if secret { None } else { value.as_deref() };
            let id = if let Some(id) = id {
                if tx
                    .execute(
                        "UPDATE clips SET title=?1,body=?2,kind=?3 WHERE id=?4 AND kind=?5",
                        params![title.trim(), body, kind, id, expected_kind],
                    )
                    .map_err(db)?
                    != 1
                {
                    return Err("资料已删除或类型已改变".into());
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
            if let Some(category) = category_id {
                if category.is_empty() {
                    tx.execute("DELETE FROM clip_membership WHERE clip_id=?1", [&id])
                        .map_err(db)?;
                } else {
                    let changed = tx
                        .execute(
                            "INSERT INTO clip_membership(clip_id,category_id)
                         SELECT ?1,id FROM clip_categories WHERE id=?2
                         ON CONFLICT(clip_id) DO UPDATE SET category_id=excluded.category_id",
                            params![id, category],
                        )
                        .map_err(db)?;
                    if changed == 0 {
                        return Err("分类已删除，请重新选择".into());
                    }
                }
            }
            if (secret && value.is_some()) || (was_password && !secret) {
                let key = entry(&id)?;
                let old_password = if was_password {
                    match key.get_password() {
                        Ok(old) => Some(old),
                        Err(keyring::Error::NoEntry) => None,
                        Err(error) => return Err(credential_error(error)),
                    }
                } else {
                    None
                };
                if let Some(password) = value.as_deref().filter(|_| secret) {
                    key.set_password(password).map_err(credential_error)?;
                } else if old_password.is_some() {
                    key.delete_credential().map_err(credential_error)?;
                }
                // Keep the write lock until both stores agree, including recovery on a failed commit.
                if let Err(error) = tx.execute_batch("COMMIT") {
                    match old_password {
                        Some(old) => key.set_password(&old).map_err(credential_error)?,
                        None if secret => key.delete_credential().map_err(credential_error)?,
                        None => (),
                    }
                    return Err(db(error));
                }
            } else {
                tx.commit().map_err(db)?;
            }
            Ok(json!(id))
        }
        Request::CopyValue { id } => {
            let body: Option<String> = c
                .query_row("SELECT body FROM clips WHERE id=?1", [&id], |r| r.get(0))
                .map_err(db)?;
            Ok(json!(match body {
                Some(value) => value,
                None => entry(&id)?.get_password().map_err(credential_error)?,
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
                    Err(error) => return Err(credential_error(error)),
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
        .map_err(|_| "无法读取本机资料目录")?;
    tauri::async_runtime::spawn_blocking(move || {
        std::fs::create_dir_all(&dir).map_err(|_| "无法创建本机资料目录")?;
        let mut c = Connection::open(dir.join("quick-clips.sqlite")).map_err(db)?;
        run(&mut c, request, credential)
    })
    .await
    .map_err(|_| "资料操作未完成".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "macos")]
    #[test]
    fn default_keychain_missing_is_distinct_from_refused_access() {
        let error = |code| {
            keyring::Error::PlatformFailure(Box::new(security_framework::base::Error::from_code(
                code,
            )))
        };
        assert!(credential_error(error(-25307)).contains("没有默认钥匙串"));
        assert!(credential_error(error(-128)).contains("已取消"));
        assert!(credential_error(error(-25293)).contains("拒绝"));
    }
    #[test]
    fn persisted_templates_search_and_copy_verbatim() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clips.sqlite");
        let mut c = Connection::open(&path).unwrap();
        let text = "  https://example.invalid/{{name}}\n";
        let id = run(
            &mut c,
            Request::Save {
                previous_kind: None,
                category_id: None,
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
                category_id: None,
                kind: None,
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
    fn type_changes_preserve_category_and_verbatim_body_and_reject_stale_or_failed_saves() {
        let mut c = Connection::open_in_memory().unwrap();
        let mut call = |request| {
            run(&mut c, serde_json::from_value(request).unwrap(), |_| {
                Err("vault unavailable".into())
            })
        };
        let group = call(json!({"action":"create_category","name":"Office"})).unwrap();
        let body = "  Agenda\r\n\r\n\t{{speaker}}  \r\n";
        let id = call(json!({"action":"save","kind":"text","title":"Meeting","value":body,"category_id":group})).unwrap();
        let convert = |from, to, value| json!({"action":"save","id":id,"previous_kind":from,"kind":to,"title":"Meeting","value":value});
        assert_eq!(call(convert("text", "template", body)).unwrap(), id);
        assert!(call(convert("text", "url", "stale")).is_err());
        assert!(call(convert("template", "password", "test-only")).is_err());
        let rows =
            call(json!({"action":"search","kind":"template","query":"","category_id":group}))
                .unwrap();
        assert_eq!(rows[0]["id"], id);
        assert_eq!(rows[0]["content"], body);
        assert_eq!(call(json!({"action":"copy_value","id":id})).unwrap(), body);
    }

    #[test]
    fn legacy_clips_can_be_organized_without_reading_or_losing_secrets() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("quick-clips.sqlite");
        let c = Connection::open(&path).unwrap();
        c.execute_batch("CREATE TABLE clips(id TEXT PRIMARY KEY, kind TEXT NOT NULL, title TEXT NOT NULL,
            body TEXT, CHECK ((kind='password') = (body IS NULL)));
            INSERT INTO clips VALUES ('pw','password','Work login',NULL), ('url','url','Work site','https://example.invalid');").unwrap();
        drop(c);
        let mut c = Connection::open(&path).unwrap();
        let mut call = |req: Value| {
            run(&mut c, serde_json::from_value(req).unwrap(), |_| {
                panic!("unexpected credential access")
            })
        };
        assert_eq!(
            call(json!({"action":"categories"})).unwrap()["uncategorized"],
            2
        );
        let group = call(json!({"action":"create_category","name":" Work "})).unwrap();
        assert!(call(json!({"action":"create_category","name":"work"})).is_err());
        let mut edit = json!({"action":"save","id":"pw","kind":"password","title":"Work login"});
        edit["category_id"] = group.clone();
        call(edit.clone()).unwrap();
        edit.as_object_mut().unwrap().remove("category_id");
        call(edit.clone()).unwrap();
        let search = |category: Value, kind: &str, query: &str| json!({"action":"search","category_id":category,"kind":kind,"query":query});
        assert_eq!(
            call(search(group.clone(), "password", "work")).unwrap()[0]["id"],
            "pw"
        );
        assert_eq!(
            call(search(group.clone(), "url", "work")).unwrap(),
            json!([])
        );
        assert_eq!(
            call(search(group.clone(), "password", "absent")).unwrap(),
            json!([])
        );
        assert_eq!(
            call(search(json!(""), "url", "example")).unwrap()[0]["id"],
            "url"
        );
        edit["category_id"] = json!("missing");
        assert!(call(edit.clone()).is_err());
        call(json!({"action":"rename_category","id":group,"name":"Office"})).unwrap();
        let summary = call(json!({"action":"categories"})).unwrap();
        assert_eq!(
            summary,
            json!({"total":2,"uncategorized":1,"categories":[{"id":group,"name":"Office","count":1}]})
        );
        edit["category_id"] = json!("");
        call(edit.clone()).unwrap();
        assert_eq!(
            call(search(json!(""), "password", "work")).unwrap()[0]["id"],
            "pw"
        );
        edit["category_id"] = group.clone();
        call(edit).unwrap();
        call(json!({"action":"delete_category","id":group})).unwrap();
        assert_eq!(
            call(json!({"action":"categories"})).unwrap()["uncategorized"],
            2
        );
        assert_eq!(
            call(search(json!(""), "password", "work")).unwrap()[0]["content"],
            Value::Null
        );
    }
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[test]
    #[ignore = "requires a writable test-user credential store"]
    fn password_clip_roundtrip_in_system_store() {
        let mut database = Connection::open_in_memory().unwrap();
        let value = uuid::Uuid::new_v4().to_string();
        let saved = run(
            &mut database,
            Request::Save {
                previous_kind: None,
                category_id: None,
                id: None,
                kind: "password".into(),
                title: "QA".into(),
                value: Some(value.clone()),
            },
            credential,
        )
        .unwrap();
        let id = saved.as_str().unwrap().to_owned();
        let value_read = run(
            &mut database,
            Request::CopyValue { id: id.clone() },
            credential,
        );
        let body: Option<String> = database
            .query_row("SELECT body FROM clips WHERE id=?1", [&id], |row| {
                row.get(0)
            })
            .unwrap();
        let convert = |from, to, content: &str| {
            serde_json::from_value(json!({
                "action":"save", "id":id, "previous_kind":from, "kind":to,
                "title":"QA", "value":content
            }))
            .unwrap()
        };
        let note = "  Example\n\n\tResult\n";
        run(&mut database, convert("password", "text", note), credential).unwrap();
        assert!(matches!(
            credential(&id).unwrap().get_password(),
            Err(keyring::Error::NoEntry)
        ));
        assert_eq!(
            run(
                &mut database,
                Request::CopyValue { id: id.clone() },
                credential
            )
            .unwrap(),
            note
        );
        run(
            &mut database,
            convert("text", "password", &value),
            credential,
        )
        .unwrap();
        assert_eq!(credential(&id).unwrap().get_password().unwrap(), value);
        assert!(database
            .query_row("SELECT body IS NULL FROM clips WHERE id=?1", [&id], |row| {
                row.get::<_, bool>(0)
            })
            .unwrap());
        run(
            &mut database,
            Request::Delete { id: id.clone() },
            credential,
        )
        .unwrap();
        assert!(matches!(
            credential(&id).unwrap().get_password(),
            Err(keyring::Error::NoEntry)
        ));
        assert!(body.is_none());
        assert!(value_read.unwrap().as_str() == Some(&value));
    }
    #[test]
    fn credentials_fail_closed_and_search_never_reads_a_secret() {
        let mut c = Connection::open_in_memory().unwrap();
        let save = || Request::Save {
            previous_kind: None,
            category_id: None,
            id: None,
            kind: "password".into(),
            title: "Work".into(),
            value: Some("demo secret".into()),
        };
        assert!(run(&mut c, save(), |_| Err("denied".into())).is_err());
        let search = |q: &str| Request::Search {
            category_id: None,
            kind: None,
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
