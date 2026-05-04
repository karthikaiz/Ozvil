use crate::db::models::{
    ModeType, Profile, Session, SessionStatus, SystemStatus, Trigger, TriggerSource,
};

#[derive(Debug, Clone)]
pub struct TriggerMatch {
    pub profile_id: String,
    pub trigger_source: TriggerSource,
    pub priority: TriggerPriority,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TriggerPriority {
    Manual = 4,
    RecordingStudio = 3,
    ForegroundApp = 2,
    BackgroundApp = 1,
    Threshold = 0,
}

pub fn evaluate_triggers(
    profiles: &[Profile],
    status: &SystemStatus,
    active_session: Option<&Session>,
    global_pause: bool,
    safe_mode: bool,
) -> Vec<TriggerMatch> {
    if safe_mode {
        return vec![];
    }

    // Only manual sessions are sticky — automatic sessions can be superseded by a
    // higher-priority trigger or replaced when their trigger condition goes away.
    if let Some(active) = active_session {
        let is_manual = matches!(
            active.trigger_source,
            TriggerSource::ManualCli | TriggerSource::ManualUi
        );
        if is_manual {
            return vec![];
        }
    }

    if global_pause {
        return vec![];
    }

    let mut matches: Vec<TriggerMatch> = vec![];

    for profile in profiles {
        if !profile.enabled {
            continue;
        }

        for trigger in &profile.triggers {
            let result = evaluate_trigger(trigger, status);
            if let Some(source) = result {
                let priority = compute_priority(&profile.mode_type, &source);
                matches.push(TriggerMatch {
                    profile_id: profile.id.clone(),
                    trigger_source: source,
                    priority,
                });
                break;
            }
        }
    }

    matches.sort_by(|a, b| b.priority.cmp(&a.priority));
    matches
}

fn evaluate_trigger(trigger: &Trigger, status: &SystemStatus) -> Option<TriggerSource> {
    match trigger {
        Trigger::AppRunning { app_id } => {
            if status
                .running_watched_processes
                .iter()
                .any(|p| p.to_lowercase() == app_id.to_lowercase())
            {
                Some(TriggerSource::AppDetected)
            } else {
                None
            }
        }
        Trigger::ProcessRunning { process_name } => {
            if status
                .running_watched_processes
                .iter()
                .any(|p| p.to_lowercase().contains(&process_name.to_lowercase()))
            {
                Some(TriggerSource::ProcessDetected)
            } else {
                None
            }
        }
        Trigger::CpuAbove { percent, .. } => {
            if status.cpu_percent >= *percent {
                Some(TriggerSource::CpuThreshold)
            } else {
                None
            }
        }
        Trigger::MemoryAbove { mb, .. } => {
            if status.ram_used_mb >= *mb {
                Some(TriggerSource::MemoryThreshold)
            } else {
                None
            }
        }
        Trigger::ManualCli { .. } => None,
        Trigger::ManualUi { .. } => None,
    }
}

fn compute_priority(mode: &ModeType, source: &TriggerSource) -> TriggerPriority {
    match source {
        TriggerSource::ManualCli | TriggerSource::ManualUi => TriggerPriority::Manual,
        TriggerSource::AppDetected | TriggerSource::ProcessDetected => {
            match mode {
                ModeType::Recording | ModeType::Studio => TriggerPriority::RecordingStudio,
                _ => TriggerPriority::ForegroundApp,
            }
        }
        TriggerSource::CpuThreshold | TriggerSource::MemoryThreshold => {
            TriggerPriority::Threshold
        }
    }
}

pub fn resolve_conflict(
    matches: &[TriggerMatch],
    active_session: Option<&Session>,
) -> Option<TriggerMatch> {
    if matches.is_empty() {
        return None;
    }

    // Active manual session takes precedence over all automatic triggers
    if let Some(session) = active_session {
        if matches!(
            session.trigger_source,
            TriggerSource::ManualCli | TriggerSource::ManualUi
        ) && matches!(session.status, SessionStatus::Active) {
            return None;
        }
    }

    matches.first().cloned()
}

pub fn build_pressure_label(cpu: f64, ram: f64) -> &'static str {
    let max = cpu.max(ram);
    if max >= 85.0 {
        "high"
    } else if max >= 60.0 {
        "medium"
    } else {
        "low"
    }
}

pub fn build_recommendation(status: &SystemStatus) -> String {
    if status.ram_percent >= 85.0 {
        "pause_background".to_string()
    } else if !status.on_ac_power && status.battery_percent.unwrap_or(100) < 20 {
        "check_power".to_string()
    } else if status.cpu_percent >= 85.0 {
        "reduce_load".to_string()
    } else {
        "none".to_string()
    }
}
