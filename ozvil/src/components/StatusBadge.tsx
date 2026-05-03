import type { ModeType } from "../types";
import { MODE_LABELS, MODE_COLORS, MODE_ICONS } from "../types";

interface Props {
  mode: ModeType | "idle";
  size?: "sm" | "md" | "lg";
}

export default function StatusBadge({ mode, size = "md" }: Props) {
  if (mode === "idle") {
    return (
      <span
        className="tag badge-idle"
        style={{ fontSize: size === "lg" ? 13 : size === "sm" ? 10 : 11 }}
      >
        ● Idle
      </span>
    );
  }

  const color = MODE_COLORS[mode];
  const label = MODE_LABELS[mode];
  const icon = MODE_ICONS[mode];

  return (
    <span
      className="tag"
      style={{
        background: `${color}22`,
        color,
        border: `1px solid ${color}44`,
        fontSize: size === "lg" ? 13 : size === "sm" ? 10 : 11,
      }}
    >
      <span>{icon}</span> {label}
    </span>
  );
}
