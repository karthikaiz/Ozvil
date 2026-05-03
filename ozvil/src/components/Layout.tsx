import { NavLink } from "react-router-dom";
import { ReactNode } from "react";
import {
  LayoutDashboard,
  Sliders,
  ScrollText,
  ShieldCheck,
  Settings,
  PauseCircle,
  PlayCircle,
  AlertTriangle,
} from "lucide-react";
import { useAppContext } from "../hooks/useAppContext";

const NAV = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/profiles", icon: Sliders, label: "Profiles" },
  { to: "/logs", icon: ScrollText, label: "Activity" },
  { to: "/restore", icon: ShieldCheck, label: "Restore" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export default function Layout({ children }: { children: ReactNode }) {
  const { appInfo, settings, toggleGlobalPause } = useAppContext();

  const isSafeMode = appInfo?.safe_mode ?? false;
  const isGlobalPause = settings?.global_pause ?? false;

  return (
    <div style={{ display: "flex", height: "100vh", overflow: "hidden" }}>
      {/* Sidebar */}
      <nav
        style={{
          width: 200,
          minWidth: 200,
          background: "var(--bg-surface)",
          borderRight: "1px solid var(--border)",
          display: "flex",
          flexDirection: "column",
          padding: "16px 0",
        }}
      >
        {/* Logo */}
        <div
          style={{
            padding: "0 16px 20px",
            borderBottom: "1px solid var(--border)",
            marginBottom: 8,
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            <div
              style={{
                width: 30,
                height: 30,
                borderRadius: 8,
                background: "linear-gradient(135deg, #4f83f1 0%, #a855f7 100%)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                fontSize: 14,
                fontWeight: 800,
                color: "#fff",
              }}
            >
              O
            </div>
            <div>
              <div style={{ fontWeight: 700, fontSize: 15 }}>Ozvil</div>
              {appInfo && (
                <div style={{ fontSize: 10, color: "var(--text-muted)" }}>
                  v{appInfo.version}
                </div>
              )}
            </div>
          </div>
          {isSafeMode && (
            <div className="tag badge-safe-mode" style={{ marginTop: 10, width: "100%", justifyContent: "center" }}>
              <AlertTriangle size={10} /> Safe Mode
            </div>
          )}
        </div>

        {/* Nav items */}
        <div style={{ flex: 1, padding: "0 8px" }}>
          {NAV.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              style={({ isActive }) => ({
                display: "flex",
                alignItems: "center",
                gap: 10,
                padding: "8px 10px",
                borderRadius: "var(--radius-sm)",
                color: isActive ? "var(--text-primary)" : "var(--text-secondary)",
                background: isActive ? "var(--bg-raised)" : "transparent",
                marginBottom: 2,
                fontWeight: isActive ? 500 : 400,
                fontSize: 13,
                transition: "background 0.1s, color 0.1s",
              })}
            >
              <Icon size={16} />
              {label}
            </NavLink>
          ))}
        </div>

        {/* Global Pause toggle */}
        <div style={{ padding: "12px 8px 0", borderTop: "1px solid var(--border)" }}>
          <button
            className={`btn ${isGlobalPause ? "btn-danger" : "btn-ghost"}`}
            style={{ width: "100%", justifyContent: "center" }}
            onClick={toggleGlobalPause}
            title={isGlobalPause ? "Automation is paused — click to resume" : "Pause all automation"}
          >
            {isGlobalPause ? (
              <>
                <PlayCircle size={14} /> Resume
              </>
            ) : (
              <>
                <PauseCircle size={14} /> Pause All
              </>
            )}
          </button>
        </div>
      </nav>

      {/* Main content */}
      <main
        style={{
          flex: 1,
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
        }}
      >
        {children}
      </main>
    </div>
  );
}
