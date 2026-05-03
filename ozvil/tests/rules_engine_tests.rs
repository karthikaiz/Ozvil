/// Rules Engine Unit Tests
/// Run with: cargo test --package ozvil
///
/// These tests use a simulated Windows adapter fixture so no real OS calls are made.

#[cfg(test)]
mod rules_engine_tests {
    use ozvil_lib::core::rules_engine::{evaluate_triggers, resolve_conflict};
    use ozvil_lib::db::models::*;
    use chrono::Utc;

    fn make_profile(id: &str, mode: ModeType, triggers: Vec<Trigger>, enabled: bool) -> Profile {
        Profile {
            id: id.to_string(),
            name: id.to_string(),
            mode_type: mode,
            triggers,
            actions: vec![],
            restore_policy: RestorePolicy::OnAppQuit,
            approval_mode: ApprovalMode::AskFirst,
            is_builtin: false,
            enabled,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn idle_status() -> SystemStatus {
        SystemStatus {
            cpu_percent: 5.0,
            ram_used_mb: 4096,
            ram_total_mb: 16384,
            ram_percent: 25.0,
            battery_percent: Some(80),
            on_ac_power: true,
            battery_saver_active: false,
            power_plan_id: None,
            power_plan_name: None,
            power_plan_supported: true,
            sleep_prevention_active: false,
            top_cpu_offenders: vec![],
            top_ram_offenders: vec![],
            running_watched_processes: vec![],
        }
    }

    fn status_with_process(process: &str) -> SystemStatus {
        let mut s = idle_status();
        s.running_watched_processes = vec![process.to_string()];
        s
    }

    fn status_with_cpu(cpu: f64) -> SystemStatus {
        let mut s = idle_status();
        s.cpu_percent = cpu;
        s
    }

    // ── Trigger evaluation ────────────────────────────────────────────────────

    #[test]
    fn app_trigger_activates_matching_profile() {
        let profiles = vec![make_profile(
            "render",
            ModeType::Render,
            vec![Trigger::ProcessRunning { process_name: "blender.exe".to_string() }],
            true,
        )];
        let status = status_with_process("blender.exe");
        let matches = evaluate_triggers(&profiles, &status, None, false, false);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].profile_id, "render");
    }

    #[test]
    fn disabled_profile_never_activates() {
        let profiles = vec![make_profile(
            "render",
            ModeType::Render,
            vec![Trigger::ProcessRunning { process_name: "blender.exe".to_string() }],
            false, // disabled
        )];
        let status = status_with_process("blender.exe");
        let matches = evaluate_triggers(&profiles, &status, None, false, false);
        assert!(matches.is_empty());
    }

    #[test]
    fn global_pause_suppresses_new_activations() {
        let profiles = vec![make_profile(
            "build",
            ModeType::Build,
            vec![Trigger::ProcessRunning { process_name: "cargo.exe".to_string() }],
            true,
        )];
        let status = status_with_process("cargo.exe");
        let matches = evaluate_triggers(&profiles, &status, None, true, false);
        assert!(matches.is_empty(), "global pause must suppress triggers");
    }

    #[test]
    fn safe_mode_suppresses_all_activations() {
        let profiles = vec![make_profile(
            "build",
            ModeType::Build,
            vec![Trigger::ProcessRunning { process_name: "cargo.exe".to_string() }],
            true,
        )];
        let status = status_with_process("cargo.exe");
        let matches = evaluate_triggers(&profiles, &status, None, false, true);
        assert!(matches.is_empty(), "safe mode must suppress triggers");
    }

    #[test]
    fn cpu_trigger_activates_above_threshold() {
        let profiles = vec![make_profile(
            "build",
            ModeType::Build,
            vec![Trigger::CpuAbove {
                process_name: None,
                percent: 70.0,
                duration_seconds: 60,
            }],
            true,
        )];
        let status = status_with_cpu(85.0);
        let matches = evaluate_triggers(&profiles, &status, None, false, false);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn cpu_trigger_does_not_activate_below_threshold() {
        let profiles = vec![make_profile(
            "build",
            ModeType::Build,
            vec![Trigger::CpuAbove {
                process_name: None,
                percent: 70.0,
                duration_seconds: 60,
            }],
            true,
        )];
        let status = status_with_cpu(30.0);
        let matches = evaluate_triggers(&profiles, &status, None, false, false);
        assert!(matches.is_empty());
    }

    // ── Conflict resolution ────────────────────────────────────────────────────

    #[test]
    fn recording_studio_modes_beat_other_automatic_profiles() {
        let profiles = vec![
            make_profile(
                "build",
                ModeType::Build,
                vec![Trigger::ProcessRunning { process_name: "cargo.exe".to_string() }],
                true,
            ),
            make_profile(
                "studio",
                ModeType::Studio,
                vec![Trigger::ProcessRunning { process_name: "reaper.exe".to_string() }],
                true,
            ),
        ];
        let mut status = idle_status();
        status.running_watched_processes = vec!["cargo.exe".to_string(), "reaper.exe".to_string()];
        let matches = evaluate_triggers(&profiles, &status, None, false, false);
        let winner = resolve_conflict(&matches, None);
        assert!(winner.is_some());
        assert_eq!(winner.unwrap().profile_id, "studio");
    }

    #[test]
    fn no_match_returns_none() {
        let matches = evaluate_triggers(&[], &idle_status(), None, false, false);
        let winner = resolve_conflict(&matches, None);
        assert!(winner.is_none());
    }

    // ── Pressure labels ───────────────────────────────────────────────────────

    #[test]
    fn pressure_label_high_at_85_plus() {
        use ozvil_lib::core::rules_engine::build_pressure_label;
        assert_eq!(build_pressure_label(90.0, 40.0), "high");
    }

    #[test]
    fn pressure_label_medium_between_65_and_85() {
        use ozvil_lib::core::rules_engine::build_pressure_label;
        assert_eq!(build_pressure_label(70.0, 40.0), "medium");
    }

    #[test]
    fn pressure_label_low_below_65() {
        use ozvil_lib::core::rules_engine::build_pressure_label;
        assert_eq!(build_pressure_label(30.0, 20.0), "low");
    }
}
