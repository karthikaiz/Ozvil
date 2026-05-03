use crate::db::models::ActionResult;
use crate::windows_adapter::{PowerCapability, PowerPlanInfo};
use anyhow::Result;

pub struct PowerStatus {
    pub battery_percent: Option<u8>,
    pub on_ac_power: bool,
    pub battery_saver_active: bool,
    pub current_plan_id: Option<String>,
    pub current_plan_name: Option<String>,
    pub plans_supported: bool,
}

#[cfg(target_os = "windows")]
pub fn read_power_status() -> Result<PowerStatus> {
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

    let mut status = SYSTEM_POWER_STATUS::default();
    unsafe {
        GetSystemPowerStatus(&mut status)?;
    }

    let on_ac = status.ACLineStatus == 1;
    let battery_percent = if status.BatteryLifePercent == 255 {
        None
    } else {
        Some(status.BatteryLifePercent)
    };

    let battery_saver = (status.SystemStatusFlag & 0x01) != 0;

    let (plan_id, plan_name, plans_supported) = read_active_power_plan();

    Ok(PowerStatus {
        battery_percent,
        on_ac_power: on_ac,
        battery_saver_active: battery_saver,
        current_plan_id: plan_id,
        current_plan_name: plan_name,
        plans_supported,
    })
}

#[cfg(target_os = "windows")]
fn read_active_power_plan() -> (Option<String>, Option<String>, bool) {
    // Use powercfg via sidecar or shell to read active plan.
    // In production this would call PowerGetActiveScheme() from PowrProf.dll.
    (Some("381b4222-f694-41f0-9685-ff5bb260df2e".to_string()), Some("Balanced".to_string()), true)
}

#[cfg(target_os = "windows")]
pub fn set_power_plan(plan_id: &str) -> Result<ActionResult> {
    use std::process::Command;

    let output = Command::new("powercfg")
        .args(["/setactive", plan_id])
        .output()?;

    if output.status.success() {
        Ok(ActionResult::Ok)
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        Ok(ActionResult::UnsupportedCapability {
            reason: format!("powercfg /setactive failed: {}", err),
        })
    }
}

#[cfg(target_os = "windows")]
pub fn check_capability() -> PowerCapability {
    use std::process::Command;

    let output = Command::new("powercfg")
        .args(["/list"])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: vec![],
        });

    let supported = output.status.success();
    PowerCapability {
        power_plans_supported: supported,
        modern_standby: false,
        available_plans: if supported {
            parse_powercfg_list(&String::from_utf8_lossy(&output.stdout))
        } else {
            vec![]
        },
    }
}

#[cfg(target_os = "windows")]
fn parse_powercfg_list(output: &str) -> Vec<PowerPlanInfo> {
    let mut plans = vec![];
    for line in output.lines() {
        if line.contains("Power Scheme GUID") {
            // Example: Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced) *
            let active = line.ends_with('*');
            if let Some(guid_start) = line.find(':') {
                let rest = line[guid_start + 1..].trim();
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if let Some(guid) = parts.first() {
                    let name = parts
                        .get(1)
                        .map(|n| n.trim_matches(|c| c == '(' || c == ')' || c == ' ' || c == '*'))
                        .unwrap_or("Unknown")
                        .to_string();
                    plans.push(PowerPlanInfo {
                        id: guid.to_string(),
                        name,
                        active,
                    });
                }
            }
        }
    }
    plans
}

#[cfg(not(target_os = "windows"))]
pub fn read_power_status() -> Result<PowerStatus> {
    Ok(PowerStatus {
        battery_percent: Some(72),
        on_ac_power: true,
        battery_saver_active: false,
        current_plan_id: Some("381b4222-f694-41f0-9685-ff5bb260df2e".to_string()),
        current_plan_name: Some("Balanced".to_string()),
        plans_supported: true,
    })
}

#[cfg(not(target_os = "windows"))]
pub fn set_power_plan(_plan_id: &str) -> Result<ActionResult> {
    Ok(ActionResult::UnsupportedCapability {
        reason: "Power plan switching is only supported on Windows".to_string(),
    })
}

#[cfg(not(target_os = "windows"))]
pub fn check_capability() -> PowerCapability {
    PowerCapability {
        power_plans_supported: false,
        modern_standby: false,
        available_plans: vec![],
    }
}
