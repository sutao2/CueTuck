use super::{backup_library_in_dir, get_setting_in_dir, set_setting_in_dir};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static AUTO_BACKUP: Mutex<()> = Mutex::new(());
const INTERVAL_SECONDS: u64 = 24 * 60 * 60;

fn current_seconds() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn run_backup(dir: &Path, now: u64) -> Result<String, String> {
    let dest = dir.join("backups").join(format!("auto-{now}-{}.sqlite", uuid::Uuid::new_v4()));
    match backup_library_in_dir(dir, &dest) {
        Ok(path) => {
            set_setting_in_dir(dir, "auto_backup_last_success", &now.to_string())?;
            set_setting_in_dir(dir, "auto_backup_last_path", &path)?;
            set_setting_in_dir(dir, "auto_backup_error", "")?;
            Ok(path)
        }
        Err(error) => {
            let _ = set_setting_in_dir(dir, "auto_backup_error", &error);
            Err(error)
        }
    }
}

pub fn set_auto_backup_in_dir(dir: &Path, enabled: bool) -> Result<(), String> {
    let _guard = AUTO_BACKUP.lock().map_err(|error| error.to_string())?;
    if enabled { run_backup(dir, current_seconds())?; }
    set_setting_in_dir(dir, "auto_backup", if enabled { "1" } else { "0" })
}

pub fn auto_backup_if_due(dir: &Path, now: u64) -> Result<bool, String> {
    let _guard = AUTO_BACKUP.lock().map_err(|error| error.to_string())?;
    if get_setting_in_dir(dir, "auto_backup").unwrap_or_default() != "1" { return Ok(false); }
    let last = get_setting_in_dir(dir, "auto_backup_last_success").ok().and_then(|raw| raw.parse::<u64>().ok());
    if last.is_some_and(|last| now >= last && now - last < INTERVAL_SECONDS) { return Ok(false); }
    run_backup(dir, now)?;
    Ok(true)
}

pub fn start_auto_backup_worker(dir: PathBuf) {
    std::thread::spawn(move || loop {
        let _ = auto_backup_if_due(&dir, current_seconds());
        std::thread::sleep(Duration::from_secs(60));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_when_due_preserves_old_backups_and_stops_when_disabled() {
        let dir = tempfile::tempdir().unwrap();
        super::super::initialize_in_dir(dir.path()).unwrap();
        assert!(!auto_backup_if_due(dir.path(), 100).unwrap());
        set_setting_in_dir(dir.path(), "auto_backup", "1").unwrap();
        assert!(auto_backup_if_due(dir.path(), 100).unwrap());
        let first = get_setting_in_dir(dir.path(), "auto_backup_last_path").unwrap();
        assert!(!auto_backup_if_due(dir.path(), 101).unwrap());
        assert!(auto_backup_if_due(dir.path(), 100 + INTERVAL_SECONDS).unwrap());
        let second = get_setting_in_dir(dir.path(), "auto_backup_last_path").unwrap();
        assert_ne!(first, second);
        assert!(Path::new(&first).exists());
        set_auto_backup_in_dir(dir.path(), false).unwrap();
        assert!(!auto_backup_if_due(dir.path(), 100 + INTERVAL_SECONDS * 2).unwrap());
    }

    #[test]
    fn failed_first_backup_does_not_enable_or_claim_success_and_can_retry() {
        let dir = tempfile::tempdir().unwrap();
        super::super::initialize_in_dir(dir.path()).unwrap();
        let blocked = dir.path().join("backups");
        std::fs::write(&blocked, "not a directory").unwrap();
        assert!(set_auto_backup_in_dir(dir.path(), true).is_err());
        assert_ne!(get_setting_in_dir(dir.path(), "auto_backup").unwrap_or_default(), "1");
        assert!(get_setting_in_dir(dir.path(), "auto_backup_last_success").is_err());
        assert!(!get_setting_in_dir(dir.path(), "auto_backup_error").unwrap().is_empty());
        std::fs::remove_file(&blocked).unwrap();
        set_auto_backup_in_dir(dir.path(), true).unwrap();
        assert_eq!(get_setting_in_dir(dir.path(), "auto_backup").unwrap(), "1");
        assert_eq!(get_setting_in_dir(dir.path(), "auto_backup_error").unwrap(), "");
    }
}
