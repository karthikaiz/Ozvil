use crate::core::AppState;
use crate::core::activity_logger::ActivityLogger;
use crate::core::session_manager::SessionManager;
use crate::core::snapshot_manager::SnapshotManager;
use crate::db::models::*;
use crate::profiles::ProfileRepository;
use crate::windows_adapter::{WindowsAdapter, WindowsNativeAdapter};
use chrono::Utc;
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

#[derive(Serialize)]
pub struct AppStateInfo {
    pub safe_mode: bool,
    pub global_pause: bool,
    pub version: String,
}

#[tauri::command]
pub async fn get_app_state_info(state: State<'_, AppState>) -> Result<AppStateInfo, String> {
    Ok(AppStateInfo {
        safe_mode: state.safe_mode,
        global_pause: state.is_global_pause(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// ─── Profiles ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_profiles(state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    let repo = ProfileRepository::new(state.db.clone());
    repo.list().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_profile(id: String, state: State<'_, AppState>) -> Result<Option<Profile>, String> {
    let repo = ProfileRepository::new(state.db.clone());
    repo.get(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_profile(profile: Profile, state: State<'_, AppState>) -> Result<Profile, String> {
    let mut p = profile;
    p.id = Uuid::new_v4().to_string();
    p.is_builtin = false;
    p.created_at = Utc::now();
    p.updated_at = Utc::now();
    let repo = ProfileRepository::new(state.db.clone());
    repo.upsert(&p).map_err(|e| e.to_string())?;
    Ok(p)
}

#[tauri::command]
pub async fn update_profile(profile: Profile, state: State<'_, AppState>) -> Result<Profile, String> {
    let mut p = profile;
    p.updated_at = Utc::now();
    let repo = ProfileRepository::new(state.db.clone());
    repo.upsert(&p).map_err(|e| e.to_string())?;
    Ok(p)
}

#[tauri::command]
pub async fn delete_profile(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let repo = ProfileRepository::new(state.db.clone());
    repo.delete(&id).map_err(|e| e.to_string())
}

// ─── Sessions ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_profile(
    profile_id: String,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    if state.safe_mode {
        return Err("Safe Mode is active. Automation is disabled.".to_string());
    }
    if state.is_global_pause() {
        return Err("Global Pause is active. Confirm to override.".to_string());
    }

    let repo = ProfileRepository::new(state.db.clone());
    let profile = repo
        .get(&profile_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Profile not found: {}", profile_id))?;

    let adapter = WindowsNativeAdapter;

    // Capture pre-session snapshot so restore knows exactly what to revert
    let snapshot = adapter
        .snapshot_state(&profile.actions)
        .map_err(|e| e.to_string())?;

    let sm = SessionManager::new(state.db.clone());
    let session = sm
        .start_session(&profile, TriggerSource::ManualUi, snapshot, state.safe_mode)
        .map_err(|e| e.to_string())?;

    // Apply actions and collect results
    let applied = SnapshotManager::apply_actions(&adapter, &profile.actions, false);

    // Persist updated snapshot with actions_applied populated
    if let Some(original_snapshot) = &session.snapshot {
        let mut updated_snapshot = original_snapshot.clone();
        updated_snapshot.actions_applied = applied;
        if let Ok(snap_json) = serde_json::to_string(&updated_snapshot) {
            let conn = state.db.conn.lock();
            let _ = conn.execute(
                "UPDATE sessions SET snapshot = ?1 WHERE id = ?2",
                rusqlite::params![snap_json, session.id],
            );
        }
    }

    let mut active_id = state.active_session_id.write();
    *active_id = Some(session.id.clone());

    let logger = ActivityLogger::new(state.db.clone(), 60);
    let mut entry = ActivityLogger::make_entry(EventType::SessionStarted);
    entry.session_id = Some(session.id.clone());
    entry.profile_id = Some(profile_id);
    let _ = logger.log(entry);

    Ok(session)
}

#[tauri::command]
pub async fn stop_session(state: State<'_, AppState>) -> Result<(), String> {
    let sm = SessionManager::new(state.db.clone());
    if let Ok(Some(session)) = sm.get_active_session() {
        sm.end_session(&session.id).map_err(|e| e.to_string())?;

        let mut active_id = state.active_session_id.write();
        *active_id = None;

        let logger = ActivityLogger::new(state.db.clone(), 60);
        let mut entry = ActivityLogger::make_entry(EventType::SessionEnded);
        entry.session_id = Some(session.id);
        let _ = logger.log(entry);
    }
    Ok(())
}

#[tauri::command]
pub async fn restore_session(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let sm = SessionManager::new(state.db.clone());
    let session = sm
        .get_active_session()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No active session".to_string())?;

    let mut errors = vec![];
    if let Some(snapshot) = &session.snapshot {
        let adapter = WindowsNativeAdapter;
        match adapter.restore_snapshot(snapshot) {
            Ok(result) => errors.extend(result.failed),
            Err(e) => errors.push(e.to_string()),
        }
    }

    sm.end_session(&session.id).map_err(|e| e.to_string())?;

    let mut active_id = state.active_session_id.write();
    *active_id = None;

    let logger = ActivityLogger::new(state.db.clone(), 60);
    let mut entry = ActivityLogger::make_entry(EventType::SessionRestored);
    entry.session_id = Some(session.id);
    let _ = logger.log(entry);

    Ok(errors)
}

/// Restore system state from a specific stale session by ID, then mark it ended.
#[tauri::command]
pub async fn restore_stale_session(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let sm = SessionManager::new(state.db.clone());
    let stale_sessions = sm.get_stale_sessions().map_err(|e| e.to_string())?;
    let session = stale_sessions
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Stale session not found: {}", id))?;

    let mut errors = vec![];
    if let Some(snapshot) = &session.snapshot {
        let adapter = WindowsNativeAdapter;
        match adapter.restore_snapshot(snapshot) {
            Ok(result) => errors.extend(result.failed),
            Err(e) => errors.push(e.to_string()),
        }
    }

    sm.dismiss_stale_session(&session.id).map_err(|e| e.to_string())?;

    let logger = ActivityLogger::new(state.db.clone(), 60);
    let mut entry = ActivityLogger::make_entry(EventType::SessionRestored);
    entry.session_id = Some(session.id);
    let _ = logger.log(entry);

    Ok(errors)
}

#[tauri::command]
pub async fn dry_run_profile(
    profile_id: String,
    state: State<'_, AppState>,
) -> Result<DryRunResult, String> {
    let repo = ProfileRepository::new(state.db.clone());
    let profile = repo
        .get(&profile_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Profile not found: {}", profile_id))?;

    let adapter = WindowsNativeAdapter;
    let capability = adapter.check_power_capability();
    let mut warnings = vec![];
    let mut planned = vec![];

    for action in &profile.actions {
        let (feasible, reason) = match action {
            Action::SetPowerPlan { .. } => {
                if !capability.power_plans_supported {
                    (false, Some("Power plan switching not supported on this device (Modern Standby/OEM restriction)".to_string()))
                } else {
                    (true, None)
                }
            }
            Action::ReduceInterruptions => {
                (false, Some("Windows Focus Assist cannot be controlled automatically. Manual checklist will be shown.".to_string()))
            }
            Action::PreventSleep => {
                if adapter.is_battery_saver_active() {
                    warnings.push("Battery Saver is active and may override sleep prevention.".to_string());
                }
                (true, None)
            }
            _ => (true, None),
        };

        planned.push(PlannedAction {
            action: action.clone(),
            feasible,
            reason,
        });
    }

    Ok(DryRunResult {
        profile_id: profile.id.clone(),
        profile_name: profile.name.clone(),
        would_trigger: true,
        trigger_reason: Some("Manual dry-run".to_string()),
        planned_actions: planned,
        warnings,
    })
}

#[tauri::command]
pub async fn get_active_session(state: State<'_, AppState>) -> Result<Option<Session>, String> {
    let sm = SessionManager::new(state.db.clone());
    sm.get_active_session().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let conn = state.db.conn.lock();
    let mut stmt = conn
        .prepare(
            "SELECT id, profile_id, trigger_source, started_at, ended_at, status, snapshot, safe_mode
             FROM sessions ORDER BY started_at DESC LIMIT 100",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                profile_id: row.get(1)?,
                trigger_source: serde_json::from_str(&row.get::<_, String>(2)?)
                    .unwrap_or(TriggerSource::ManualUi),
                started_at: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| Utc::now()),
                ended_at: row.get::<_, Option<String>>(4)?.and_then(|s| s.parse().ok()),
                status: serde_json::from_str(&row.get::<_, String>(5)?)
                    .unwrap_or(SessionStatus::Ended),
                snapshot: row
                    .get::<_, Option<String>>(6)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                safe_mode: row.get::<_, i32>(7)? != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

#[tauri::command]
pub async fn get_stale_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let sm = SessionManager::new(state.db.clone());
    sm.get_stale_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dismiss_stale_session(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let sm = SessionManager::new(state.db.clone());
    sm.dismiss_stale_session(&id).map_err(|e| e.to_string())
}

// ─── Activity Logs ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_activity_logs(
    session_id: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<ActivityLog>, String> {
    let logger = ActivityLogger::new(state.db.clone(), 60);
    logger
        .get_logs(session_id.as_deref(), limit.unwrap_or(500))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_logs_json(state: State<'_, AppState>) -> Result<String, String> {
    let logger = ActivityLogger::new(state.db.clone(), usize::MAX);
    let logs = logger.get_logs(None, 10000).map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&logs).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_logs_csv(state: State<'_, AppState>) -> Result<String, String> {
    let logger = ActivityLogger::new(state.db.clone(), usize::MAX);
    let logs = logger.get_logs(None, 10000).map_err(|e| e.to_string())?;

    let mut out = String::from("id,session_id,profile_id,event_type,action_kind,result,failure_reason,created_at\n");
    for l in &logs {
        out.push_str(&format!(
            "{},{},{},{:?},{},{},{},{}\n",
            l.id,
            l.session_id.as_deref().unwrap_or(""),
            l.profile_id.as_deref().unwrap_or(""),
            l.event_type,
            l.action_kind.as_deref().unwrap_or(""),
            l.result,
            l.failure_reason.as_deref().unwrap_or(""),
            l.created_at.to_rfc3339(),
        ));
    }
    Ok(out)
}

// ─── Profile Import/Export ────────────────────────────────────────────────────

#[tauri::command]
pub async fn export_profile_json(
    profile_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let repo = ProfileRepository::new(state.db.clone());
    let profile = repo
        .get(&profile_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Profile not found".to_string())?;
    serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_profile_json(json: String, state: State<'_, AppState>) -> Result<Profile, String> {
    let mut profile: Profile = serde_json::from_str(&json).map_err(|e| format!("Invalid profile JSON: {}", e))?;
    profile.is_builtin = false;
    profile.updated_at = Utc::now();
    let repo = ProfileRepository::new(state.db.clone());
    repo.upsert(&profile).map_err(|e| e.to_string())?;
    Ok(profile)
}

// ─── System Status ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_system_status(_state: State<'_, AppState>) -> Result<SystemStatus, String> {
    let adapter = WindowsNativeAdapter;
    adapter.read_system_status().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_top_offenders(_state: State<'_, AppState>) -> Result<(Vec<ProcessInfo>, Vec<ProcessInfo>), String> {
    let adapter = WindowsNativeAdapter;
    let status = adapter.read_system_status().map_err(|e| e.to_string())?;
    Ok((status.top_cpu_offenders, status.top_ram_offenders))
}

// ─── Settings ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(state.settings.read().clone())
}

#[tauri::command]
pub async fn update_settings(settings: Settings, state: State<'_, AppState>) -> Result<(), String> {
    let json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;
    let conn = state.db.conn.lock();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('app_settings', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![json],
    )
    .map_err(|e| e.to_string())?;
    drop(conn);
    *state.settings.write() = settings;
    Ok(())
}

#[tauri::command]
pub async fn toggle_global_pause(state: State<'_, AppState>) -> Result<bool, String> {
    // Perform the toggle and serialize within the write lock to avoid TOCTOU
    // between another concurrent toggle and the DB write.
    let new_val;
    let json;
    {
        let mut settings = state.settings.write();
        settings.global_pause = !settings.global_pause;
        new_val = settings.global_pause;
        json = serde_json::to_string(&*settings).map_err(|e| e.to_string())?;
    }

    let conn = state.db.conn.lock();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('app_settings', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![json],
    )
    .map_err(|e| e.to_string())?;

    Ok(new_val)
}

// ─── Approved Apps ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_approved_apps(state: State<'_, AppState>) -> Result<Vec<ApprovedApp>, String> {
    let conn = state.db.conn.lock();
    let mut stmt = conn
        .prepare("SELECT id, name, process_name, action, profile_ids, created_at FROM approved_apps")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ApprovedApp {
                id: row.get(0)?,
                name: row.get(1)?,
                process_name: row.get(2)?,
                action: serde_json::from_str(&row.get::<_, String>(3)?)
                    .unwrap_or(AppControlAction::Pause),
                profile_ids: serde_json::from_str(&row.get::<_, String>(4)?)
                    .unwrap_or_default(),
                created_at: row.get::<_, String>(5)?.parse().unwrap_or_else(|_| Utc::now()),
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

#[tauri::command]
pub async fn upsert_approved_app(app: ApprovedApp, state: State<'_, AppState>) -> Result<ApprovedApp, String> {
    let mut a = app;
    if a.id.is_empty() {
        a.id = Uuid::new_v4().to_string();
        a.created_at = Utc::now();
    }
    let conn = state.db.conn.lock();
    conn.execute(
        "INSERT INTO approved_apps (id, name, process_name, action, profile_ids, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
           process_name = excluded.process_name,
           action = excluded.action,
           profile_ids = excluded.profile_ids",
        rusqlite::params![
            a.id,
            a.name,
            a.process_name,
            serde_json::to_string(&a.action).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            serde_json::to_string(&a.profile_ids).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
            a.created_at.to_rfc3339(),
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(a)
}

#[tauri::command]
pub async fn remove_approved_app(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.conn.lock();
    conn.execute("DELETE FROM approved_apps WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
