use crate::db::{models::ActivityLog, Database};
use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use std::sync::Arc;
use uuid::Uuid;

pub struct ActivityLogger {
    db: Arc<Database>,
    rate_limit_per_minute: usize,
    log_count_this_minute: parking_lot::Mutex<(chrono::DateTime<Utc>, usize)>,
}

impl ActivityLogger {
    pub fn new(db: Arc<Database>, rate_limit_per_minute: usize) -> Self {
        ActivityLogger {
            db,
            rate_limit_per_minute,
            log_count_this_minute: parking_lot::Mutex::new((Utc::now(), 0)),
        }
    }

    pub fn log(&self, entry: ActivityLog) -> Result<()> {
        // Rate limiting
        {
            let mut guard = self.log_count_this_minute.lock();
            let now = Utc::now();
            if (now - guard.0).num_seconds() >= 60 {
                *guard = (now, 0);
            }
            if guard.1 >= self.rate_limit_per_minute {
                return Ok(());
            }
            guard.1 += 1;
        }

        let conn = self.db.conn.lock();
        conn.execute(
            "INSERT INTO activity_logs
             (id, session_id, profile_id, event_type, action_kind, trigger_kind,
              result, failure_reason, restore_status, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                entry.id,
                entry.session_id,
                entry.profile_id,
                serde_json::to_string(&entry.event_type)?,
                entry.action_kind,
                entry.trigger_kind,
                entry.result,
                entry.failure_reason,
                entry.restore_status,
                entry.metadata.map(|m| serde_json::to_string(&m).unwrap_or_default()),
                entry.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_logs(
        &self,
        session_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ActivityLog>> {
        let conn = self.db.conn.lock();
        let sql = if session_id.is_some() {
            "SELECT id, session_id, profile_id, event_type, action_kind, trigger_kind,
              result, failure_reason, restore_status, metadata, created_at
             FROM activity_logs WHERE session_id = ?1 ORDER BY created_at DESC LIMIT ?2"
        } else {
            "SELECT id, session_id, profile_id, event_type, action_kind, trigger_kind,
              result, failure_reason, restore_status, metadata, created_at
             FROM activity_logs ORDER BY created_at DESC LIMIT ?2"
        };

        let mut stmt = conn.prepare(sql)?;

        let parse = |row: &rusqlite::Row| -> rusqlite::Result<ActivityLog> {
            Ok(ActivityLog {
                id: row.get(0)?,
                session_id: row.get(1)?,
                profile_id: row.get(2)?,
                event_type: serde_json::from_str(&row.get::<_, String>(3)?)
                    .unwrap_or(crate::db::models::EventType::ActionApplied),
                action_kind: row.get(4)?,
                trigger_kind: row.get(5)?,
                result: row.get(6)?,
                failure_reason: row.get(7)?,
                restore_status: row.get(8)?,
                metadata: row
                    .get::<_, Option<String>>(9)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                created_at: row
                    .get::<_, String>(10)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
            })
        };

        let rows = if let Some(sid) = session_id {
            stmt.query_map(params![sid, limit], parse)?
                .filter_map(|r| r.ok())
                .collect()
        } else {
            stmt.query_map(params![rusqlite::types::Null, limit], |row| {
                Ok(ActivityLog {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    profile_id: row.get(2)?,
                    event_type: serde_json::from_str(&row.get::<_, String>(3)?)
                        .unwrap_or(crate::db::models::EventType::ActionApplied),
                    action_kind: row.get(4)?,
                    trigger_kind: row.get(5)?,
                    result: row.get(6)?,
                    failure_reason: row.get(7)?,
                    restore_status: row.get(8)?,
                    metadata: row
                        .get::<_, Option<String>>(9)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    created_at: row
                        .get::<_, String>(10)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect()
        };

        Ok(rows)
    }

    pub fn make_entry(event_type: crate::db::models::EventType) -> ActivityLog {
        ActivityLog {
            id: Uuid::new_v4().to_string(),
            session_id: None,
            profile_id: None,
            event_type,
            action_kind: None,
            trigger_kind: None,
            result: "ok".to_string(),
            failure_reason: None,
            restore_status: None,
            metadata: None,
            created_at: Utc::now(),
        }
    }
}
