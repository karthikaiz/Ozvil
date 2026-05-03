interface PressureBarProps {
  value: number;
  max?: number;
  label?: string;
  showPercent?: boolean;
}

function pressureColor(pct: number): string {
  if (pct >= 85) return "var(--danger)";
  if (pct >= 65) return "var(--warning)";
  return "var(--success)";
}

export default function PressureBar({
  value,
  max = 100,
  label,
  showPercent = true,
}: PressureBarProps) {
  const pct = Math.min(100, (value / max) * 100);
  const color = pressureColor(pct);

  return (
    <div>
      {(label || showPercent) && (
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: 5,
          }}
        >
          {label && (
            <span style={{ fontSize: 12, color: "var(--text-secondary)" }}>
              {label}
            </span>
          )}
          {showPercent && (
            <span
              style={{
                fontSize: 12,
                fontWeight: 600,
                color,
                fontVariantNumeric: "tabular-nums",
              }}
            >
              {Math.round(pct)}%
            </span>
          )}
        </div>
      )}
      <div className="pressure-bar">
        <div
          className="pressure-bar-fill"
          style={{ width: `${pct}%`, background: color }}
        />
      </div>
    </div>
  );
}
