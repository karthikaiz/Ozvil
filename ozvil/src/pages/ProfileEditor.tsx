import { useState, useEffect } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { save, open as openDialog } from "@tauri-apps/plugin-dialog";
import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
import {
  Plus,
  Trash2,
  Save,
  Download,
  Upload,
  Edit3,
} from "lucide-react";
import type { Profile, Trigger, Action, ModeType, RestorePolicy, ApprovalMode } from "../types";
import { useAppContext } from "../hooks/useAppContext";
import { MODE_LABELS } from "../types";

const EMPTY_PROFILE: Omit<Profile, "id" | "created_at" | "updated_at"> = {
  name: "",
  mode_type: "custom",
  triggers: [],
  actions: [],
  restore_policy: "on_app_quit",
  approval_mode: "ask_first",
  is_builtin: false,
  enabled: true,
};

export default function ProfileEditor() {
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();
  const { profiles, refreshProfiles } = useAppContext();
  const [selected, setSelected] = useState<Profile | null>(null);
  const [draft, setDraft] = useState<Partial<Profile> | null>(null);
  const [saving, setSaving] = useState(false);
  const [isNew, setIsNew] = useState(false);

  useEffect(() => {
    if (id && profiles) {
      const p = profiles.find((pr) => pr.id === id);
      if (p) {
        setSelected(p);
        setDraft({ ...p });
        setIsNew(false);
      }
    }
  }, [id, profiles]);

  const handleNew = () => {
    setIsNew(true);
    setSelected(null);
    setDraft({ ...EMPTY_PROFILE });
  };

  const handleSelect = (p: Profile) => {
    setIsNew(false);
    setSelected(p);
    setDraft({ ...p });
    navigate(`/profiles/${p.id}`);
  };

  const handleSave = async () => {
    if (!draft) return;
    setSaving(true);
    try {
      if (isNew) {
        const created = await invoke<Profile>("create_profile", { profile: draft });
        refreshProfiles();
        navigate(`/profiles/${created.id}`);
        setIsNew(false);
        setSelected(created);
        setDraft({ ...created });
      } else {
        const updated = await invoke<Profile>("update_profile", { profile: draft });
        refreshProfiles();
        setSelected(updated);
        setDraft({ ...updated });
      }
    } catch (e: any) {
      alert(`Save failed: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!selected) return;
    if (selected.is_builtin) {
      alert("Built-in profiles cannot be deleted.");
      return;
    }
    if (!confirm(`Delete profile "${selected.name}"?`)) return;
    await invoke("delete_profile", { id: selected.id });
    refreshProfiles();
    navigate("/profiles");
    setSelected(null);
    setDraft(null);
  };

  const handleExport = async () => {
    if (!selected) return;
    const json = await invoke<string>("export_profile_json", { profileId: selected.id });
    const path = await save({
      defaultPath: `${selected.name.toLowerCase().replace(/\s+/g, "-")}.json`,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (path) await writeTextFile(path, json);
  };

  const handleImport = async () => {
    const path = await openDialog({
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!path || Array.isArray(path)) return;
    const json = await readTextFile(path as string);
    await invoke<Profile>("import_profile_json", { json });
    refreshProfiles();
  };

  return (
    <div style={{ height: "100%", display: "flex", overflow: "hidden" }}>
      {/* Profile list */}
      <div
        style={{
          width: 220,
          borderRight: "1px solid var(--border)",
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
        }}
      >
        <div
          style={{
            padding: "16px 12px 10px",
            borderBottom: "1px solid var(--border)",
            flexShrink: 0,
          }}
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              marginBottom: 10,
            }}
          >
            <span style={{ fontSize: 12, fontWeight: 600, color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.4px" }}>
              Profiles
            </span>
            <button className="btn btn-ghost" style={{ padding: "4px 8px", fontSize: 12 }} onClick={handleNew}>
              <Plus size={13} />
            </button>
          </div>
          <div style={{ display: "flex", gap: 6 }}>
            <button
              className="btn btn-ghost"
              style={{ flex: 1, fontSize: 11, padding: "4px 8px" }}
              onClick={handleImport}
            >
              <Upload size={11} /> Import
            </button>
          </div>
        </div>
        <div className="scroll-area" style={{ padding: "8px 6px" }}>
          {(profiles ?? []).map((p) => (
            <button
              key={p.id}
              onClick={() => handleSelect(p)}
              style={{
                width: "100%",
                textAlign: "left",
                padding: "8px 10px",
                borderRadius: "var(--radius-sm)",
                background:
                  selected?.id === p.id ? "var(--bg-raised)" : "transparent",
                border: "none",
                color:
                  selected?.id === p.id
                    ? "var(--text-primary)"
                    : "var(--text-secondary)",
                fontSize: 13,
                cursor: "pointer",
                display: "flex",
                alignItems: "center",
                gap: 6,
                marginBottom: 1,
              }}
            >
              <span style={{ opacity: p.enabled ? 1 : 0.4 }}>
                {p.name}
              </span>
              {p.is_builtin && (
                <span style={{ fontSize: 9, color: "var(--text-muted)", marginLeft: "auto" }}>
                  built-in
                </span>
              )}
            </button>
          ))}
        </div>
      </div>

      {/* Editor */}
      <div
        style={{
          flex: 1,
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
        }}
      >
        {draft ? (
          <>
            {/* Toolbar */}
            <div
              style={{
                padding: "12px 20px",
                borderBottom: "1px solid var(--border)",
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                flexShrink: 0,
              }}
            >
              <div style={{ fontWeight: 600, fontSize: 15 }}>
                {isNew ? "New Profile" : draft.name || "Untitled"}
              </div>
              <div style={{ display: "flex", gap: 8 }}>
                {!isNew && selected && (
                  <>
                    <button className="btn btn-ghost" onClick={handleExport}>
                      <Download size={13} /> Export
                    </button>
                    {!selected.is_builtin && (
                      <button className="btn btn-danger" onClick={handleDelete}>
                        <Trash2 size={13} />
                      </button>
                    )}
                  </>
                )}
                <button
                  className="btn btn-primary"
                  onClick={handleSave}
                  disabled={saving || !draft.name}
                >
                  <Save size={13} />{saving ? "Saving…" : "Save"}
                </button>
              </div>
            </div>

            <div className="scroll-area" style={{ padding: 20 }}>
              {/* Basic info */}
              <Section title="Basic Information">
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 14 }}>
                  <div>
                    <label className="label">Profile Name</label>
                    <input
                      className="input"
                      value={draft.name ?? ""}
                      onChange={(e) => setDraft({ ...draft, name: e.target.value })}
                      placeholder="e.g. My Render Setup"
                      disabled={selected?.is_builtin}
                    />
                  </div>
                  <div>
                    <label className="label">Mode Type</label>
                    <select
                      className="input"
                      value={draft.mode_type ?? "custom"}
                      onChange={(e) => setDraft({ ...draft, mode_type: e.target.value as ModeType })}
                      disabled={selected?.is_builtin}
                    >
                      {(Object.entries(MODE_LABELS) as [ModeType, string][]).map(([k, v]) => (
                        <option key={k} value={k}>{v}</option>
                      ))}
                    </select>
                  </div>
                  <div>
                    <label className="label">Restore Policy</label>
                    <select
                      className="input"
                      value={draft.restore_policy ?? "on_app_quit"}
                      onChange={(e) => setDraft({ ...draft, restore_policy: e.target.value as RestorePolicy })}
                    >
                      <option value="on_app_quit">On App Quit</option>
                      <option value="on_resource_idle">On Resource Idle</option>
                      <option value="manual">Manual Only</option>
                    </select>
                  </div>
                  <div>
                    <label className="label">Approval Mode</label>
                    <select
                      className="input"
                      value={draft.approval_mode ?? "ask_first"}
                      onChange={(e) => setDraft({ ...draft, approval_mode: e.target.value as ApprovalMode })}
                    >
                      <option value="ask_first">Ask First</option>
                      <option value="automatic_after_trusted">Automatic After Trusted</option>
                    </select>
                  </div>
                </div>
                <div style={{ marginTop: 12, display: "flex", alignItems: "center", gap: 8 }}>
                  <input
                    type="checkbox"
                    id="enabled"
                    checked={draft.enabled ?? true}
                    onChange={(e) => setDraft({ ...draft, enabled: e.target.checked })}
                  />
                  <label htmlFor="enabled" style={{ fontSize: 13, cursor: "pointer" }}>
                    Profile enabled
                  </label>
                </div>
              </Section>

              {/* Triggers */}
              <Section title={`Triggers (${draft.triggers?.length ?? 0})`}>
                {selected?.is_builtin ? (
                  <div style={{ fontSize: 12, color: "var(--text-muted)", padding: "8px 0" }}>
                    Built-in profile triggers are read-only.
                  </div>
                ) : (
                  <TriggerEditor
                    triggers={draft.triggers ?? []}
                    onChange={(triggers) => setDraft({ ...draft, triggers })}
                  />
                )}
                {(draft.triggers ?? []).length > 0 && (
                  <div style={{ marginTop: 10 }}>
                    {(draft.triggers ?? []).map((t, i) => (
                      <TriggerRow
                        key={i}
                        trigger={t}
                        readOnly={selected?.is_builtin}
                        onRemove={() => {
                          const next = [...(draft.triggers ?? [])];
                          next.splice(i, 1);
                          setDraft({ ...draft, triggers: next });
                        }}
                      />
                    ))}
                  </div>
                )}
              </Section>

              {/* Actions */}
              <Section title={`Actions (${draft.actions?.length ?? 0})`}>
                {selected?.is_builtin ? (
                  <div style={{ fontSize: 12, color: "var(--text-muted)", padding: "8px 0" }}>
                    Built-in profile actions are read-only.
                  </div>
                ) : (
                  <ActionEditor
                    actions={draft.actions ?? []}
                    onChange={(actions) => setDraft({ ...draft, actions })}
                  />
                )}
                {(draft.actions ?? []).length > 0 && (
                  <div style={{ marginTop: 10 }}>
                    {(draft.actions ?? []).map((a, i) => (
                      <ActionRow
                        key={i}
                        action={a}
                        readOnly={selected?.is_builtin}
                        onRemove={() => {
                          const next = [...(draft.actions ?? [])];
                          next.splice(i, 1);
                          setDraft({ ...draft, actions: next });
                        }}
                      />
                    ))}
                  </div>
                )}
              </Section>
            </div>
          </>
        ) : (
          <div className="empty-state">
            <Edit3 size={32} />
            <div style={{ fontSize: 15, fontWeight: 600, marginTop: 8 }}>
              Select a profile
            </div>
            <div style={{ fontSize: 13 }}>
              Choose a profile from the list or create a new one.
            </div>
            <button className="btn btn-primary" style={{ marginTop: 12 }} onClick={handleNew}>
              <Plus size={14} /> New Profile
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div style={{ marginBottom: 24 }}>
      <div
        style={{
          fontSize: 12,
          fontWeight: 600,
          color: "var(--text-muted)",
          textTransform: "uppercase",
          letterSpacing: "0.5px",
          marginBottom: 12,
          paddingBottom: 8,
          borderBottom: "1px solid var(--border)",
        }}
      >
        {title}
      </div>
      {children}
    </div>
  );
}

function TriggerRow({ trigger, readOnly, onRemove }: { trigger: Trigger; readOnly?: boolean; onRemove: () => void }) {
  const label = (() => {
    switch (trigger.kind) {
      case "app_running":       return `App running: ${trigger.app_id}`;
      case "process_running":   return `Process running: ${trigger.process_name}`;
      case "cpu_above":         return `CPU > ${trigger.percent}% for ${trigger.duration_seconds}s`;
      case "memory_above":      return `RAM > ${trigger.mb} MB for ${trigger.duration_seconds}s`;
      case "manual_cli":        return "Manual (CLI)";
      case "manual_ui":         return "Manual (UI)";
      default:                  return JSON.stringify(trigger);
    }
  })();

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "7px 10px",
        borderRadius: "var(--radius-sm)",
        background: "var(--bg-raised)",
        border: "1px solid var(--border)",
        marginBottom: 5,
        fontSize: 12,
        color: "var(--text-secondary)",
      }}
    >
      <span>🔀 {label}</span>
      {!readOnly && (
        <button
          style={{ background: "none", border: "none", cursor: "pointer", color: "var(--text-muted)", padding: 2 }}
          onClick={onRemove}
        >
          <Trash2 size={12} />
        </button>
      )}
    </div>
  );
}

function ActionRow({ action, readOnly, onRemove }: { action: Action; readOnly?: boolean; onRemove: () => void }) {
  const label = (() => {
    switch (action.kind) {
      case "prevent_sleep":         return "Prevent sleep";
      case "reduce_interruptions":  return "Reduce interruptions (best-effort)";
      case "set_power_plan":        return `Set power plan: ${action.plan_id}`;
      case "pause_approved_app":    return `Pause app: ${action.app_id}`;
      case "watch_battery":         return `Watch battery < ${action.warn_below_percent}%`;
      case "watch_memory":          return `Watch RAM > ${action.warn_above_percent}%`;
      case "watch_cpu":             return `Watch CPU > ${action.warn_above_percent}%`;
      case "run_approved_script":   return `Run script: ${action.script_id}`;
      default:                      return JSON.stringify(action);
    }
  })();

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "7px 10px",
        borderRadius: "var(--radius-sm)",
        background: "var(--bg-raised)",
        border: "1px solid var(--border)",
        marginBottom: 5,
        fontSize: 12,
        color: "var(--text-secondary)",
      }}
    >
      <span>⚡ {label}</span>
      {!readOnly && (
        <button
          style={{ background: "none", border: "none", cursor: "pointer", color: "var(--text-muted)", padding: 2 }}
          onClick={onRemove}
        >
          <Trash2 size={12} />
        </button>
      )}
    </div>
  );
}

function TriggerEditor({ triggers, onChange }: { triggers: Trigger[]; onChange: (t: Trigger[]) => void }) {
  const [kind, setKind] = useState<string>("process_running");
  const [value, setValue] = useState("");

  const add = () => {
    if (!value) return;
    let t: Trigger;
    switch (kind) {
      case "process_running": t = { kind: "process_running", process_name: value }; break;
      case "app_running":     t = { kind: "app_running", app_id: value }; break;
      default: return;
    }
    onChange([...triggers, t]);
    setValue("");
  };

  return (
    <div style={{ display: "flex", gap: 8 }}>
      <select className="input" style={{ width: 160 }} value={kind} onChange={(e) => setKind(e.target.value)}>
        <option value="process_running">Process running</option>
        <option value="app_running">App running</option>
      </select>
      <input
        className="input"
        placeholder="e.g. blender.exe"
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && add()}
      />
      <button className="btn btn-ghost" onClick={add}><Plus size={14} /></button>
    </div>
  );
}

function ActionEditor({ actions, onChange }: { actions: Action[]; onChange: (a: Action[]) => void }) {
  const [kind, setKind] = useState("prevent_sleep");
  const [numVal, setNumVal] = useState("85");
  const [strVal, setStrVal] = useState("");

  const add = () => {
    let a: Action;
    switch (kind) {
      case "prevent_sleep":       a = { kind: "prevent_sleep" }; break;
      case "reduce_interruptions": a = { kind: "reduce_interruptions" }; break;
      case "watch_cpu":           a = { kind: "watch_cpu", warn_above_percent: Number(numVal) }; break;
      case "watch_memory":        a = { kind: "watch_memory", warn_above_percent: Number(numVal) }; break;
      case "watch_battery":       a = { kind: "watch_battery", warn_below_percent: Number(numVal) }; break;
      case "pause_approved_app":  if (!strVal) return; a = { kind: "pause_approved_app", app_id: strVal }; break;
      default: return;
    }
    onChange([...actions, a]);
    setStrVal("");
  };

  const needsNum = ["watch_cpu", "watch_memory", "watch_battery"].includes(kind);
  const needsStr = ["pause_approved_app"].includes(kind);

  return (
    <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
      <select className="input" style={{ width: 180 }} value={kind} onChange={(e) => setKind(e.target.value)}>
        <option value="prevent_sleep">Prevent sleep</option>
        <option value="reduce_interruptions">Reduce interruptions</option>
        <option value="watch_cpu">Watch CPU</option>
        <option value="watch_memory">Watch RAM</option>
        <option value="watch_battery">Watch battery</option>
        <option value="pause_approved_app">Pause approved app</option>
      </select>
      {needsNum && (
        <input
          className="input"
          type="number"
          style={{ width: 80 }}
          value={numVal}
          onChange={(e) => setNumVal(e.target.value)}
          min={1} max={100}
        />
      )}
      {needsStr && (
        <input
          className="input"
          placeholder="app_id"
          value={strVal}
          onChange={(e) => setStrVal(e.target.value)}
        />
      )}
      <button className="btn btn-ghost" onClick={add}><Plus size={14} /></button>
    </div>
  );
}
