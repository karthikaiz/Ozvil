pub mod migrations;
pub mod models;

use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;

pub struct Database {
    pub conn: parking_lot::Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let db = Database {
            conn: parking_lot::Mutex::new(conn),
        };

        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch(migrations::SCHEMA)?;
        Ok(())
    }

    pub fn prune_logs(&self, max_entries: usize) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM activity_logs WHERE id NOT IN (
                SELECT id FROM activity_logs ORDER BY created_at DESC LIMIT ?1
            )",
            params![max_entries],
        )?;
        Ok(())
    }
}
