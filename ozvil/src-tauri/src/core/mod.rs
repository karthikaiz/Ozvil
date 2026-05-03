pub mod monitoring_loop;
pub mod rules_engine;
pub mod session_manager;
pub mod snapshot_manager;
pub mod activity_logger;

use crate::db::{models::Settings, Database};
use crate::profiles::builtin::seed_builtin_profiles;
use anyhow::Result;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppState {
    pub db: Arc<Database>,
    pub settings: RwLock<Settings>,
    pub safe_mode: bool,
    pub active_session_id: RwLock<Option<String>>,
}

impl AppState {
    pub fn new(db_path: PathBuf, safe_mode: bool) -> Result<Self> {
        let db = Arc::new(Database::open(&db_path)?);
        let settings = load_settings(&db).unwrap_or_default();

        seed_builtin_profiles(&db)?;
        reconcile_stale_sessions(&db)?;

        Ok(AppState {
            db,
            settings: RwLock::new(settings),
            safe_mode,
            active_session_id: RwLock::new(None),
        })
    }

    pub fn is_global_pause(&self) -> bool {
        self.settings.read().global_pause
    }
}

fn load_settings(db: &Database) -> Option<Settings> {
    let conn = db.conn.lock();
    let val: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'app_settings'",
            [],
            |r| r.get(0),
        )
        .ok();
    val.and_then(|v| serde_json::from_str(&v).ok())
}

fn reconcile_stale_sessions(db: &Database) -> Result<()> {
    let conn = db.conn.lock();
    conn.execute(
        "UPDATE sessions SET status = 'stale' WHERE status = 'active'",
        [],
    )?;
    Ok(())
}
