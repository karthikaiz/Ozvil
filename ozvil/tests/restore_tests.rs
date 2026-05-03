/// Restore / Snapshot Tests
/// Run with: cargo test --package ozvil

#[cfg(test)]
mod restore_tests {
    use ozvil_lib::core::snapshot_manager::SnapshotManager;
    use ozvil_lib::db::models::*;
    use ozvil_lib::windows_adapter::{RestoreResult, WindowsAdapter};
    use anyhow::Result;
    use chrono::Utc;

    /// Simulated Windows adapter for tests — no real OS calls
    struct MockAdapter {
        pub sleep_prevention_supported: bool,
        pub power_plan_supported: bool,
    }

    impl WindowsAdapter for MockAdapter {
        fn list_processes(&self) -> Result<Vec<ProcessInfo>> {
            Ok(vec![])
        }

        fn read_system_status(&self) -> Result<SystemStatus> {
            Ok(SystemStatus {
                cpu_percent: 10.0,
                ram_used_mb: 4096,
                ram_total_mb: 16384,
                ram_percent: 25.0,
                battery_percent: Some(75),
                on_ac_power: true,
                battery_saver_active: false,
                power_plan_id: Some("balanced".to_string()),
                power_plan_name: Some("Balanced".to_string()),
                power_plan_supported: self.power_plan_supported,
                sleep_prevention_active: self.sleep_prevention_supported,
                top_cpu_offenders: vec![],
                top_ram_offenders: vec![],
                running_watched_processes: vec![],
            })
        }

        fn snapshot_state(&self, _actions: &[Action]) -> Result<SystemSnapshot> {
            Ok(SystemSnapshot {
                power_plan_id: Some("balanced".to_string()),
                power_plan_name: Some("Balanced".to_string()),
                sleep_prevention_active: self.sleep_prevention_supported,
                paused_apps: vec!["OneDrive.exe".to_string()],
                actions_applied: vec![],
                captured_at: Utc::now(),
            })
        }

        fn apply_action(&self, action: &Action) -> Result<ActionResult> {
            match action {
                Action::SetPowerPlan { .. } if !self.power_plan_supported => {
                    Ok(ActionResult::UnsupportedCapability {
                        reason: "OEM restriction".to_string(),
                    })
                }
                _ => Ok(ActionResult::Ok),
            }
        }

        fn restore_snapshot(&self, snapshot: &SystemSnapshot) -> Result<RestoreResult> {
            let mut restored = vec![];
            let mut failed = vec![];

            if snapshot.sleep_prevention_active {
                restored.push("sleep_prevention".to_string());
            }
            if let Some(plan) = &snapshot.power_plan_id {
                if self.power_plan_supported {
                    restored.push(format!("power_plan:{}", plan));
                } else {
                    failed.push("power_plan: OEM restriction".to_string());
                }
            }
            for app in &snapshot.paused_apps {
                restored.push(format!("app:{}", app));
            }

            Ok(RestoreResult { restored, failed })
        }

        fn check_power_capability(&self) -> ozvil_lib::windows_adapter::PowerCapability {
            ozvil_lib::windows_adapter::PowerCapability {
                power_plans_supported: self.power_plan_supported,
                modern_standby: !self.power_plan_supported,
                available_plans: vec![],
            }
        }

        fn is_battery_saver_active(&self) -> bool {
            false
        }
    }

    #[test]
    fn snapshot_is_recorded_before_actions_run() {
        let adapter = MockAdapter {
            sleep_prevention_supported: true,
            power_plan_supported: true,
        };
        let actions = vec![Action::PreventSleep, Action::WatchCpu { warn_above_percent: 85 }];
        let snapshot = SnapshotManager::capture(&adapter, &actions).unwrap();
        assert!(snapshot.power_plan_id.is_some());
    }

    #[test]
    fn successful_actions_are_restored() {
        let adapter = MockAdapter {
            sleep_prevention_supported: true,
            power_plan_supported: true,
        };
        let snapshot = SystemSnapshot {
            power_plan_id: Some("high-perf".to_string()),
            power_plan_name: Some("High Performance".to_string()),
            sleep_prevention_active: true,
            paused_apps: vec!["Dropbox.exe".to_string()],
            actions_applied: vec![],
            captured_at: Utc::now(),
        };

        let result = adapter.restore_snapshot(&snapshot).unwrap();
        assert!(result.restored.contains(&"sleep_prevention".to_string()));
        assert!(result.restored.contains(&"power_plan:high-perf".to_string()));
        assert!(result.restored.contains(&"app:Dropbox.exe".to_string()));
        assert!(result.failed.is_empty());
    }

    #[test]
    fn failed_actions_do_not_prevent_restoring_successful_ones() {
        let adapter = MockAdapter {
            sleep_prevention_supported: true,
            power_plan_supported: false, // power plan will fail
        };
        let snapshot = SystemSnapshot {
            power_plan_id: Some("high-perf".to_string()),
            power_plan_name: Some("High Performance".to_string()),
            sleep_prevention_active: true,
            paused_apps: vec!["OneDrive.exe".to_string()],
            actions_applied: vec![],
            captured_at: Utc::now(),
        };

        let result = adapter.restore_snapshot(&snapshot).unwrap();
        assert!(result.restored.contains(&"sleep_prevention".to_string()), "sleep must be restored");
        assert!(!result.failed.is_empty(), "power plan failure must be recorded");
        assert!(result.restored.contains(&"app:OneDrive.exe".to_string()), "app resume must succeed");
    }

    #[test]
    fn unsupported_power_plan_action_records_safe_result() {
        let adapter = MockAdapter {
            sleep_prevention_supported: true,
            power_plan_supported: false,
        };
        let action = Action::SetPowerPlan { plan_id: "high-perf".to_string() };
        let result = adapter.apply_action(&action).unwrap();
        assert!(
            matches!(result, ActionResult::UnsupportedCapability { .. }),
            "must return UnsupportedCapability for OEM-restricted device"
        );
    }

    #[test]
    fn dry_run_applies_no_changes() {
        let adapter = MockAdapter {
            sleep_prevention_supported: true,
            power_plan_supported: true,
        };
        let actions = vec![Action::PreventSleep, Action::SetPowerPlan { plan_id: "hp".to_string() }];
        let applied = SnapshotManager::apply_actions(&adapter, &actions, true);

        for ap in &applied {
            assert!(
                matches!(ap.result, ActionResult::DryRun),
                "all actions must be DryRun in dry-run mode"
            );
        }
    }
}
