use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub mode_type: ModeType,
    pub triggers: Vec<Trigger>,
    pub actions: Vec<Action>,
    pub restore_policy: RestorePolicy,
    pub approval_mode: ApprovalMode,
    pub is_builtin: bool,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Profile {
    pub fn new_id() -> String {
        Uuid::new_v4().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModeType {
    Render,
    Studio,
    Build,
    Game,
    Design,
    Recording,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Trigger {
    AppRunning { app_id: String },
    ProcessRunning { process_name: String },
    CpuAbove {
        process_name: Option<String>,
        percent: f64,
        duration_seconds: u64,
    },
    MemoryAbove {
        process_name: Option<String>,
        mb: u64,
        duration_seconds: u64,
    },
    ManualCli { profile_id: String },
    ManualUi { profile_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    PreventSleep,
    ReduceInterruptions,
    SetPowerPlan { plan_id: String },
    PauseApprovedApp { app_id: String },
    WatchBattery { warn_below_percent: u8 },
    WatchMemory { warn_above_percent: u8 },
    WatchCpu { warn_above_percent: u8 },
    RunApprovedScript { script_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestorePolicy {
    OnAppQuit,
    OnResourceIdle,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalMode {
    AskFirst,
    AutomaticAfterTrusted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub profile_id: String,
    pub trigger_source: TriggerSource,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: SessionStatus,
    pub snapshot: Option<SystemSnapshot>,
    pub safe_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerSource {
    AppDetected,
    ProcessDetected,
    CpuThreshold,
    MemoryThreshold,
    ManualCli,
    ManualUi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Ended,
    Stale,
    Restoring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub power_plan_id: Option<String>,
    pub power_plan_name: Option<String>,
    pub sleep_prevention_active: bool,
    pub paused_apps: Vec<String>,
    pub actions_applied: Vec<AppliedAction>,
    pub captured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedAction {
    pub action: Action,
    pub result: ActionResult,
    pub applied_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionResult {
    Ok,
    UnsupportedCapability { reason: String },
    PermissionDenied { reason: String },
    Failed { reason: String },
    DryRun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: String,
    pub session_id: Option<String>,
    pub profile_id: Option<String>,
    pub event_type: EventType,
    pub action_kind: Option<String>,
    pub trigger_kind: Option<String>,
    pub result: String,
    pub failure_reason: Option<String>,
    pub restore_status: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    SessionStarted,
    SessionEnded,
    SessionRestored,
    ActionApplied,
    ActionFailed,
    TriggerDetected,
    TriggerSuppressed,
    ProfileActivated,
    ProfileDeactivated,
    GlobalPauseToggled,
    StaleSessionDetected,
    StaleSessionRecovered,
    SafeModeStarted,
    WarningRaised,
    ScriptExecuted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedApp {
    pub id: String,
    pub name: String,
    pub process_name: String,
    pub action: AppControlAction,
    pub profile_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppControlAction {
    Pause,
    Quit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub cpu_percent: f64,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub ram_percent: f64,
    pub battery_percent: Option<u8>,
    pub on_ac_power: bool,
    pub battery_saver_active: bool,
    pub power_plan_id: Option<String>,
    pub power_plan_name: Option<String>,
    pub power_plan_supported: bool,
    pub sleep_prevention_active: bool,
    pub top_cpu_offenders: Vec<ProcessInfo>,
    pub top_ram_offenders: Vec<ProcessInfo>,
    pub running_watched_processes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub ram_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub global_pause: bool,
    pub safe_mode: bool,
    pub monitoring_interval_ms: u64,
    pub log_max_entries: usize,
    pub log_rate_limit_per_minute: usize,
    pub default_approval_mode: ApprovalMode,
    pub battery_warn_percent: u8,
    pub cpu_warn_percent: u8,
    pub ram_warn_percent: u8,
    pub stall_alert_seconds: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            global_pause: false,
            safe_mode: false,
            monitoring_interval_ms: 5000,
            log_max_entries: 10000,
            log_rate_limit_per_minute: 60,
            default_approval_mode: ApprovalMode::AskFirst,
            battery_warn_percent: 20,
            cpu_warn_percent: 85,
            ram_warn_percent: 85,
            stall_alert_seconds: 120,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub mode: String,
    pub pressure: String,
    pub ram: u8,
    pub cpu: u8,
    pub battery: Option<u8>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryRunResult {
    pub profile_id: String,
    pub profile_name: String,
    pub would_trigger: bool,
    pub trigger_reason: Option<String>,
    pub planned_actions: Vec<PlannedAction>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAction {
    pub action: Action,
    pub feasible: bool,
    pub reason: Option<String>,
}
