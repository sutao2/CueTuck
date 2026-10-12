use std::{fs, io::{Read, Write}, path::PathBuf};

pub struct FileRefreshStore(pub PathBuf);

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedSession {
    refresh_token: String,
}

impl FileRefreshStore {
    fn path(&self) -> Result<PathBuf, String> {
        let dir = self.0.join("auth");
        let path = dir.join("session.json");
        for (item, directory) in [(&dir, true), (&path, false)] {
            match fs::symlink_metadata(item) {
                Ok(meta) if meta.file_type().is_symlink() || (directory && !meta.is_dir()) || (!directory && !meta.is_file()) => return Err("登录配置路径无效".into()),
                Ok(_) => {},
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {},
                Err(_) => return Err("无法访问登录配置文件".into()),
            }
        }
        Ok(path)
    }
}

impl RefreshStore for FileRefreshStore {
    fn save_refresh(&self, token: &str) -> Result<(), String> {
        if !token.starts_with("ref.") || token.len() > 8192 {
            return Err("登录令牌无效".into());
        }
        let path = self.path()?;
        let dir = path.parent().unwrap();
        fs::create_dir_all(&self.0).map_err(|_| "无法创建应用数据目录")?;
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        { use std::os::unix::fs::DirBuilderExt; builder.mode(0o700); }
        match builder.create(dir) {
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(_) => return Err("无法创建登录配置目录".into()),
        }
        #[cfg(unix)]
        { use std::os::unix::fs::PermissionsExt; fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).map_err(|_| "无法设置登录配置权限")?; }
        let bytes = serde_json::to_vec(&SavedSession { refresh_token: token.into() }).map_err(|_| "登录配置编码失败")?;
        let mut temp = tempfile::NamedTempFile::new_in(dir).map_err(|_| "无法创建登录配置文件")?;
        #[cfg(unix)]
        { use std::os::unix::fs::PermissionsExt; temp.as_file().set_permissions(fs::Permissions::from_mode(0o600)).map_err(|_| "无法设置登录配置权限")?; }
        temp.write_all(&bytes).and_then(|_| temp.as_file().sync_all()).map_err(|_| "无法保存登录配置文件")?;
        temp.persist(path).map_err(|_| "无法替换登录配置文件")?;
        Ok(())
    }

    fn load_refresh(&self) -> Result<Option<String>, String> {
        let path = self.path()?;
        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("无法读取登录配置文件".into()),
        };
        let mut bytes = Vec::new();
        file.take(16385).read_to_end(&mut bytes).map_err(|_| "无法读取登录配置文件")?;
        if bytes.len() > 16384 { return Err("登录配置文件过大，请重新登录".into()); }
        let saved: SavedSession = serde_json::from_slice(&bytes).map_err(|_| "登录配置损坏，请重新登录")?;
        if !saved.refresh_token.starts_with("ref.") || saved.refresh_token.len() > 8192 { return Err("登录令牌无效，请重新登录".into()); }
        Ok(Some(saved.refresh_token))
    }

    fn clear_refresh(&self) -> Result<(), String> {
        match fs::remove_file(self.path()?) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err("无法删除登录配置文件".into()),
        }
    }
}

pub trait RefreshStore {
    fn save_refresh(&self, token: &str) -> Result<(), String>;
    fn load_refresh(&self) -> Result<Option<String>, String>;
    fn clear_refresh(&self) -> Result<(), String>;
}

#[derive(Default)]
#[cfg(test)]
pub struct MemoryRefreshStore {
    token: std::sync::Mutex<Option<String>>,
}

#[cfg(test)]
impl RefreshStore for MemoryRefreshStore {
    fn save_refresh(&self, token: &str) -> Result<(), String> {
        *self.token.lock().map_err(|error| error.to_string())? = Some(token.to_string());
        Ok(())
    }

    fn load_refresh(&self) -> Result<Option<String>, String> {
        Ok(self.token.lock().map_err(|error| error.to_string())?.clone())
    }

    fn clear_refresh(&self) -> Result<(), String> {
        *self.token.lock().map_err(|error| error.to_string())? = None;
        Ok(())
    }
}

pub fn persist_session_tokens(
    store: &dyn RefreshStore,
    access_token: &str,
    refresh_token: &str,
) -> Result<(), String> {
    if !refresh_token.starts_with("ref.") {
        return Err("refresh token 类型不对".to_string());
    }
    if !access_token.starts_with("acc.") {
        return Err("access token 类型不对".to_string());
    }
    store.save_refresh(refresh_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_session_survives_restart_rotates_and_clears() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileRefreshStore(dir.path().into());
        assert_eq!(store.load_refresh().unwrap(), None);
        persist_session_tokens(&store, "acc.first", "ref.first").unwrap();
        let restarted = FileRefreshStore(dir.path().into());
        assert_eq!(restarted.load_refresh().unwrap().as_deref(), Some("ref.first"));
        persist_session_tokens(&restarted, "acc.second", "ref.second").unwrap();
        let bytes = fs::read_to_string(dir.path().join("auth/session.json")).unwrap();
        assert_eq!(bytes, r#"{"refresh_token":"ref.second"}"#);
        assert_eq!(fs::read_dir(dir.path().join("auth")).unwrap().count(), 1);
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(dir.path().join("auth")).unwrap().permissions().mode() & 0o777, 0o700);
            assert_eq!(fs::metadata(dir.path().join("auth/session.json")).unwrap().permissions().mode() & 0o777, 0o600);
        }
        store.clear_refresh().unwrap();
        store.clear_refresh().unwrap();
        assert_eq!(restarted.load_refresh().unwrap(), None);
    }

    #[test]
    fn malformed_session_is_not_exposed_and_can_be_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileRefreshStore(dir.path().into());
        store.save_refresh("ref.original").unwrap();
        assert!(store.save_refresh("acc.invalid").is_err());
        assert_eq!(store.load_refresh().unwrap().as_deref(), Some("ref.original"));
        for bytes in [b"secret malformed".to_vec(), vec![b'x'; 16385]] {
            fs::write(dir.path().join("auth/session.json"), bytes).unwrap();
            let error = store.load_refresh().unwrap_err();
            assert!(!error.contains("secret"));
        }
        store.save_refresh("ref.relogin").unwrap();
        assert_eq!(store.load_refresh().unwrap().as_deref(), Some("ref.relogin"));
    }

    #[test]
    #[cfg(unix)]
    fn rejects_linked_session_without_touching_target() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileRefreshStore(dir.path().into());
        fs::create_dir(dir.path().join("auth")).unwrap();
        let target = dir.path().join("other");
        fs::write(&target, "unchanged").unwrap();
        std::os::unix::fs::symlink(&target, dir.path().join("auth/session.json")).unwrap();
        assert!(store.load_refresh().is_err());
        assert!(store.save_refresh("ref.new").is_err());
        assert!(store.clear_refresh().is_err());
        assert_eq!(fs::read_to_string(target).unwrap(), "unchanged");
    }

    #[test]
    fn refresh_goes_to_store_access_does_not() {
        let store = MemoryRefreshStore::default();
        persist_session_tokens(&store, "acc.1", "ref.1").unwrap();
        assert_eq!(store.load_refresh().unwrap().as_deref(), Some("ref.1"));
    }

    #[test]
    fn rejects_refresh_written_as_access() {
        let store = MemoryRefreshStore::default();
        let error = persist_session_tokens(&store, "ref.1", "ref.2").unwrap_err();
        assert!(error.contains("access token 类型不对"));
        assert!(store.load_refresh().unwrap().is_none());
    }

    #[test]
    fn rotate_replaces_refresh_in_store() {
        let store = MemoryRefreshStore::default();
        persist_session_tokens(&store, "acc.1", "ref.1").unwrap();
        persist_session_tokens(&store, "acc.2", "ref.2").unwrap();
        assert_eq!(store.load_refresh().unwrap().as_deref(), Some("ref.2"));
    }
}
