pub struct KeyringRefreshStore;

impl RefreshStore for KeyringRefreshStore {
    fn save_refresh(&self, token: &str) -> Result<(), String> {
        keyring::Entry::new("PromptArk", "refresh")
            .map_err(|error| error.to_string())?
            .set_password(token)
            .map_err(|error| error.to_string())
    }

    fn load_refresh(&self) -> Result<Option<String>, String> {
        match keyring::Entry::new("PromptArk", "refresh").map_err(|error| error.to_string())?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    fn clear_refresh(&self) -> Result<(), String> {
        match keyring::Entry::new("PromptArk", "refresh").map_err(|error| error.to_string())?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.to_string()),
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
