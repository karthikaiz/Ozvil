import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Save, Shield, Clock, Database, Bell, AlertTriangle } from "lucide-react";
import type { Settings } from "../types";
import { useAppContext } from "../hooks/useAppContext";

export default function SettingsPage() {
  const { settings, refreshSettings, appInfo, toggleGlobalPause } = useAppContext();
  const [draft, setDraft] = useState<Settings | null>(null);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (settings) setDraft({ ...settings });
  }, [settings]);

  const handleSave = async () => {
    if (!draft) return;
    setSaving(true);
    try {
      await invoke("update_settings", { settings: draft });
      refreshSettings();
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e: any) {
      alert(`Save failed: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  if (!draft) return <div className="empty-state">Loading settings…</div>;

  return (
    <div style={{ height: "100%", overflow: "hidden", display: "flex", flexDirection: "column" }}>
      {/* Header */}
      <div
        style={{
          padding: "20px 24px 16px",
          borderBottom: "1px solid var(--border)",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          flexShrink: 0,
        }}
      >
        <div>
          <h1 style={{ fontSize: 18, fontWeight: 700, marginBottom: 2 }}>Settings</h1>
          <p style={{ fontSize: 12, color: "var(--text-muted)" }}>
            Ozvil v{appInfo?.version ?? "—"} · Windows-only v1
          </p>
        </div>
        <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
          <Save size={13} /> {saving ? "Saving…" : saved ? "Saved ✓" : "Save Changes"}
        </button>
      </div>

      <div className="scroll-area" style={{ padding: 24 }}>
        {/* Safe Mode status */}
        {appInfo?.safe_mode && (
          <div
            className="card"
            style={{
              marginBottom: 24,
              borderColor: "rgba(168,85,247,0.3)",
              background: "rgba(168,85,247,0.07)",
              display: "flex",
              alignItems: "center",
              gap: 10,
            }}
          >
            <Shield size={16} color="#a855f7" />
            <div>
              <div style={{ fontWeight: 600, color: "#a855f7", marginBottom: 2 }}>
                Safe Mode Active
              </div>
              <div style={{ fontSize: 12, color: "var(--text-secondary)" }}>
                Automation is fully disabled. No profiles, scripts, app pauses, or power
                actions will run. Restart Ozvil normally to re-enable automation.
              </div>
            </div>
          </div>
        )}

        {/* Global Pause */}
        <SettingSection icon={<Bell size={15} />} title="Automation">
          <SettingRow
            label="Global Pause"
            description="Pause all automatic profile activation. Emergency restore is still available."
          >
            <Toggle
              value={draft.global_pause}
              onChange={(v) => {
                setDraft({ ...draft, global_pause: v });
                toggleGlobalPause();
              }}
            />
          </SettingRow>
          <SettingRow
            label="Default Approval Mode"
            description="How Ozvil asks before activating a profile for the first time."
          >
            <select
              className="input"
              style={{ width: 220 }}
              value={draft.default_approval_mode}
              onChange={(e) => setDraft({ ...draft, default_approval_mode: e.target.value as any })}
            >
              <option value="ask_first">Ask First (recommended)</option>
              <option value="automatic_after_trusted">Automatic After Trusted</option>
            </select>
          </SettingRow>
        </SettingSection>

        {/* Monitoring */}
        <SettingSection icon={<Clock size={15} />} title="Monitoring">
          <SettingRow
            label="Base Poll Interval"
            description="How often Ozvil checks system state when idle. Speeds up automatically during active sessions."
          >
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <input
                className="input"
                type="number"
                style={{ width: 90 }}
                value={draft.monitoring_interval_ms / 1000}
                min={2}
                max={60}
                onChange={(e) =>
                  setDraft({ ...draft, monitoring_interval_ms: Number(e.target.value) * 1000 })
                }
              />
              <span style={{ fontSize: 12, color: "var(--text-muted)" }}>seconds</span>
            </div>
          </SettingRow>
          <SettingRow
            label="Stall Alert Threshold"
            description="Alert if an approved local AI or dev process shows near-zero CPU for this duration."
          >
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <input
                className="input"
                type="number"
                style={{ width: 90 }}
                value={draft.stall_alert_seconds}
                min={30}
                max={600}
                onChange={(e) =>
                  setDraft({ ...draft, stall_alert_seconds: Number(e.target.value) })
                }
              />
              <span style={{ fontSize: 12, color: "var(--text-muted)" }}>seconds</span>
            </div>
          </SettingRow>
        </SettingSection>

        {/* Warning thresholds */}
        <SettingSection icon={<AlertTriangle size={15} />} title="Warning Thresholds">
          <SettingRow label="CPU Warning" description="Warn when system CPU usage exceeds this level.">
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <input
                className="input"
                type="number"
                style={{ width: 70 }}
                value={draft.cpu_warn_percent}
                min={50} max={100}
                onChange={(e) => setDraft({ ...draft, cpu_warn_percent: Number(e.target.value) })}
              />
              <span style={{ fontSize: 12, color: "var(--text-muted)" }}>%</span>
            </div>
          </SettingRow>
          <SettingRow label="RAM Warning" description="Warn when RAM pressure exceeds this level.">
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <input
                className="input"
                type="number"
                style={{ width: 70 }}
                value={draft.ram_warn_percent}
                min={50} max={100}
                onChange={(e) => setDraft({ ...draft, ram_warn_percent: Number(e.target.value) })}
              />
              <span style={{ fontSize: 12, color: "var(--text-muted)" }}>%</span>
            </div>
          </SettingRow>
          <SettingRow label="Battery Warning" description="Warn when battery drops below this level.">
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <input
                className="input"
                type="number"
                style={{ width: 70 }}
                value={draft.battery_warn_percent}
                min={5} max={50}
                onChange={(e) => setDraft({ ...draft, battery_warn_percent: Number(e.target.value) })}
              />
              <span style={{ fontSize: 12, color: "var(--text-muted)" }}>%</span>
            </div>
          </SettingRow>
        </SettingSection>

        {/* Logs */}
        <SettingSection icon={<Database size={15} />} title="Activity Log Retention">
          <SettingRow
            label="Max Log Entries"
            description="Older entries are pruned automatically to keep the database small."
          >
            <input
              className="input"
              type="number"
              style={{ width: 100 }}
              value={draft.log_max_entries}
              min={100}
              max={100000}
              step={100}
              onChange={(e) => setDraft({ ...draft, log_max_entries: Number(e.target.value) })}
            />
          </SettingRow>
          <SettingRow
            label="Log Rate Limit"
            description="Maximum log entries written per minute to prevent SQLite bloat during heavy events."
          >
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <input
                className="input"
                type="number"
                style={{ width: 80 }}
                value={draft.log_rate_limit_per_minute}
                min={10}
                max={600}
                onChange={(e) => setDraft({ ...draft, log_rate_limit_per_minute: Number(e.target.value) })}
              />
              <span style={{ fontSize: 12, color: "var(--text-muted)" }}>/ min</span>
            </div>
          </SettingRow>
        </SettingSection>

        {/* About */}
        <SettingSection icon={<Shield size={15} />} title="About Ozvil">
          <div
            style={{
              fontSize: 12,
              color: "var(--text-secondary)",
              lineHeight: 1.7,
              padding: "4px 0",
            }}
          >
            <p style={{ marginBottom: 6 }}>
              <strong style={{ color: "var(--text-primary)" }}>Core promise:</strong> When heavy work starts,
              Ozvil clears the runway for your Windows PC, protects the session, and restores everything afterward.
            </p>
            <p style={{ marginBottom: 6 }}>
              Ozvil makes <strong style={{ color: "var(--text-primary)" }}>visible, reversible, user-approved</strong>{" "}
              system changes. Every action is logged. Every change has a restore path.
            </p>
            <p style={{ color: "var(--text-muted)" }}>
              No registry cleaning · No hidden process killing · No default telemetry · No kernel drivers
            </p>
          </div>
        </SettingSection>
      </div>
    </div>
  );
}

function SettingSection({
  icon,
  title,
  children,
}: {
  icon: React.ReactNode;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div style={{ marginBottom: 28 }}>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 7,
          marginBottom: 14,
          paddingBottom: 8,
          borderBottom: "1px solid var(--border)",
          color: "var(--text-secondary)",
          fontWeight: 600,
          fontSize: 13,
        }}
      >
        {icon} {title}
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
        {children}
      </div>
    </div>
  );
}

function SettingRow({
  label,
  description,
  children,
}: {
  label: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 20 }}>
      <div style={{ flex: 1 }}>
        <div style={{ fontWeight: 500, fontSize: 13, marginBottom: 2 }}>{label}</div>
        <div style={{ fontSize: 12, color: "var(--text-muted)" }}>{description}</div>
      </div>
      <div style={{ flexShrink: 0 }}>{children}</div>
    </div>
  );
}

function Toggle({ value, onChange }: { value: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!value)}
      style={{
        width: 40,
        height: 22,
        borderRadius: 11,
        border: "none",
        background: value ? "var(--accent)" : "var(--bg-hover)",
        position: "relative",
        cursor: "pointer",
        transition: "background 0.2s",
      }}
    >
      <div
        style={{
          width: 16,
          height: 16,
          borderRadius: "50%",
          background: "#fff",
          position: "absolute",
          top: 3,
          left: value ? 21 : 3,
          transition: "left 0.2s",
        }}
      />
    </button>
  );
}
