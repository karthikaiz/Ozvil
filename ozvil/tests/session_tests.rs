/// Session Manager Tests
/// Run with: cargo test --package ozvil

#[cfg(test)]
mod session_tests {
    use ozvil_lib::core::session_manager::SessionManager;
    use ozvil_lib::db::models::*;
    use ozvil_lib::db::Database;
    use chrono::Utc;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn make_db() -> Arc<Database> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        Arc::new(Database::open(&path).unwrap())
    }

    fn make_profile(id: &str) -> Profile {
        Profile {
            id: id.to_string(),
            name: id.to_string(),
            mode_type: ModeType::Build,
            triggers: vec![],
            actions: vec![
                Action::PreventSleep,
                Action::WatchCpu { warn_above_percent: 85 },
            ],
            restore_policy: RestorePolicy::OnAppQuit,
            approval_mode: ApprovalMode::AskFirst,
            is_builtin: false,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_snapshot() -> SystemSnapshot {
        SystemSnapshot {
            power_plan_id: Some("381b4222-f694-41f0-9685-ff5bb260df2e".to_string()),
            power_plan_name: Some("Balanced".to_string()),
            sleep_prevention_active: true,
            paused_apps: vec!["OneDrive.exe".to_string()],
            actions_applied: vec![],
            captured_at: Utc::now(),
        }
    }

    #[test]
    fn start_and_retrieve_active_session() {
        let db = make_db();
        // Insert a dummy profile first (FK required)
        {
            let conn = db.conn.lock();
            conn.execute(
                "INSERT INTO profiles (id, name, mode_type, triggers, actions, restore_policy, approval_mode, is_builtin, enabled, created_at, updated_at)
                 VALUES ('p1', 'test', '\"build\"', '[]', '[]', '\"on_app_quit\"', '\"ask_first\"', 0, 1, ?, ?)",
                rusqlite::params![Utc::now().to_rfc3339(), Utc::now().to_rfc3339()],
            ).unwrap();
        }

        let sm = SessionManager::new(db.clone());
        let profile = make_profile("p1");
        let snapshot = make_snapshot();

        let session = sm
            .start_session(&profile, TriggerSource::ManualUi, snapshot, false)
            .expect("start_session failed");

        assert!(!session.id.is_empty());

        let retrieved = sm.get_active_session().unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, session.id);
    }

    #[test]
    fn end_session_clears_active() {
        let db = make_db();
        {
            let conn = db.conn.lock();
            conn.execute(
                "INSERT INTO profiles (id, name, mode_type, triggers, actions, restore_policy, approval_mode, is_builtin, enabled, created_at, updated_at)
                 VALUES ('p2', 'test', '\"build\"', '[]', '[]', '\"on_app_quit\"', '\"ask_first\"', 0, 1, ?, ?)",
                rusqlite::params![Utc::now().to_rfc3339(), Utc::now().to_rfc3339()],
            ).unwrap();
        }

        let sm = SessionManager::new(db.clone());
        let profile = make_profile("p2");
        let session = sm
            .start_session(&profile, TriggerSource::ManualUi, make_snapshot(), false)
            .unwrap();

        sm.end_session(&session.id).unwrap();

        let active = sm.get_active_session().unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn stale_sessions_appear_after_reconciliation() {
        let db = make_db();
        // Manually insert an "active" session that simulates an unclean exit
        {
            let conn = db.conn.lock();
            conn.execute(
                "INSERT INTO profiles (id, name, mode_type, triggers, actions, restore_policy, approval_mode, is_builtin, enabled, created_at, updated_at)
                 VALUES ('p3', 'test', '\"build\"', '[]', '[]', '\"on_app_quit\"', '\"ask_first\"', 0, 1, ?, ?)",
                rusqlite::params![Utc::now().to_rfc3339(), Utc::now().to_rfc3339()],
            ).unwrap();

            conn.execute(
                "INSERT INTO sessions (id, profile_id, trigger_source, started_at, status, safe_mode)
                 VALUES ('stale-123', 'p3', '\"manual_ui\"', ?, 'stale', 0)",
                rusqlite::params![Utc::now().to_rfc3339()],
            ).unwrap();
        }

        let sm = SessionManager::new(db.clone());
        let stale = sm.get_stale_sessions().unwrap();
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].id, "stale-123");
    }

    #[test]
    fn dismiss_stale_session_marks_ended() {
        let db = make_db();
        {
            let conn = db.conn.lock();
            conn.execute(
                "INSERT INTO profiles (id, name, mode_type, triggers, actions, restore_policy, approval_mode, is_builtin, enabled, created_at, updated_at)
                 VALUES ('p4', 'test', '\"build\"', '[]', '[]', '\"on_app_quit\"', '\"ask_first\"', 0, 1, ?, ?)",
                rusqlite::params![Utc::now().to_rfc3339(), Utc::now().to_rfc3339()],
            ).unwrap();

            conn.execute(
                "INSERT INTO sessions (id, profile_id, trigger_source, started_at, status, safe_mode)
                 VALUES ('stale-456', 'p4', '\"manual_ui\"', ?, 'stale', 0)",
                rusqlite::params![Utc::now().to_rfc3339()],
            ).unwrap();
        }

        let sm = SessionManager::new(db.clone());
        sm.dismiss_stale_session("stale-456").unwrap();

        let stale = sm.get_stale_sessions().unwrap();
        assert!(stale.is_empty());
    }
}
