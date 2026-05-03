pub mod builtin;

use crate::db::{models::Profile, Database};
use anyhow::Result;
use rusqlite::params;
use std::sync::Arc;

pub struct ProfileRepository {
    db: Arc<Database>,
}

impl ProfileRepository {
    pub fn new(db: Arc<Database>) -> Self {
        ProfileRepository { db }
    }

    pub fn list(&self) -> Result<Vec<Profile>> {
        let conn = self.db.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, mode_type, triggers, actions, restore_policy, approval_mode,
                    is_builtin, enabled, created_at, updated_at
             FROM profiles ORDER BY is_builtin DESC, name ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i32>(7)?,
                row.get::<_, i32>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
            ))
        })?;

        let mut profiles = vec![];
        for row in rows {
            let (id, name, mode_type, triggers, actions, restore_policy, approval_mode,
                is_builtin, enabled, created_at, updated_at) = row?;
            profiles.push(Profile {
                id,
                name,
                mode_type: serde_json::from_str(&mode_type).unwrap_or(
                    crate::db::models::ModeType::Custom,
                ),
                triggers: serde_json::from_str(&triggers).unwrap_or_default(),
                actions: serde_json::from_str(&actions).unwrap_or_default(),
                restore_policy: serde_json::from_str(&restore_policy).unwrap_or(
                    crate::db::models::RestorePolicy::OnAppQuit,
                ),
                approval_mode: serde_json::from_str(&approval_mode).unwrap_or(
                    crate::db::models::ApprovalMode::AskFirst,
                ),
                is_builtin: is_builtin != 0,
                enabled: enabled != 0,
                created_at: created_at.parse().unwrap_or_else(|_| chrono::Utc::now()),
                updated_at: updated_at.parse().unwrap_or_else(|_| chrono::Utc::now()),
            });
        }
        Ok(profiles)
    }

    pub fn get(&self, id: &str) -> Result<Option<Profile>> {
        let all = self.list()?;
        Ok(all.into_iter().find(|p| p.id == id))
    }

    pub fn upsert(&self, profile: &Profile) -> Result<()> {
        let conn = self.db.conn.lock();
        conn.execute(
            "INSERT INTO profiles (id, name, mode_type, triggers, actions, restore_policy,
              approval_mode, is_builtin, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name,
               mode_type = excluded.mode_type,
               triggers = excluded.triggers,
               actions = excluded.actions,
               restore_policy = excluded.restore_policy,
               approval_mode = excluded.approval_mode,
               enabled = excluded.enabled,
               updated_at = excluded.updated_at",
            params![
                profile.id,
                profile.name,
                serde_json::to_string(&profile.mode_type)?,
                serde_json::to_string(&profile.triggers)?,
                serde_json::to_string(&profile.actions)?,
                serde_json::to_string(&profile.restore_policy)?,
                serde_json::to_string(&profile.approval_mode)?,
                profile.is_builtin as i32,
                profile.enabled as i32,
                profile.created_at.to_rfc3339(),
                profile.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.db.conn.lock();
        conn.execute("DELETE FROM profiles WHERE id = ?1 AND is_builtin = 0", params![id])?;
        Ok(())
    }
}
