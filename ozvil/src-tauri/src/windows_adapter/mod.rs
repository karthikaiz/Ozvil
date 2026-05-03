pub mod app_control_provider;
pub mod notification_provider;
pub mod performance_provider;
pub mod power_provider;
pub mod process_provider;
pub mod sleep_provider;

use crate::db::models::{Action, ActionResult, ProcessInfo, SystemSnapshot, SystemStatus};
use anyhow::Result;

pub trait WindowsAdapter: Send + Sync {
    fn list_processes(&self) -> Result<Vec<ProcessInfo>>;
    fn read_system_status(&self) -> Result<SystemStatus>;
    fn snapshot_state(&self, actions: &[Action]) -> Result<SystemSnapshot>;
    fn apply_action(&self, action: &Action) -> Result<ActionResult>;
    fn restore_snapshot(&self, snapshot: &SystemSnapshot) -> Result<RestoreResult>;
    fn check_power_capability(&self) -> PowerCapability;
    fn is_battery_saver_active(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct RestoreResult {
    pub restored: Vec<String>,
    pub failed: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PowerCapability {
    pub power_plans_supported: bool,
    pub modern_standby: bool,
    pub available_plans: Vec<PowerPlanInfo>,
}

#[derive(Debug, Clone)]
pub struct PowerPlanInfo {
    pub id: String,
    pub name: String,
    pub active: bool,
}

/// Production Windows adapter
pub struct WindowsNativeAdapter;

impl WindowsAdapter for WindowsNativeAdapter {
    fn list_processes(&self) -> Result<Vec<ProcessInfo>> {
        process_provider::list_processes()
    }

    fn read_system_status(&self) -> Result<SystemStatus> {
        let processes = self.list_processes()?;
        let perf = performance_provider::read_performance()?;
        let power = power_provider::read_power_status()?;

        let mut by_cpu = processes.clone();
        by_cpu.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());
        let mut by_ram = processes.clone();
        by_ram.sort_by(|a, b| b.ram_mb.cmp(&a.ram_mb));

        Ok(SystemStatus {
            cpu_percent: perf.cpu_percent,
            ram_used_mb: perf.ram_used_mb,
            ram_total_mb: perf.ram_total_mb,
            ram_percent: perf.ram_used_mb as f64 / perf.ram_total_mb.max(1) as f64 * 100.0,
            battery_percent: power.battery_percent,
            on_ac_power: power.on_ac_power,
            battery_saver_active: power.battery_saver_active,
            power_plan_id: power.current_plan_id,
            power_plan_name: power.current_plan_name,
            power_plan_supported: power.plans_supported,
            sleep_prevention_active: sleep_provider::is_sleep_prevented(),
            top_cpu_offenders: by_cpu.into_iter().take(5).collect(),
            top_ram_offenders: by_ram.into_iter().take(5).collect(),
            running_watched_processes: processes.iter().map(|p| p.name.clone()).collect(),
        })
    }

    fn snapshot_state(&self, _actions: &[Action]) -> Result<SystemSnapshot> {
        let status = self.read_system_status()?;
        Ok(SystemSnapshot {
            power_plan_id: status.power_plan_id,
            power_plan_name: status.power_plan_name,
            sleep_prevention_active: status.sleep_prevention_active,
            paused_apps: vec![],
            actions_applied: vec![],
            captured_at: chrono::Utc::now(),
        })
    }

    fn apply_action(&self, action: &Action) -> Result<ActionResult> {
        use crate::db::models::Action::*;
        match action {
            PreventSleep => sleep_provider::prevent_sleep(),
            ReduceInterruptions => notification_provider::reduce_interruptions(),
            SetPowerPlan { plan_id } => power_provider::set_power_plan(plan_id),
            PauseApprovedApp { app_id } => app_control_provider::pause_app(app_id),
            WatchBattery { warn_below_percent: _ } => Ok(ActionResult::Ok),
            WatchMemory { warn_above_percent: _ } => Ok(ActionResult::Ok),
            WatchCpu { warn_above_percent: _ } => Ok(ActionResult::Ok),
            RunApprovedScript { script_id } => {
                Ok(ActionResult::Failed {
                    reason: format!("Script execution requires explicit approval. script_id={}", script_id),
                })
            }
        }
    }

    fn restore_snapshot(&self, snapshot: &SystemSnapshot) -> Result<RestoreResult> {
        let mut restored = vec![];
        let mut failed = vec![];

        if snapshot.sleep_prevention_active {
            match sleep_provider::release_sleep_prevention() {
                Ok(_) => restored.push("sleep_prevention".to_string()),
                Err(e) => failed.push(format!("sleep_prevention: {}", e)),
            }
        }

        if let Some(plan_id) = &snapshot.power_plan_id {
            match power_provider::set_power_plan(plan_id) {
                Ok(_) => restored.push(format!("power_plan:{}", plan_id)),
                Err(e) => failed.push(format!("power_plan: {}", e)),
            }
        }

        for app_id in &snapshot.paused_apps {
            match app_control_provider::resume_app(app_id) {
                Ok(_) => restored.push(format!("app:{}", app_id)),
                Err(e) => failed.push(format!("app:{}: {}", app_id, e)),
            }
        }

        Ok(RestoreResult { restored, failed })
    }

    fn check_power_capability(&self) -> PowerCapability {
        power_provider::check_capability()
    }

    fn is_battery_saver_active(&self) -> bool {
        power_provider::read_power_status()
            .map(|p| p.battery_saver_active)
            .unwrap_or(false)
    }
}
