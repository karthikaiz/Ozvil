import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import {
  Download,
  Filter,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  Info,
} from "lucide-react";
import type { ActivityLog } from "../types";
import { format } from "date-fns";

const RESULT_ICON: Record<string, JSX.Element> = {
  ok: <CheckCircle2 size={13} color="var(--success)" />,
  failed: <XCircle size={13} color="var(--danger)" />,
  dry_run: <Info size={13} color="var(--accent)" />,
  warning: <AlertTriangle size={13} color="var(--warning)" />,
};

export default function ActivityLogPage() {
  const [logs, setLogs] = useState<ActivityLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState("");

  useEffect(() => {
    invoke<ActivityLog[]>("get_activity_logs", { limit: 500 })
      .then(setLogs)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const filtered = filter
    ? logs.filter(
        (l) =>
          l.event_type.toLowerCase().includes(filter.toLowerCase()) ||
          l.result.toLowerCase().includes(filter.toLowerCase()) ||
          (l.failure_reason ?? "").toLowerCase().includes(filter.toLowerCase())
      )
    : logs;

  const handleExport = async (fmt: "json" | "csv") => {
    const content =
      fmt === "json"
        ? await invoke<string>("export_logs_json")
        : await invoke<string>("export_logs_csv");

    const path = await save({
      defaultPath: `ozvil-logs.${fmt}`,
      filters: [{ name: fmt.toUpperCase(), extensions: [fmt] }],
    });

    if (path) {
      await writeTextFile(path, content);
    }
  };

  return (
    <div
      style={{
        height: "100%",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
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
          <h1 style={{ fontSize: 18, fontWeight: 700, marginBottom: 2 }}>Activity Log</h1>
          <p style={{ fontSize: 12, color: "var(--text-muted)" }}>
            {logs.length} entries
          </p>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-ghost" onClick={() => handleExport("json")}>
            <Download size={14} /> JSON
          </button>
          <button className="btn btn-ghost" onClick={() => handleExport("csv")}>
            <Download size={14} /> CSV
          </button>
        </div>
      </div>

      {/* Filter */}
      <div
        style={{
          padding: "12px 24px",
          borderBottom: "1px solid var(--border)",
          flexShrink: 0,
        }}
      >
        <div style={{ position: "relative", maxWidth: 320 }}>
          <Filter
            size={13}
            style={{
              position: "absolute",
              left: 9,
              top: "50%",
              transform: "translateY(-50%)",
              color: "var(--text-muted)",
            }}
          />
          <input
            className="input"
            placeholder="Filter logs…"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            style={{ paddingLeft: 30 }}
          />
        </div>
      </div>

      {/* Log table */}
      <div className="scroll-area" style={{ flex: 1 }}>
        {loading ? (
          <div className="empty-state">Loading…</div>
        ) : filtered.length === 0 ? (
          <div className="empty-state">
            <Info size={28} />
            <div style={{ fontSize: 14 }}>No log entries</div>
          </div>
        ) : (
          <table
            style={{
              width: "100%",
              borderCollapse: "collapse",
              fontSize: 12,
            }}
          >
            <thead>
              <tr
                style={{
                  borderBottom: "1px solid var(--border)",
                  position: "sticky",
                  top: 0,
                  background: "var(--bg-base)",
                  zIndex: 1,
                }}
              >
                {["Time", "Event", "Result", "Details"].map((h) => (
                  <th
                    key={h}
                    style={{
                      padding: "8px 16px",
                      textAlign: "left",
                      fontWeight: 600,
                      color: "var(--text-muted)",
                      fontSize: 11,
                      textTransform: "uppercase",
                      letterSpacing: "0.4px",
                    }}
                  >
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filtered.map((log) => (
                <tr
                  key={log.id}
                  style={{
                    borderBottom: "1px solid var(--border-subtle)",
                  }}
                >
                  <td
                    style={{
                      padding: "9px 16px",
                      color: "var(--text-muted)",
                      fontVariantNumeric: "tabular-nums",
                      whiteSpace: "nowrap",
                      fontFamily: "var(--font-mono)",
                      fontSize: 11,
                    }}
                  >
                    {format(new Date(log.created_at), "MMM d, HH:mm:ss")}
                  </td>
                  <td style={{ padding: "9px 16px", color: "var(--text-secondary)" }}>
                    {humanizeEvent(log.event_type)}
                  </td>
                  <td style={{ padding: "9px 16px" }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 5 }}>
                      {RESULT_ICON[log.result] ?? <Info size={13} />}
                      <span
                        style={{
                          color:
                            log.result === "ok"
                              ? "var(--success)"
                              : log.result === "failed"
                              ? "var(--danger)"
                              : "var(--text-secondary)",
                          textTransform: "capitalize",
                        }}
                      >
                        {log.result}
                      </span>
                    </div>
                  </td>
                  <td
                    style={{
                      padding: "9px 16px",
                      color: "var(--text-muted)",
                      maxWidth: 320,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {log.failure_reason ??
                      log.action_kind ??
                      log.trigger_kind ??
                      ""}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}

function humanizeEvent(type: string): string {
  return type
    .replace(/_/g, " ")
    .replace(/\b\w/g, (c) => c.toUpperCase());
}
