import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  RotateCcw,
  CheckCircle2,
  XCircle,
  Clock,
  AlertTriangle,
  ShieldCheck,
} from "lucide-react";
import type { Session } from "../types";
import { useAppContext } from "../hooks/useAppContext";
import { formatDistanceToNow } from "date-fns";

export default function RestoreCenter() {
  const { activeSession, refreshSession, profiles } = useAppContext();
  const [staleSessions, setStaleSessions] = useState<Session[]>([]);
  const [restoring, setRestoring] = useState(false);
  const [restoreErrors, setRestoreErrors] = useState<string[]>([]);
  const [restoreSuccess, setRestoreSuccess] = useState(false);

  useEffect(() => {
    invoke<Session[]>("get_stale_sessions")
      .then(setStaleSessions)
      .catch(() => {});
  }, []);

  const handleRestore = async (sessionId?: string) => {
    setRestoring(true);
    setRestoreErrors([]);
    setRestoreSuccess(false);
    try {
      // Stale sessions have a specific ID and need their own restore command.
      // Active sessions use restore_session which operates on the current session.
      const errors = sessionId
        ? await invoke<string[]>("restore_stale_session", { id: sessionId })
        : await invoke<string[]>("restore_session");
      setRestoreErrors(errors);
      setRestoreSuccess(errors.length === 0);
      refreshSession();
      invoke<Session[]>("get_stale_sessions").then(setStaleSessions);
    } catch (e: any) {
      setRestoreErrors([e.toString()]);
    } finally {
      setRestoring(false);
    }
  };

  const handleDismissStale = async (id: string) => {
    await invoke("dismiss_stale_session", { id });
    setStaleSessions((prev) => prev.filter((s) => s.id !== id));
  };

  const profileName = (id: string) =>
    profiles?.find((p) => p.id === id)?.name ?? id;

  const hasAnything = activeSession || staleSessions.length > 0;

  return (
    <div
      style={{
        height: "100%",
        overflow: "hidden",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "20px 24px 16px",
          borderBottom: "1px solid var(--border)",
          flexShrink: 0,
        }}
      >
        <h1 style={{ fontSize: 18, fontWeight: 700, marginBottom: 2 }}>Restore Center</h1>
        <p style={{ fontSize: 12, color: "var(--text-muted)" }}>
          Review and restore system changes made during workload sessions.
        </p>
      </div>

      <div className="scroll-area" style={{ padding: 24 }}>
        {/* Restore feedback */}
        {restoreSuccess && (
          <div
            className="card"
            style={{
              marginBottom: 20,
              borderColor: "rgba(34,197,94,0.35)",
              background: "rgba(34,197,94,0.07)",
              display: "flex",
              alignItems: "center",
              gap: 10,
              color: "var(--success)",
            }}
          >
            <CheckCircle2 size={16} />
            <span>System state restored successfully.</span>
          </div>
        )}

        {restoreErrors.length > 0 && (
          <div
            className="card"
            style={{
              marginBottom: 20,
              borderColor: "rgba(239,68,68,0.35)",
              background: "rgba(239,68,68,0.07)",
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                marginBottom: 8,
                color: "var(--danger)",
                fontWeight: 600,
              }}
            >
              <XCircle size={15} /> Restore completed with errors
            </div>
            {restoreErrors.map((e, i) => (
              <div
                key={i}
                style={{ fontSize: 12, color: "var(--text-secondary)", marginBottom: 3 }}
              >
                • {e}
              </div>
            ))}
          </div>
        )}

        {/* Active session */}
        {activeSession && (
          <div style={{ marginBottom: 20 }}>
            <h2
              style={{
                fontSize: 12,
                fontWeight: 600,
                color: "var(--text-muted)",
                textTransform: "uppercase",
                letterSpacing: "0.5px",
                marginBottom: 10,
              }}
            >
              Active Session
            </h2>
            <SessionCard
              session={activeSession}
              profileName={profileName(activeSession.profile_id)}
              isStale={false}
              onRestore={() => handleRestore()}
              onDismiss={undefined}
              restoring={restoring}
            />
          </div>
        )}

        {/* Stale sessions */}
        {staleSessions.length > 0 && (
          <div style={{ marginBottom: 20 }}>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                marginBottom: 10,
              }}
            >
              <h2
                style={{
                  fontSize: 12,
                  fontWeight: 600,
                  color: "var(--text-muted)",
                  textTransform: "uppercase",
                  letterSpacing: "0.5px",
                }}
              >
                Unfinished Sessions
              </h2>
              <span className="tag badge-warning">
                <AlertTriangle size={9} /> {staleSessions.length}
              </span>
            </div>
            <p
              style={{
                fontSize: 12,
                color: "var(--text-secondary)",
                marginBottom: 12,
              }}
            >
              These sessions were active when Ozvil last closed. System changes from these
              sessions may still be applied.
            </p>
            {staleSessions.map((s) => (
              <SessionCard
                key={s.id}
                session={s}
                profileName={profileName(s.profile_id)}
                isStale={true}
                onRestore={() => handleRestore(s.id)}
                onDismiss={() => handleDismissStale(s.id)}
                restoring={restoring}
              />
            ))}
          </div>
        )}

        {!hasAnything && !restoreSuccess && (
          <div className="empty-state">
            <ShieldCheck size={36} />
            <div style={{ fontSize: 15, fontWeight: 600, marginTop: 8 }}>
              System is clean
            </div>
            <div style={{ fontSize: 13 }}>
              No active or unfinished sessions detected.
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function SessionCard({
  session,
  profileName,
  isStale,
  onRestore,
  onDismiss,
  restoring,
}: {
  session: Session;
  profileName: string;
  isStale: boolean;
  onRestore: () => void;
  onDismiss?: () => void;
  restoring: boolean;
}) {
  const snapshot = session.snapshot;

  return (
    <div
      className="card"
      style={{
        marginBottom: 10,
        borderColor: isStale ? "rgba(245,158,11,0.3)" : "var(--border)",
        background: isStale ? "rgba(245,158,11,0.05)" : "var(--bg-surface)",
      }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "flex-start",
          marginBottom: 12,
        }}
      >
        <div>
          <div style={{ fontWeight: 600, marginBottom: 3 }}>{profileName}</div>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 6,
              fontSize: 12,
              color: "var(--text-muted)",
            }}
          >
            <Clock size={11} />
            Started{" "}
            {formatDistanceToNow(new Date(session.started_at), { addSuffix: true })}
            {isStale && (
              <span className="tag badge-warning" style={{ fontSize: 9 }}>
                Stale
              </span>
            )}
          </div>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button
            className="btn btn-primary"
            style={{ fontSize: 12, padding: "5px 12px" }}
            onClick={onRestore}
            disabled={restoring}
          >
            <RotateCcw size={12} />
            {restoring ? "Restoring…" : "Restore Now"}
          </button>
          {onDismiss && (
            <button
              className="btn btn-ghost"
              style={{ fontSize: 12, padding: "5px 12px" }}
              onClick={onDismiss}
            >
              Dismiss
            </button>
          )}
        </div>
      </div>

      {/* Snapshot summary */}
      {snapshot && (
        <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
          {snapshot.sleep_prevention_active && (
            <SnapChip label="🌙 Sleep Prevention Active" />
          )}
          {snapshot.power_plan_name && (
            <SnapChip label={`⚡ Power Plan: ${snapshot.power_plan_name}`} />
          )}
          {snapshot.paused_apps.map((app) => (
            <SnapChip key={app} label={`⏸ ${app} paused`} />
          ))}
          {snapshot.actions_applied.length === 0 && (
            <SnapChip label="No recorded actions" />
          )}
        </div>
      )}
    </div>
  );
}

function SnapChip({ label }: { label: string }) {
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
      {label}
    </div>
  );
}
