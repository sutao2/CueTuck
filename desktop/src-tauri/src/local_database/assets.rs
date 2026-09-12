use base64::{engine::general_purpose::STANDARD, Engine};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub mime: String,
    pub data: String,
}

pub fn validate(assets: &[Asset]) -> Result<Vec<Vec<u8>>, String> {
    if assets.len() > 12 { return Err("每条提示词最多 12 个附件".into()); }
    let mut total = 0;
    let mut ids = HashSet::new();
    assets.iter().map(|asset| {
        if Uuid::parse_str(&asset.id).is_err() || !ids.insert(&asset.id) { return Err("附件 id 无效或重复".into()); }
        if asset.name.is_empty() || asset.name.chars().count() > 255 || asset.name.chars().any(|c| c.is_control() || c == '/' || c == '\\') { return Err("附件名称无效".into()); }
        if asset.data.len() > 7 * 1024 * 1024 { return Err("单个附件不能超过 5 MiB".into()); }
        let bytes = STANDARD.decode(&asset.data).map_err(|_| "附件编码无效")?;
        total += bytes.len();
        if bytes.len() > 5 * 1024 * 1024 || total > 20 * 1024 * 1024 { return Err("单文件上限 5 MiB，每条提示词附件总量上限 20 MiB".into()); }
        let ext = asset.name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        let valid = match (ext.as_str(), asset.mime.as_str()) {
            ("png", "image/png") => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            ("jpg" | "jpeg", "image/jpeg") => bytes.starts_with(&[255, 216, 255]),
            ("gif", "image/gif") => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
            ("webp", "image/webp") => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP"),
            ("pdf", "application/pdf") => bytes.starts_with(b"%PDF-"),
            ("txt" | "md" | "csv" | "json", "text/plain") => std::str::from_utf8(&bytes).is_ok() && !bytes.contains(&0),
            ("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document") |
            ("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet") |
            ("pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation") => bytes.starts_with(b"PK\x03\x04"),
            _ => false,
        };
        if !valid { return Err(format!("不支持的附件类型或内容不匹配：{}", asset.name)); }
        Ok(bytes)
    }).collect()
}

pub(crate) fn replace(connection: &Connection, prompt_id: &str, assets: &[Asset]) -> Result<(), String> {
    let decoded = validate(assets)?;
    connection.execute("DELETE FROM prompt_assets WHERE prompt_id=?1", [prompt_id]).map_err(|e| e.to_string())?;
    for (position, (asset, bytes)) in assets.iter().zip(decoded).enumerate() {
        connection.execute("INSERT INTO prompt_assets(prompt_id,id,name,mime,data,position) VALUES(?1,?2,?3,?4,?5,?6)",
            params![prompt_id, asset.id, asset.name, asset.mime, bytes, position as i64]).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(crate) fn read(connection: &Connection, prompt_id: &str) -> Result<Vec<Asset>, String> {
    let mut statement = connection.prepare("SELECT id,name,mime,data FROM prompt_assets WHERE prompt_id=?1 ORDER BY position").map_err(|e| e.to_string())?;
    let rows = statement.query_map([prompt_id], |row| Ok(Asset {
        id: row.get(0)?, name: row.get(1)?, mime: row.get(2)?, data: STANDARD.encode(row.get::<_, Vec<u8>>(3)?),
    })).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn list(dir: &Path, prompt_id: &str) -> Result<Vec<Asset>, String> {
    let connection = Connection::open(dir.join("promptark.sqlite")).map_err(|e| e.to_string())?;
    let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM prompts WHERE id=?1 AND deleted_at IS NULL)", [prompt_id], |row| row.get(0)).map_err(|e| e.to_string())?;
    if !exists { return Err("提示词不存在".into()); }
    let assets = read(&connection, prompt_id)?;
    validate(&assets)?;
    Ok(assets)
}

pub fn first_image(dir: &Path, prompt_id: &str) -> Result<Option<Asset>, String> {
    use rusqlite::OptionalExtension;
    let connection = Connection::open(dir.join("promptark.sqlite")).map_err(|e|e.to_string())?;
    let asset = connection.query_row("SELECT a.id,a.name,a.mime,a.data FROM prompt_assets a JOIN prompts p ON p.id=a.prompt_id WHERE p.id=?1 AND p.deleted_at IS NULL AND a.mime IN ('image/png','image/jpeg','image/gif','image/webp') ORDER BY a.position LIMIT 1", [prompt_id], |row| Ok(Asset {
        id:row.get(0)?,name:row.get(1)?,mime:row.get(2)?,data:STANDARD.encode(row.get::<_,Vec<u8>>(3)?),
    })).optional().map_err(|e|e.to_string())?;
    if let Some(ref asset) = asset { validate(std::slice::from_ref(asset))?; }
    Ok(asset)
}

pub fn first_thumbnail(dir: &Path, prompt_id: &str) -> Result<Option<Asset>, String> {
    use image::ImageDecoder;
    use std::io::Cursor;
    let Some(mut asset) = first_image(dir, prompt_id)? else { return Ok(None); };
    let bytes = STANDARD.decode(&asset.data).map_err(|e| e.to_string())?;
    let mut reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format().map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|e| e.to_string())?;
    if decoder.total_bytes() > 64 * 1024 * 1024 { return Err("图片过大，无法生成缩略图".into()); }
    let orientation = decoder.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut picture = image::DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
    if picture.width() > 480 || picture.height() > 480 { picture = picture.thumbnail(480, 480); }
    picture.apply_orientation(orientation);
    let mut output = Cursor::new(Vec::new());
    picture.write_to(&mut output, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    asset.data = STANDARD.encode(output.into_inner());
    asset.mime = "image/png".into(); asset.name = "thumbnail.png".into();
    Ok(Some(asset))
}

pub fn save_prompt(dir: &Path, id: Option<&str>, title: &str, content: &str, category: Option<&str>, model: Option<&str>, assets: &[Asset]) -> Result<super::PromptRecord, String> {
    if title.trim().is_empty() { return Err("标题不能为空".into()); }
    validate(assets)?;
    let mut connection = Connection::open(dir.join("promptark.sqlite")).map_err(|e| e.to_string())?;
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let stamp = super::now_millis();
    let prompt_id = id.map(str::to_string).unwrap_or_else(|| Uuid::new_v4().to_string());
    if id.is_some() {
        let changed = transaction.execute("UPDATE prompts SET title=?1,content=?2,category_id=?3,model=?4,updated_at=?5 WHERE id=?6 AND deleted_at IS NULL", params![title.trim(), content, category, model, stamp, prompt_id]).map_err(|e| e.to_string())?;
        if changed == 0 { return Err("提示词不存在".into()); }
    } else {
        transaction.execute("INSERT INTO prompts(id,title,content,category_id,model,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?6)", params![prompt_id,title.trim(),content,category,model,stamp]).map_err(|e| e.to_string())?;
    }
    replace(&transaction, &prompt_id, assets)?;
    let result = super::prompts::read_prompt(&transaction, &prompt_id)?;
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(result)
}

pub fn export_file(dir: &Path, prompt_id: &str, asset_id: &str) -> Result<String, String> {
    let asset = list(dir, prompt_id)?.into_iter().find(|a| a.id == asset_id).ok_or("附件不存在")?;
    let decoded = validate(std::slice::from_ref(&asset))?;
    // A unique directory avoids overwrites; only a validated basename is used.
    let destination = dir.join("exports").join(Uuid::new_v4().to_string());
    std::fs::create_dir_all(&destination).map_err(|e| e.to_string())?;
    let path = destination.join(asset.name);
    std::fs::write(&path, &decoded[0]).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

pub fn append_downloaded(dir: &Path, prompt_id: &str, remote_id: &str, additions: &[Asset]) -> Result<super::PromptRecord, String> {
    validate(additions)?;
    let mut connection = Connection::open(dir.join("promptark.sqlite")).map_err(|e|e.to_string())?;
    let tx = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
    let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM prompts WHERE id=?1 AND remote_id=?2 AND deleted_at IS NULL)",params![prompt_id,remote_id],|r|r.get(0)).map_err(|e|e.to_string())?;
    if !exists { return Err("本地副本不存在或来源已变化".into()); }
    let mut assets = read(&tx,prompt_id)?; let before = assets.len();
    for asset in additions {
        if !assets.iter().any(|old|old.mime==asset.mime && old.data==asset.data) {
            let mut asset = asset.clone(); asset.id=Uuid::new_v4().to_string(); assets.push(asset);
        }
    }
    if assets.len()>before {
        replace(&tx,prompt_id,&assets)?;
        tx.execute("UPDATE prompts SET updated_at=?1 WHERE id=?2",params![super::now_millis(),prompt_id]).map_err(|e|e.to_string())?;
    }
    let row = super::prompts::read_prompt(&tx,prompt_id)?;
    tx.commit().map_err(|e|e.to_string())?; Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn file(name: &str, raw: &[u8], mime: &str) -> Asset { Asset { id: Uuid::new_v4().to_string(), name: name.into(), mime: mime.into(), data: STANDARD.encode(raw) } }
    #[test]
    fn assets_persist_and_lists_return_only_counts() {
        let dir = tempfile::tempdir().unwrap(); super::super::initialize_in_dir(dir.path()).unwrap();
        let assets = vec![file("参考.txt", "离线资料".as_bytes(), "text/plain"), file("image.png", b"\x89PNG\r\n\x1a\n", "image/png")];
        let prompt = save_prompt(dir.path(), None, "图片提示词", "正文", None, None, &assets).unwrap();
        assert_eq!((prompt.asset_count, prompt.image_count), (2, 1));
        super::super::initialize_in_dir(dir.path()).unwrap();
        assert_eq!(list(dir.path(), &prompt.id).unwrap()[0].data, assets[0].data);
        let rows = super::super::list_prompts_in_dir(dir.path(), "", None).unwrap();
        assert_eq!(rows[0].asset_count, 2);
        assert!(!serde_json::to_string(&rows).unwrap().contains(&assets[0].data));
        let path = export_file(dir.path(), &prompt.id, &assets[0].id).unwrap();
        assert_eq!(std::fs::read(path).unwrap(), "离线资料".as_bytes());
    }
    #[test]
    fn cover_reads_only_first_image_and_ignores_deleted_prompts() {
        let dir=tempfile::tempdir().unwrap();super::super::initialize_in_dir(dir.path()).unwrap();
        let text=file("note.txt",b"notes","text/plain");
        let first=file("first.png",b"\x89PNG\r\n\x1a\n","image/png");
        let second=file("second.gif",b"GIF89a","image/gif");
        let row=save_prompt(dir.path(),None,"cover","body",None,None,&[text.clone(),first.clone(),second]).unwrap();
        let conn=Connection::open(dir.path().join("promptark.sqlite")).unwrap();
        // A malformed other attachment must not be read or validated for the cover.
        conn.execute("UPDATE prompt_assets SET data=x'00' WHERE prompt_id=?1 AND position=2",[&row.id]).unwrap();
        assert_eq!(first_image(dir.path(),&row.id).unwrap().unwrap().id,first.id);
        save_prompt(dir.path(),Some(&row.id),"cover","body",None,None,&[text]).unwrap();
        assert!(first_image(dir.path(),&row.id).unwrap().is_none());
        save_prompt(dir.path(),Some(&row.id),"cover","body",None,None,&[first]).unwrap();
        conn.execute("UPDATE prompts SET deleted_at=1 WHERE id=?1",[&row.id]).unwrap();
        assert!(first_image(dir.path(),&row.id).unwrap().is_none());
        assert!(first_image(dir.path(),"missing").unwrap().is_none());
    }
    #[test]
    fn thumbnail_bounds_pixels_preserves_alpha_and_never_replaces_original() {
        let dir = tempfile::tempdir().unwrap(); super::super::initialize_in_dir(dir.path()).unwrap();
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(1600, 800, image::Rgba([80, 120, 200, 100])))
            .write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        let original = file("large.png", bytes.get_ref(), "image/png");
        let row = save_prompt(dir.path(), None, "cover", "full text", None, None, std::slice::from_ref(&original)).unwrap();
        let thumbnail = first_thumbnail(dir.path(), &row.id).unwrap().unwrap();
        let decoded = image::load_from_memory(&STANDARD.decode(&thumbnail.data).unwrap()).unwrap().into_rgba8();
        assert_eq!(decoded.dimensions(), (480, 240));
        assert_eq!(decoded.get_pixel(0, 0).0[3], 100);
        assert!(thumbnail.data.len() < original.data.len());
        assert_eq!(first_image(dir.path(), &row.id).unwrap().unwrap().data, original.data);
    }
    #[test]
    fn thumbnail_handles_supported_formats_small_images_invalid_data_and_limits() {
        let dir = tempfile::tempdir().unwrap(); super::super::initialize_in_dir(dir.path()).unwrap();
        for (format, name, mime) in [(image::ImageFormat::Jpeg, "test.jpg", "image/jpeg"), (image::ImageFormat::Gif, "test.gif", "image/gif"), (image::ImageFormat::WebP, "test.webp", "image/webp")] {
            let mut bytes = std::io::Cursor::new(Vec::new());
            image::DynamicImage::new_rgb8(32, 16).write_to(&mut bytes, format).unwrap();
            let row = save_prompt(dir.path(), None, "cover", "body", None, None, &[file(name, bytes.get_ref(), mime)]).unwrap();
            let thumbnail = first_thumbnail(dir.path(), &row.id).unwrap().unwrap();
            let picture = image::load_from_memory(&STANDARD.decode(thumbnail.data).unwrap()).unwrap();
            assert_eq!((picture.width(), picture.height()), (32, 16));
        }
        let row = save_prompt(dir.path(), None, "bad", "body", None, None, &[file("bad.png", b"\x89PNG\r\n\x1a\n", "image/png")]).unwrap();
        assert!(first_thumbnail(dir.path(), &row.id).is_err());
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(8193, 1).write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        save_prompt(dir.path(), Some(&row.id), "wide", "body", None, None, &[file("wide.png", bytes.get_ref(), "image/png")]).unwrap();
        assert!(first_thumbnail(dir.path(), &row.id).is_err());
        assert!(first_thumbnail(dir.path(), "missing").unwrap().is_none());
    }
    #[test]
    fn supplemental_images_preserve_text_deduplicate_and_rollback_on_failure() {
        let dir=tempfile::tempdir().unwrap(); super::super::initialize_in_dir(dir.path()).unwrap();
        let prompt=save_prompt(dir.path(),None,"edited","keep body",None,None,&[file("old.txt",b"keep file","text/plain")]).unwrap();
        let conn=Connection::open(dir.path().join("promptark.sqlite")).unwrap();
        conn.execute("UPDATE prompts SET remote_id='square-test' WHERE id=?1",[&prompt.id]).unwrap();
        let image=file("image.png",b"\x89PNG\r\n\x1a\n","image/png");
        let row=append_downloaded(dir.path(),&prompt.id,"square-test",std::slice::from_ref(&image)).unwrap();
        assert_eq!(row.content,"keep body");assert_eq!(row.title,"edited");assert_eq!(row.asset_count,2);
        assert_eq!(append_downloaded(dir.path(),&prompt.id,"square-test",std::slice::from_ref(&image)).unwrap().asset_count,2);
        assert!(append_downloaded(dir.path(),&prompt.id,"wrong",std::slice::from_ref(&image)).is_err());
        conn.execute_batch("CREATE TRIGGER reject_extra BEFORE INSERT ON prompt_assets BEGIN SELECT RAISE(ABORT,'failure'); END;").unwrap();
        assert!(append_downloaded(dir.path(),&prompt.id,"square-test",&[file("another.png",b"\x89PNG\r\n\x1a\nother","image/png")]).is_err());
        assert_eq!(list(dir.path(),&prompt.id).unwrap().len(),2);
        conn.execute("UPDATE prompts SET deleted_at=1 WHERE id=?1",[&prompt.id]).unwrap();
        assert!(append_downloaded(dir.path(),&prompt.id,"square-test",&[image]).is_err());
    }
    #[test]
    fn attachment_failure_rolls_back_text_and_existing_files() {
        let dir = tempfile::tempdir().unwrap(); super::super::initialize_in_dir(dir.path()).unwrap();
        let assets = vec![file("old.txt", b"original", "text/plain")];
        let prompt = save_prompt(dir.path(), None, "original", "body", None, None, &assets).unwrap();
        let conn = Connection::open(dir.path().join("promptark.sqlite")).unwrap();
        conn.execute_batch("CREATE TRIGGER reject_asset BEFORE INSERT ON prompt_assets BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert!(save_prompt(dir.path(), Some(&prompt.id), "changed", "changed", None, None, &assets).is_err());
        assert_eq!(super::super::list_prompts_in_dir(dir.path(), "", None).unwrap()[0].title, "original");
        assert_eq!(list(dir.path(), &prompt.id).unwrap()[0].data, assets[0].data);
    }
    #[test]
    fn invalid_and_oversized_files_are_rejected_before_save() {
        assert!(validate(&[file("bad.svg", b"<svg/>", "image/svg+xml")]).is_err());
        assert!(validate(&[file("../escape.txt", b"text", "text/plain")]).is_err());
        assert!(validate(&[file("fake.png", b"<html>", "image/png")]).is_err());
        assert!(validate(&[file("large.txt", &vec![b'a'; 5 * 1024 * 1024 + 1], "text/plain")]).is_err());
        let many: Vec<_> = (0..13).map(|_| file("a.txt", b"a", "text/plain")).collect();
        assert!(validate(&many).is_err());
        let total: Vec<_> = (0..5).map(|_| file("a.txt", &vec![b'a'; 5 * 1024 * 1024], "text/plain")).collect();
        assert!(validate(&total).is_err());
    }
    #[test]
    fn collection_import_keeps_file_ownership_and_rolls_back_all_members_on_failure() {
        let dir = tempfile::tempdir().unwrap(); super::super::initialize_in_dir(dir.path()).unwrap();
        let first = file("first.txt", b"first", "text/plain");
        let second = file("second.txt", b"second", "text/plain");
        let snapshot = serde_json::json!({"version":2,"collections":[{"id":"download","title":"collection"}],"prompts":[
            {"title":"first","content":"body one","collection_id":"download","assets":[first]},
            {"title":"second","content":"body two","collection_id":"download","assets":[second]},
        ]}).to_string();
        let conn = Connection::open(dir.path().join("promptark.sqlite")).unwrap();
        conn.execute_batch("CREATE TRIGGER fail_second BEFORE INSERT ON prompt_assets WHEN NEW.name='second.txt' BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert!(super::super::apply_import_json_in_dir(dir.path(), &snapshot).is_err());
        for table in ["prompts","collections","prompt_assets"] {
            assert_eq!(conn.query_row(&format!("SELECT count(*) FROM {table}"),[],|row| row.get::<_,i64>(0)).unwrap(),0);
        }
        conn.execute_batch("DROP TRIGGER fail_second;").unwrap();
        super::super::apply_import_json_in_dir(dir.path(), &snapshot).unwrap();
        let rows = super::super::list_prompts_in_dir(dir.path(), "", None).unwrap();
        assert_eq!(rows.len(),2);
        for row in rows {
            let assets = list(dir.path(), &row.id).unwrap();
            assert_eq!(assets.len(),1); assert_eq!(assets[0].name,format!("{}.txt",row.title));
            let collection: String = conn.query_row("SELECT collection_id FROM prompts WHERE id=?1",[&row.id],|r| r.get(0)).unwrap();
            assert_ne!(collection,"download");
            assert_eq!(conn.query_row("SELECT title FROM collections WHERE id=?1",[collection],|r| r.get::<_,String>(0)).unwrap(),"collection");
        }
    }
    #[test]
    fn local_exports_roundtrip_but_sync_cannot_read_or_replace_assets() {
        let dir = tempfile::tempdir().unwrap(); super::super::initialize_in_dir(dir.path()).unwrap();
        let assets = vec![file("private.txt", b"private-file-content", "text/plain")];
        let prompt = save_prompt(dir.path(), None, "With file", "body", None, None, &assets).unwrap();
        let sync = super::super::export_sync_changes(dir.path()).unwrap();
        assert!(!serde_json::to_string(&sync).unwrap().contains("private-file-content"));
        assert!(sync.iter().find(|r| r.id == prompt.id).unwrap().payload.get("assets").is_none());
        let json = super::super::export_library_json_in_dir(dir.path()).unwrap();
        super::super::apply_import_json_in_dir(dir.path(), &json).unwrap();
        let rows = super::super::list_prompts_in_dir(dir.path(), "", None).unwrap();
        assert_eq!(rows.len(), 2);
        for row in &rows { assert_eq!(list(dir.path(), &row.id).unwrap()[0].data, assets[0].data); }
        let change = super::super::SyncChange { id: prompt.id.clone(), kind: "prompt".into(), updated_at: super::super::now_millis(), deleted_at: None,
            payload: serde_json::json!({"title":"remote edit", "content":"new", "assets":[]}) };
        super::super::apply_sync_changes(dir.path(), &[change], false).unwrap();
        assert_eq!(list(dir.path(), &prompt.id).unwrap().len(), 1);
        let backup = dir.path().join("backup.sqlite");
        super::super::backup_library_in_dir(dir.path(), &backup).unwrap();
        save_prompt(dir.path(), Some(&prompt.id), "empty", "", None, None, &[]).unwrap();
        super::super::restore_library_in_dir(dir.path(), &backup).unwrap();
        assert_eq!(list(dir.path(), &prompt.id).unwrap()[0].data, assets[0].data);
    }
}
