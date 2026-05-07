use crate::db::{
    models::{
        Profile, Session,
        SessionStatus, SystemSnapshot, TriggerSource,
    },
    Database,
};
use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use std::sync::Arc;
use uuid::Uuid;

/// Serialize a simple string enum to a plain SQL value (no JSON quotes).
/// serde_json::to_string produces `"active"` (with quotes) which breaks SQL WHERE comparisons.
fn enum_to_sql<T: serde::Serialize>(val: &T) -> Result<String> {
    let json = serde_json::to_string(val)?;
    if json.starts_with('"') && json.ends_with('"') && json.len() >= 2 {
        Ok(json[1..json.len() - 1].to_string())
    } else {
        Ok(json)
    }
}

/// Deserialize a plain SQL string back to an enum by wrapping it in JSON quotes first.
fn sql_to_enum<T: for<'de> serde::Deserialize<'de>>(s: &str) -> Option<T> {
    serde_json::from_str(&format!("\"{}\"", s)).ok()
}

pub struct SessionManager {
    pub db: Arc<Database>,
}

impl SessionManager {
    pub fn new(db: Arc<Database>) -> Self {
        SessionManager { db }
    }

    pub fn start_session(
        &self,
        profile: &Profile,
        trigger: TriggerSource,
        snapshot: SystemSnapshot,
        safe_mode: bool,
    ) -> Result<Session> {
        let session = Session {
            id: Uuid::new_v4().to_string(),
            profile_id: profile.id.clone(),
            trigger_source: trigger,
            started_at: Utc::now(),
            ended_at: None,
            status: SessionStatus::Active,
            snapshot: Some(snapshot),
            safe_mode,
        };

        let conn = self.db.conn.lock();
        conn.execute(
            "INSERT INTO sessions (id, profile_id, trigger_source, started_at, status, snapshot, safe_mode)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session.id,
                session.profile_id,
                enum_to_sql(&session.trigger_source)?,
                session.started_at.to_rfc3339(),
                enum_to_sql(&session.status)?,
                serde_json::to_string(&session.snapshot)?,
                session.safe_mode as i32,
            ],
        )?;

        Ok(session)
    }

    pub fn end_session(&self, session_id: &str) -> Result<()> {
        let conn = self.db.conn.lock();
        conn.execute(
            "UPDATE sessions SET status = 'ended', ended_at = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), session_id],
        )?;
        Ok(())
    }

    pub fn get_active_session(&self) -> Result<Option<Session>> {
        let conn = self.db.conn.lock();
        let result = conn.query_row(
            "SELECT id, profile_id, trigger_source, started_at, ended_at, status, snapshot, safe_mode
             FROM sessions WHERE status = 'active' LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, i32>(7)?,
                ))
            },
        );

        match result {
            Ok((id, profile_id, trigger_src, started_at, ended_at, status, snapshot, safe_mode)) => {
                Ok(Some(Session {
                    id,
                    profile_id,
                    trigger_source: sql_to_enum(&trigger_src).unwrap_or(TriggerSource::ManualUi),
                    started_at: started_at.parse().unwrap_or_else(|_| Utc::now()),
                    ended_at: ended_at.and_then(|s| s.parse().ok()),
                    status: sql_to_enum(&status).unwrap_or(SessionStatus::Active),
                    snapshot: snapshot.and_then(|s| serde_json::from_str(&s).ok()),
                    safe_mode: safe_mode != 0,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_stale_sessions(&self) -> Result<Vec<Session>> {
        let conn = self.db.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, profile_id, trigger_source, started_at, ended_at, status, snapshot, safe_mode
             FROM sessions WHERE status = 'stale'",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, i32>(7)?,
            ))
        })?;

        let mut sessions = vec![];
        for row in rows {
            let (id, profile_id, trigger_src, started_at, ended_at, status, snapshot, safe_mode) =
                row?;
            sessions.push(Session {
                id,
                profile_id,
                trigger_source: sql_to_enum(&trigger_src).unwrap_or(TriggerSource::ManualUi),
                started_at: started_at.parse().unwrap_or_else(|_| Utc::now()),
                ended_at: ended_at.and_then(|s| s.parse().ok()),
                status: sql_to_enum(&status).unwrap_or(SessionStatus::Stale),
                snapshot: snapshot.and_then(|s| serde_json::from_str(&s).ok()),
                safe_mode: safe_mode != 0,
            });
        }

        Ok(sessions)
    }

    pub fn dismiss_stale_session(&self, session_id: &str) -> Result<()> {
        let conn = self.db.conn.lock();
        conn.execute(
            "UPDATE sessions SET status = 'ended' WHERE id = ?1",
            params![session_id],
        )?;
        Ok(())
    }
}
