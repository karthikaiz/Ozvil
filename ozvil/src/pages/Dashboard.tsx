import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import {
  Play,
  Square,
  RotateCcw,
  Zap,
  Battery,
  Cpu,
  MemoryStick,
  Clock,
  AlertTriangle,
  CheckCircle2,
  Moon,
} from "lucide-react";
import { useAppContext } from "../hooks/useAppContext";
import PressureBar from "../components/PressureBar";
import TopOffenderList from "../components/TopOffenderList";
import StatusBadge from "../components/StatusBadge";
import type { ModeType, Profile } from "../types";
import { MODE_LABELS, MODE_COLORS } from "../types";
import { formatDistanceToNow } from "date-fns";

export default function Dashboard() {
  const { appInfo, activeSession, systemStatus, profiles, refreshSession, refreshStatus } =
    useAppContext();
  const navigate = useNavigate();
  const [offenderTab, setOffenderTab] = useState<"cpu" | "ram">("cpu");
  const [staleSessions, setStaleSessions] = useState<any[]>([]);

  useEffect(() => {
    invoke<any[]>("get_stale_sessions")
      .then(setStaleSessions)
      .catch(() => {});
  }, []);

  const activeProfile = profiles?.find((p) => p.id === activeSession?.profile_id);

  const handleStart = async (profile: Profile) => {
    try {
      await invoke("start_profile", { profileId: profile.id });
      refreshSession();
    } catch (e: any) {
      alert(`Could not start profile: ${e}`);
    }
  };

  const handleStop = async () => {
    try {
      await invoke("stop_session");
      refreshSession();
    } catch {}
  };

  const handleRestore = async () => {
    try {
      const errors = await invoke<string[]>("restore_session");
      refreshSession();
      if (errors.length > 0) {
        alert(`Restore completed with warnings:\n${errors.join("\n")}`);
      }
    } catch (e: any) {
      alert(`Restore failed: ${e}`);
    }
  };

  const handleDismissStale = async (id: string) => {
    await invoke("dismiss_stale_session", { id });
    setStaleSessions((prev) => prev.filter((s) => s.id !== id));
  };

  return (
    <div
      style={{
        height: "100%",
        overflow: "hidden",
        display: "grid",
        gridTemplateColumns: "1fr 300px",
        gridTemplateRows: "1fr",
      }}
    >
      {/* Left column */}
      <div className="scroll-area" style={{ padding: 24 }}>
        {/* Page header */}
        <div style={{ marginBottom: 24 }}>
          <h1 style={{ fontSize: 20, fontWeight: 700, marginBottom: 4 }}>Dashboard</h1>
          <p style={{ color: "var(--text-secondary)", fontSize: 13 }}>
            {appInfo?.safe_mode
              ? "Running in Safe Mode — automation is disabled."
              : "Monitor your system and manage workload sessions."}
          </p>
        </div>

        {/* Stale session banner */}
        {staleSessions.length > 0 && (
          <div
            className="card"
            style={{
              marginBottom: 20,
              background: "rgba(245, 158, 11, 0.08)",
              border: "1px solid rgba(245, 158, 11, 0.3)",
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 10,
                marginBottom: 10,
              }}
            >
              <AlertTriangle size={16} color="var(--warning)" />
              <span style={{ fontWeight: 600, color: "var(--warning)" }}>
                Unfinished session detected
              </span>
            </div>
            <p style={{ fontSize: 13, color: "var(--text-secondary)", marginBottom: 12 }}>
              Ozvil was closed while a session was active. Your system may still have profile
              changes applied. Review and restore below.
            </p>
            <div style={{ display: "flex", gap: 8 }}>
              <button className="btn btn-primary" onClick={() => navigate("/restore")}>
                <RotateCcw size={14} /> Go to Restore Center
              </button>
              {staleSessions.map((s) => (
                <button
                  key={s.id}
                  className="btn btn-ghost"
                  onClick={() => handleDismissStale(s.id)}
                >
                  Dismiss
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Active session panel */}
        {activeSession && activeProfile ? (
          <ActiveSessionPanel
            profile={activeProfile}
            session={activeSession}
            onStop={handleStop}
            onRestore={handleRestore}
          />
        ) : (
          <div className="card" style={{ marginBottom: 20 }}>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                marginBottom: 16,
              }}
            >
              <div>
                <StatusBadge mode="idle" size="lg" />
                <p style={{ marginTop: 8, color: "var(--text-secondary)", fontSize: 13 }}>
                  No workload session active. Start a profile below or launch a watched app.
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Quick-start profiles */}
        <div style={{ marginBottom: 8 }}>
          <h2 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12, color: "var(--text-secondary)" }}>
            WORKLOAD PROFILES
          </h2>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))",
              gap: 10,
            }}
          >
            {(profiles ?? []).map((profile) => (
              <ProfileCard
                key={profile.id}
                profile={profile}
                isActive={activeSession?.profile_id === profile.id}
                onStart={() => handleStart(profile)}
                onDryRun={() => navigate(`/dry-run/${profile.id}`)}
              />
            ))}
          </div>
        </div>
      </div>

      {/* Right sidebar — live status */}
      <div
        style={{
          borderLeft: "1px solid var(--border)",
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
        }}
      >
        <div className="scroll-area" style={{ padding: 16 }}>
          <div
            style={{
              fontSize: 11,
              fontWeight: 600,
              color: "var(--text-muted)",
              letterSpacing: "0.5px",
              marginBottom: 14,
              textTransform: "uppercase",
            }}
          >
            System Status
          </div>

          {systemStatus ? (
            <>
              {/* CPU */}
              <div style={{ marginBottom: 14 }}>
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    marginBottom: 6,
                    color: "var(--text-secondary)",
                    fontSize: 12,
                  }}
                >
                  <Cpu size={13} /> CPU
                </div>
                <PressureBar value={systemStatus.cpu_percent} label="" />
              </div>

              {/* RAM */}
              <div style={{ marginBottom: 14 }}>
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    marginBottom: 6,
                    color: "var(--text-secondary)",
                    fontSize: 12,
                  }}
                >
                  <MemoryStick size={13} /> RAM
                </div>
                <PressureBar
                  value={systemStatus.ram_used_mb}
                  max={systemStatus.ram_total_mb}
                />
                <div
                  style={{
                    fontSize: 11,
                    color: "var(--text-muted)",
                    marginTop: 3,
                    textAlign: "right",
                  }}
                >
                  {(systemStatus.ram_used_mb / 1024).toFixed(1)} /{" "}
                  {(systemStatus.ram_total_mb / 1024).toFixed(1)} GB
                </div>
              </div>

              {/* Battery */}
              {systemStatus.battery_percent !== null && (
                <div style={{ marginBottom: 14 }}>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 6,
                      marginBottom: 6,
                      color: "var(--text-secondary)",
                      fontSize: 12,
                    }}
                  >
                    <Battery size={13} />
                    Battery
                    {systemStatus.on_ac_power && (
                      <span style={{ color: "var(--success)", fontSize: 10 }}>
                        ⚡ AC
                      </span>
                    )}
                  </div>
                  <PressureBar value={systemStatus.battery_percent} />
                  {systemStatus.battery_saver_active && (
                    <div
                      className="tag badge-warning"
                      style={{ marginTop: 6, fontSize: 10 }}
                    >
                      <AlertTriangle size={9} /> Battery Saver active — sleep prevention
                      may be overridden
                    </div>
                  )}
                </div>
              )}

              {/* Power plan */}
              {systemStatus.power_plan_name && (
                <div style={{ marginBottom: 14 }}>
                  <div
                    style={{
                      fontSize: 11,
                      color: "var(--text-muted)",
                      marginBottom: 4,
                    }}
                  >
                    Power Plan
                  </div>
                  <div style={{ fontSize: 12, color: "var(--text-secondary)" }}>
                    {systemStatus.power_plan_supported ? (
                      <span>
                        <Zap size={11} style={{ verticalAlign: "middle" }} />{" "}
                        {systemStatus.power_plan_name}
                      </span>
                    ) : (
                      <span style={{ color: "var(--text-muted)" }}>
                        Not supported on this device
                      </span>
                    )}
                  </div>
                </div>
              )}

              {/* Sleep prevention */}
              <div style={{ marginBottom: 14 }}>
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    fontSize: 12,
                    color: systemStatus.sleep_prevention_active
                      ? "var(--success)"
                      : "var(--text-muted)",
                  }}
                >
                  <Moon size={13} />
                  Sleep prevention:{" "}
                  {systemStatus.sleep_prevention_active ? "Active" : "Off"}
                </div>
              </div>

              <div className="divider" />

              {/* Top offenders */}
              <div>
                <div
                  style={{
                    display: "flex",
                    gap: 4,
                    marginBottom: 10,
                  }}
                >
                  {(["cpu", "ram"] as const).map((tab) => (
                    <button
                      key={tab}
                      onClick={() => setOffenderTab(tab)}
                      style={{
                        padding: "3px 10px",
                        borderRadius: 999,
                        border: "1px solid",
                        fontSize: 11,
                        fontWeight: 600,
                        background: offenderTab === tab ? "var(--bg-raised)" : "transparent",
                        borderColor: offenderTab === tab ? "var(--border)" : "transparent",
                        color: offenderTab === tab ? "var(--text-primary)" : "var(--text-muted)",
                        cursor: "pointer",
                        textTransform: "uppercase",
                        letterSpacing: "0.3px",
                      }}
                    >
                      {tab}
                    </button>
                  ))}
                </div>
                <TopOffenderList
                  processes={
                    offenderTab === "cpu"
                      ? systemStatus.top_cpu_offenders
                      : systemStatus.top_ram_offenders
                  }
                  metric={offenderTab}
                />
              </div>
            </>
          ) : (
            <div className="empty-state" style={{ padding: 24 }}>
              <div style={{ fontSize: 12, color: "var(--text-muted)" }}>
                Loading system data…
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function ActiveSessionPanel({
  profile,
  session,
  onStop,
  onRestore,
}: {
  profile: Profile;
  session: any;
  onStop: () => void;
  onRestore: () => void;
}) {
  const color = MODE_COLORS[profile.mode_type as ModeType];
  const elapsed = formatDistanceToNow(new Date(session.started_at), {
    addSuffix: false,
  });

  return (
    <div
      className="card"
      style={{
        marginBottom: 20,
        borderColor: `${color}44`,
        background: `${color}0a`,
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "flex-start",
          justifyContent: "space-between",
          marginBottom: 14,
        }}
      >
        <div>
          <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 6 }}>
            <StatusBadge mode={profile.mode_type as ModeType} size="lg" />
            <span className="tag badge-active">● Live</span>
          </div>
          <div style={{ fontSize: 17, fontWeight: 700, marginBottom: 2 }}>
            {profile.name}
          </div>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 5,
              fontSize: 12,
              color: "var(--text-secondary)",
            }}
          >
            <Clock size={12} />
            Running for {elapsed}
          </div>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-ghost" onClick={onRestore}>
            <RotateCcw size={14} /> Restore
          </button>
          <button className="btn btn-danger" onClick={onStop}>
            <Square size={14} /> Stop
          </button>
        </div>
      </div>

      {/* Active actions */}
      <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
        {profile.actions.map((action, i) => (
          <ActionChip key={i} action={action} />
        ))}
      </div>
    </div>
  );
}

function ActionChip({ action }: { action: any }) {
  const labels: Record<string, string> = {
    prevent_sleep: "🌙 Sleep Prevented",
    reduce_interruptions: "🔕 Interruptions Reduced",
    set_power_plan: "⚡ High Performance",
    watch_battery: "🔋 Battery Watch",
    watch_memory: "💾 RAM Watch",
    watch_cpu: "⚙️ CPU Watch",
    pause_approved_app: `⏸ ${action.app_id ?? "App"} Paused`,
    run_approved_script: "📜 Script Running",
  };

  return (
    <div
      style={{
        padding: "3px 10px",
        borderRadius: 999,
        background: "var(--bg-raised)",
        border: "1px solid var(--border)",
        fontSize: 11,
        color: "var(--text-secondary)",
      }}
    >
      {labels[action.kind] ?? action.kind}
    </div>
  );
}

function ProfileCard({
  profile,
  isActive,
  onStart,
  onDryRun,
}: {
  profile: Profile;
  isActive: boolean;
  onStart: () => void;
  onDryRun: () => void;
}) {
  const color = MODE_COLORS[profile.mode_type as ModeType] ?? "#64748b";

  return (
    <div
      className="card"
      style={{
        borderColor: isActive ? `${color}55` : "var(--border)",
        background: isActive ? `${color}0a` : "var(--bg-surface)",
        transition: "border-color 0.2s",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "flex-start",
          justifyContent: "space-between",
          marginBottom: 10,
        }}
      >
        <div>
          <StatusBadge mode={profile.mode_type as ModeType} size="sm" />
          <div style={{ fontWeight: 600, marginTop: 6, marginBottom: 2, fontSize: 14 }}>
            {profile.name}
          </div>
          {profile.is_builtin && (
            <div style={{ fontSize: 11, color: "var(--text-muted)" }}>Built-in</div>
          )}
        </div>
        {!profile.enabled && (
          <span className="tag badge-idle" style={{ fontSize: 10 }}>Disabled</span>
        )}
      </div>

      <div
        style={{
          fontSize: 12,
          color: "var(--text-secondary)",
          marginBottom: 12,
        }}
      >
        {profile.triggers.length} trigger{profile.triggers.length !== 1 ? "s" : ""} ·{" "}
        {profile.actions.length} action{profile.actions.length !== 1 ? "s" : ""}
      </div>

      <div style={{ display: "flex", gap: 6 }}>
        {isActive ? (
          <div className="tag badge-active">● Active</div>
        ) : (
          <button
            className="btn btn-primary"
            style={{ fontSize: 12, padding: "5px 12px" }}
            onClick={onStart}
            disabled={!profile.enabled}
          >
            <Play size={12} /> Start
          </button>
        )}
        <button
          className="btn btn-ghost"
          style={{ fontSize: 12, padding: "5px 12px" }}
          onClick={onDryRun}
        >
          Dry Run
        </button>
      </div>
    </div>
  );
}
