// ─── Core Models (mirror Rust types) ─────────────────────────────────────────

export type ModeType =
  | "render"
  | "studio"
  | "build"
  | "game"
  | "design"
  | "recording"
  | "custom";

export type RestorePolicy = "on_app_quit" | "on_resource_idle" | "manual";
export type ApprovalMode = "ask_first" | "automatic_after_trusted";
export type SessionStatus = "active" | "ended" | "stale" | "restoring";
export type TriggerSource =
  | "app_detected"
  | "process_detected"
  | "cpu_threshold"
  | "memory_threshold"
  | "manual_cli"
  | "manual_ui";

export type Trigger =
  | { kind: "app_running"; app_id: string }
  | { kind: "process_running"; process_name: string }
  | { kind: "cpu_above"; process_name?: string; percent: number; duration_seconds: number }
  | { kind: "memory_above"; process_name?: string; mb: number; duration_seconds: number }
  | { kind: "manual_cli"; profile_id: string }
  | { kind: "manual_ui"; profile_id: string };

export type Action =
  | { kind: "prevent_sleep" }
  | { kind: "reduce_interruptions" }
  | { kind: "set_power_plan"; plan_id: string }
  | { kind: "pause_approved_app"; app_id: string }
  | { kind: "watch_battery"; warn_below_percent: number }
  | { kind: "watch_memory"; warn_above_percent: number }
  | { kind: "watch_cpu"; warn_above_percent: number }
  | { kind: "run_approved_script"; script_id: string };

export type ActionResult =
  | { type: "ok" }
  | { type: "unsupported_capability"; reason: string }
  | { type: "permission_denied"; reason: string }
  | { type: "failed"; reason: string }
  | { type: "dry_run" };

export interface AppliedAction {
  action: Action;
  result: ActionResult;
  applied_at: string;
}

export interface SystemSnapshot {
  power_plan_id: string | null;
  power_plan_name: string | null;
  sleep_prevention_active: boolean;
  paused_apps: string[];
  actions_applied: AppliedAction[];
  captured_at: string;
}

export interface Profile {
  id: string;
  name: string;
  mode_type: ModeType;
  triggers: Trigger[];
  actions: Action[];
  restore_policy: RestorePolicy;
  approval_mode: ApprovalMode;
  is_builtin: boolean;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface Session {
  id: string;
  profile_id: string;
  trigger_source: TriggerSource;
  started_at: string;
  ended_at: string | null;
  status: SessionStatus;
  snapshot: SystemSnapshot | null;
  safe_mode: boolean;
}

export interface ProcessInfo {
  pid: number;
  name: string;
  cpu_percent: number;
  ram_mb: number;
}

export interface SystemStatus {
  cpu_percent: number;
  ram_used_mb: number;
  ram_total_mb: number;
  ram_percent: number;
  battery_percent: number | null;
  on_ac_power: boolean;
  battery_saver_active: boolean;
  power_plan_id: string | null;
  power_plan_name: string | null;
  power_plan_supported: boolean;
  sleep_prevention_active: boolean;
  top_cpu_offenders: ProcessInfo[];
  top_ram_offenders: ProcessInfo[];
  running_watched_processes: string[];
}

export interface ActivityLog {
  id: string;
  session_id: string | null;
  profile_id: string | null;
  event_type: string;
  action_kind: string | null;
  trigger_kind: string | null;
  result: string;
  failure_reason: string | null;
  restore_status: string | null;
  metadata: Record<string, unknown> | null;
  created_at: string;
}

export interface Settings {
  global_pause: boolean;
  safe_mode: boolean;
  monitoring_interval_ms: number;
  log_max_entries: number;
  log_rate_limit_per_minute: number;
  default_approval_mode: ApprovalMode;
  battery_warn_percent: number;
  cpu_warn_percent: number;
  ram_warn_percent: number;
  stall_alert_seconds: number;
}

export interface ApprovedApp {
  id: string;
  name: string;
  process_name: string;
  action: "pause" | "quit";
  profile_ids: string[];
  created_at: string;
}

export interface AppStateInfo {
  safe_mode: boolean;
  global_pause: boolean;
  version: string;
}

export interface DryRunResult {
  profile_id: string;
  profile_name: string;
  would_trigger: boolean;
  trigger_reason: string | null;
  planned_actions: PlannedAction[];
  warnings: string[];
}

export interface PlannedAction {
  action: Action;
  feasible: boolean;
  reason: string | null;
}

export interface AgentStatus {
  mode: string;
  pressure: "low" | "medium" | "high";
  ram: number;
  cpu: number;
  battery: number | null;
  recommendation: string;
}

// ─── UI Helpers ───────────────────────────────────────────────────────────────

export const MODE_LABELS: Record<ModeType, string> = {
  render: "Render Mode",
  studio: "Studio Mode",
  build: "Build Mode",
  game: "Game Mode Plus",
  design: "Design Mode",
  recording: "Recording Mode",
  custom: "Custom",
};

export const MODE_COLORS: Record<ModeType, string> = {
  render:    "#f97316",
  studio:    "#a855f7",
  build:     "#3b82f6",
  game:      "#22c55e",
  design:    "#ec4899",
  recording: "#ef4444",
  custom:    "#64748b",
};

export const MODE_ICONS: Record<ModeType, string> = {
  render:    "🎬",
  studio:    "🎵",
  build:     "⚙️",
  game:      "🎮",
  design:    "🎨",
  recording: "📹",
  custom:    "⚡",
};
