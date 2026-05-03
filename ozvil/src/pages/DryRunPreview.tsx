import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import {
  Play,
  ArrowLeft,
  CheckCircle2,
  XCircle,
  AlertTriangle,
} from "lucide-react";
import type { DryRunResult, Action } from "../types";

function actionLabel(action: Action): string {
  switch (action.kind) {
    case "prevent_sleep":         return "Prevent system sleep";
    case "reduce_interruptions":  return "Reduce interruptions (best-effort)";
    case "set_power_plan":        return `Switch to power plan: ${action.plan_id}`;
    case "pause_approved_app":    return `Pause approved app: ${action.app_id}`;
    case "watch_battery":         return `Watch battery — warn below ${action.warn_below_percent}%`;
    case "watch_memory":          return `Watch RAM — warn above ${action.warn_above_percent}%`;
    case "watch_cpu":             return `Watch CPU — warn above ${action.warn_above_percent}%`;
    case "run_approved_script":   return `Run approved script: ${action.script_id}`;
    default:                      return JSON.stringify(action);
  }
}

export default function DryRunPreview() {
  const { profileId } = useParams<{ profileId: string }>();
  const navigate = useNavigate();
  const [result, setResult] = useState<DryRunResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [starting, setStarting] = useState(false);

  useEffect(() => {
    if (!profileId) return;
    invoke<DryRunResult>("dry_run_profile", { profileId })
      .then(setResult)
      .finally(() => setLoading(false));
  }, [profileId]);

  const handleStart = async () => {
    if (!profileId) return;
    setStarting(true);
    try {
      await invoke("start_profile", { profileId });
      navigate("/");
    } catch (e: any) {
      alert(`Could not start: ${e}`);
    } finally {
      setStarting(false);
    }
  };

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
          display: "flex",
          alignItems: "center",
          gap: 12,
          flexShrink: 0,
        }}
      >
        <button className="btn btn-ghost" onClick={() => navigate(-1)}>
          <ArrowLeft size={14} />
        </button>
        <div>
          <h1 style={{ fontSize: 18, fontWeight: 700, marginBottom: 2 }}>Dry Run Preview</h1>
          <p style={{ fontSize: 12, color: "var(--text-muted)" }}>
            No system changes will be applied until you confirm.
          </p>
        </div>
      </div>

      <div className="scroll-area" style={{ padding: 24 }}>
        {loading ? (
          <div className="empty-state">Loading preview…</div>
        ) : !result ? (
          <div className="empty-state">
            <XCircle size={28} />
            <div>Failed to load dry run</div>
          </div>
        ) : (
          <>
            {/* Profile info */}
            <div className="card" style={{ marginBottom: 20 }}>
              <div style={{ fontWeight: 700, fontSize: 16, marginBottom: 4 }}>
                {result.profile_name}
              </div>
              <div style={{ fontSize: 12, color: "var(--text-muted)" }}>
                Profile ID: {result.profile_id}
              </div>
            </div>

            {/* Warnings */}
            {result.warnings.length > 0 && (
              <div
                className="card"
                style={{
                  marginBottom: 20,
                  borderColor: "rgba(245,158,11,0.35)",
                  background: "rgba(245,158,11,0.07)",
                }}
              >
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    marginBottom: 10,
                    fontWeight: 600,
                    color: "var(--warning)",
                  }}
                >
                  <AlertTriangle size={15} /> Warnings
                </div>
                {result.warnings.map((w, i) => (
                  <div
                    key={i}
                    style={{
                      fontSize: 13,
                      color: "var(--text-secondary)",
                      marginBottom: 4,
                      display: "flex",
                      gap: 8,
                    }}
                  >
                    <span style={{ color: "var(--warning)" }}>•</span> {w}
                  </div>
                ))}
              </div>
            )}

            {/* Planned actions */}
            <div style={{ marginBottom: 24 }}>
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
                Planned Actions
              </h2>
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                {result.planned_actions.map((pa, i) => (
                  <div
                    key={i}
                    className="card"
                    style={{
                      display: "flex",
                      alignItems: "flex-start",
                      gap: 12,
                      padding: "12px 16px",
                      borderColor: pa.feasible ? "var(--border)" : "rgba(239,68,68,0.3)",
                      background: pa.feasible
                        ? "var(--bg-surface)"
                        : "rgba(239,68,68,0.05)",
                    }}
                  >
                    <div style={{ marginTop: 1 }}>
                      {pa.feasible ? (
                        <CheckCircle2 size={16} color="var(--success)" />
                      ) : (
                        <XCircle size={16} color="var(--danger)" />
                      )}
                    </div>
                    <div>
                      <div
                        style={{
                          fontWeight: 500,
                          marginBottom: pa.reason ? 4 : 0,
                          color: pa.feasible ? "var(--text-primary)" : "var(--text-secondary)",
                        }}
                      >
                        {actionLabel(pa.action)}
                      </div>
                      {pa.reason && (
                        <div style={{ fontSize: 12, color: "var(--text-muted)" }}>
                          {pa.reason}
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Confirm */}
            <div
              style={{
                display: "flex",
                gap: 10,
                paddingTop: 8,
                borderTop: "1px solid var(--border)",
              }}
            >
              <button
                className="btn btn-primary"
                onClick={handleStart}
                disabled={starting}
              >
                <Play size={14} />{" "}
                {starting ? "Starting…" : "Confirm & Start"}
              </button>
              <button className="btn btn-ghost" onClick={() => navigate(-1)}>
                Cancel
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
