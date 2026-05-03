# Ozvil Changelog

## [Unreleased] — v0.1.0

### Added

**App scaffold & core**
- Tauri v2 + Rust + React + TypeScript + Vite project structure
- SQLite data layer (WAL mode, foreign keys, bundled `rusqlite`)
- Database migrations: profiles, sessions, activity_logs, approved_apps, approved_scripts, settings

**Rules engine**
- Trigger evaluation: app/process running, CPU above threshold, RAM above threshold, manual CLI/UI
- Conflict resolution: Recording/Studio beat other auto profiles; manual sessions are sticky
- Global Pause: suppresses all new automatic activations; emergency restore still runs
- Safe Mode: disables all automation before any profile/script/action can run

**Session management**
- Session start/stop with system snapshot capture
- Stale session reconciliation on startup (handles Ozvil crash, force-quit, Windows reboot)
- One-click manual restore from active or stale snapshot

**Snapshot & restore**
- Pre-action snapshot: power plan, sleep prevention state, paused apps
- Per-action failure isolation: failed actions do not block restoring successful ones
- Dry-run mode: plans actions without applying any system changes
- Idempotent restore

**Windows adapter**
- `process_provider`: process enumeration via ToolHelp32 (Windows), stub for dev/test
- `performance_provider`: CPU/RAM via PDH + GlobalMemoryStatusEx
- `power_provider`: battery/AC status via GetSystemPowerStatus; power plan via powercfg; capability detection for Modern Standby/OEM-restricted devices
- `sleep_provider`: non-persistent sleep prevention via SetThreadExecutionState; releases on exit
- `app_control_provider`: user-approved process suspend/resume via NtSuspendProcess thread enumeration
- `notification_provider`: best-effort interruption reduction (returns UnsupportedCapability per spec); manual interruption checklist

**Built-in profiles (6)**
- Render Mode: DaVinci Resolve, Premiere Pro, After Effects, Blender, Cinema 4D, Media Encoder
- Studio Mode: FL Studio, Ableton Live, Reaper, Pro Tools, Audition, Cubase
- Build Mode: Docker, WSL, node, python, cargo, ollama, MSBuild, UnrealBuildTool
- Game Mode Plus: Steam, Epic, Unity, Unreal Editor, OBS
- Design Mode: Photoshop, Illustrator, Figma, Affinity Designer, Stable Diffusion WebUI, ComfyUI
- Recording Mode: OBS, Streamlabs, Camtasia, Zoom, Teams, PowerPoint

**CLI** (`ozvil.exe`)
- `ozvil status` — human-readable system status
- `ozvil status --agent` — stable minimal JSON for terminal agents/scripts
- `ozvil --safe-mode` / `ozvil safe-start` — launch with automation disabled
- `ozvil profiles list` — list available profiles
- `ozvil start <profile>` — activate a profile
- `ozvil dry-run <profile>` — preview actions without applying
- `ozvil stop` — end active session
- `ozvil restore` — restore system state
- `ozvil logs export --format json|csv [--output path]`
- `ozvil profile export <profile> [--output path]`
- `ozvil profile import <path>`

**Activity logging**
- Rate-limited (configurable per-minute cap)
- Prunable (configurable max entries; auto-pruned on startup)
- Logs every action with: timestamp, profile/session id, trigger, action, result, failure reason, restore status
- JSON and CSV export

**React UI**
- Dashboard: live session panel, quick-start cards, stale session recovery banner, system status sidebar (CPU/RAM/battery/power plan/sleep), top CPU/RAM offender list with toggle
- Profile Editor: create/edit/delete custom profiles, trigger + action builders, import/export JSON, read-only built-in view
- Activity Log: sortable/filterable table, JSON/CSV export via save dialog
- Dry Run Preview: feasibility check per action, warnings for OEM power plan restrictions and Battery Saver conflicts
- Restore Center: active and stale session restore with per-action result display
- Settings: Global Pause toggle, monitoring interval, warning thresholds, log retention, approval mode

**Installer & release**
- WiX MSI installer with custom Safe Mode shortcut fragment
- NSIS installer with `!customInstall` / `!customUninstall` macros for Safe Mode shortcut
- Both create dedicated `Ozvil (Safe Mode)` Start Menu shortcut (per spec)
- Tauri updater plugin with ed25519 signature verification
- Code signing via Authenticode (signtool, DigiCert timestamp)
- GitHub Actions: CI workflow (typecheck + clippy + tests) and Release workflow (build + sign + GitHub Release)
- PowerShell scripts: key generation, code signing, release build, installer verification

**Tests**
- Rules engine unit tests (mock adapter, no OS calls)
- Session manager unit tests
- Restore / snapshot unit tests with mock Windows adapter
- All tests run on any platform (Windows, Linux, macOS)

### Not in v1

- Aggressive RAM cleaning, registry cleaning, kernel drivers, virtual audio drivers
- Automatic process restart on hang
- Network/domain blocking
- VRAM/GPU metrics (experimental, deferred)
- ASIO stream detection (research-only)
- Default telemetry
- macOS support
