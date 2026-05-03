use crate::db::models::{
    Action, ActionResult, AppliedAction, SystemSnapshot,
};
use crate::windows_adapter::WindowsAdapter;
use anyhow::Result;
use chrono::Utc;

pub struct SnapshotManager;

impl SnapshotManager {
    /// Capture the current system state BEFORE actions are applied so that
    /// restore knows exactly what to revert. `sleep_prevention_active` and
    /// `paused_apps` reflect the real pre-session state, not an empty default.
    pub fn capture(adapter: &dyn WindowsAdapter, _actions: &[Action]) -> Result<SystemSnapshot> {
        let status = adapter.read_system_status()?;

        let snapshot = SystemSnapshot {
            power_plan_id: status.power_plan_id.clone(),
            power_plan_name: status.power_plan_name.clone(),
            sleep_prevention_active: status.sleep_prevention_active,
            paused_apps: vec![],
            actions_applied: vec![],
            captured_at: Utc::now(),
        };

        Ok(snapshot)
    }

    pub fn apply_actions(
        adapter: &dyn WindowsAdapter,
        actions: &[Action],
        dry_run: bool,
    ) -> Vec<AppliedAction> {
        let mut applied = vec![];

        for action in actions {
            let result = if dry_run {
                ActionResult::DryRun
            } else {
                match adapter.apply_action(action) {
                    Ok(r) => r,
                    Err(e) => ActionResult::Failed {
                        reason: e.to_string(),
                    },
                }
            };

            applied.push(AppliedAction {
                action: action.clone(),
                result,
                applied_at: Utc::now(),
            });
        }

        applied
    }

    pub fn restore(adapter: &dyn WindowsAdapter, snapshot: &SystemSnapshot) -> Result<Vec<String>> {
        let mut errors = vec![];

        let restore_result = adapter.restore_snapshot(snapshot);
        if let Err(e) = restore_result {
            errors.push(format!("restore failed: {}", e));
        }

        Ok(errors)
    }
}
