import type { ProcessInfo } from "../types";

interface Props {
  processes: ProcessInfo[];
  metric: "cpu" | "ram";
}

export default function TopOffenderList({ processes, metric }: Props) {
  if (processes.length === 0) {
    return (
      <div style={{ color: "var(--text-muted)", fontSize: 12, padding: "8px 0" }}>
        No data
      </div>
    );
  }

  const max =
    metric === "cpu"
      ? Math.max(...processes.map((p) => p.cpu_percent), 1)
      : Math.max(...processes.map((p) => p.ram_mb), 1);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
      {processes.map((p) => {
        const value = metric === "cpu" ? p.cpu_percent : p.ram_mb;
        const label =
          metric === "cpu"
            ? `${p.cpu_percent.toFixed(1)}%`
            : `${(p.ram_mb / 1024).toFixed(1)} GB`;
        const pct = (value / max) * 100;
        const color = metric === "cpu" ? "var(--accent)" : "var(--studio)";

        return (
          <div key={p.pid}>
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                marginBottom: 3,
              }}
            >
              <span
                style={{
                  fontSize: 12,
                  color: "var(--text-primary)",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                  maxWidth: "70%",
                }}
              >
                {p.name}
              </span>
              <span
                style={{
                  fontSize: 11,
                  color: "var(--text-secondary)",
                  fontVariantNumeric: "tabular-nums",
                }}
              >
                {label}
              </span>
            </div>
            <div className="pressure-bar">
              <div
                className="pressure-bar-fill"
                style={{ width: `${pct}%`, background: color }}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
}
