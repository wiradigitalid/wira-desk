//! Migrate user data from a legacy WinTick installation (M-01).
// LEGACY: remove in v0.3.0
//! Copy, do not move — the `%APPDATA%\WinTick\` directory MUST remain intact so
//! rollback remains possible.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::constants::{APP_DIR_NAME, CONFIG_FILE_NAME, LOG_FILE_NAME};

const LEGACY_APP_DIR: &str = "WinTick";
const LEGACY_LOG_FILE: &str = "wintick.log";

fn appdata_root() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

fn resolve_appdata_root(override_root: Option<&Path>) -> Option<PathBuf> {
    override_root.map(Path::to_path_buf).or_else(appdata_root)
}

fn legacy_dir_at(appdata: Option<&Path>) -> Option<PathBuf> {
    resolve_appdata_root(appdata).map(|p| p.join(LEGACY_APP_DIR))
}

fn new_dir_at(appdata: Option<&Path>) -> PathBuf {
    match resolve_appdata_root(appdata) {
        Some(appdata) => appdata.join(APP_DIR_NAME),
        None => PathBuf::from(".").join(APP_DIR_NAME),
    }
}

fn append_migration_log(new_log: &Path, line: &str) {
    if let Some(parent) = new_log.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(new_log)
    {
        let _ = writeln!(f, "{line}");
    }
}

/// M-01: copy legacy `config.toml` and log into the Wira Desk directory when needed.
/// Idempotent — if `%APPDATA%\WiraDesk\` already exists, does nothing.
/// Copy failures are logged to the new log as Tier-2 warnings and do not
/// abort startup.
pub fn migrate_appdata() {
    migrate_appdata_impl(None);
}

fn migrate_appdata_impl(appdata: Option<&Path>) {
    let new_path = new_dir_at(appdata);
    if new_path.exists() {
        return;
    }

    let Some(legacy_path) = legacy_dir_at(appdata) else {
        return;
    };
    if !legacy_path.exists() {
        return;
    }

    if fs::create_dir_all(&new_path).is_err() {
        return;
    }

    let legacy_config = legacy_path.join(CONFIG_FILE_NAME);
    let new_config = new_path.join(CONFIG_FILE_NAME);
    if legacy_config.exists() {
        if let Err(e) = fs::copy(&legacy_config, &new_config) {
            append_migration_log(
                &new_path.join(LOG_FILE_NAME),
                &format!("MIGRATE WARN: failed to copy config.toml: {e}"),
            );
        }
    }

    let legacy_log = legacy_path.join(LEGACY_LOG_FILE);
    let new_log = new_path.join(LOG_FILE_NAME);
    if legacy_log.exists() {
        if let Err(e) = fs::copy(&legacy_log, &new_log) {
            append_migration_log(
                &new_path.join(LOG_FILE_NAME),
                &format!("MIGRATE WARN: failed to copy log: {e}"),
            );
        }
    }

    append_migration_log(
        &new_path.join(LOG_FILE_NAME),
        "MIGRATE: config imported from legacy installation",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn temp_appdata() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("wiradesk-migrate-test-{}", std::process::id()));
        p
    }

    #[test]
    fn skips_when_new_dir_already_exists() {
        let _guard = lock();
        let root = temp_appdata();
        let _ = fs::remove_dir_all(&root);
        let legacy = root.join(LEGACY_APP_DIR);
        let new_path = root.join(APP_DIR_NAME);
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join(CONFIG_FILE_NAME), "x = 1").unwrap();
        fs::create_dir_all(&new_path).unwrap();
        fs::write(new_path.join(CONFIG_FILE_NAME), "y = 2").unwrap();

        migrate_appdata_impl(Some(&root));
        let contents = fs::read_to_string(new_path.join(CONFIG_FILE_NAME)).unwrap();
        assert_eq!(contents, "y = 2");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn copies_config_and_preserves_legacy() {
        let _guard = lock();
        let root = temp_appdata();
        let _ = fs::remove_dir_all(&root);
        let legacy = root.join(LEGACY_APP_DIR);
        fs::create_dir_all(&legacy).unwrap();
        fs::write(
            legacy.join(CONFIG_FILE_NAME),
            "[general]\nauto_start = true\n",
        )
        .unwrap();

        migrate_appdata_impl(Some(&root));

        let new_path = root.join(APP_DIR_NAME);
        assert!(new_path.exists());
        assert!(legacy.exists());
        let copied = fs::read_to_string(new_path.join(CONFIG_FILE_NAME)).unwrap();
        assert!(copied.contains("auto_start"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn idempotent_second_run() {
        let _guard = lock();
        let root = temp_appdata();
        let _ = fs::remove_dir_all(&root);
        let legacy = root.join(LEGACY_APP_DIR);
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join(CONFIG_FILE_NAME), "a = 1").unwrap();

        migrate_appdata_impl(Some(&root));
        let new_config = root.join(APP_DIR_NAME).join(CONFIG_FILE_NAME);
        fs::write(&new_config, "b = 2").unwrap();
        migrate_appdata_impl(Some(&root));
        assert_eq!(fs::read_to_string(new_config).unwrap(), "b = 2");
        let _ = fs::remove_dir_all(&root);
    }
}
