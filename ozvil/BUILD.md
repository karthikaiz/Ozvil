# Building Ozvil

## Prerequisites

- **Windows 10 or 11** (required — Windows-native APIs are used)
- **Rust** stable toolchain: https://rustup.rs
  ```
  rustup target add x86_64-pc-windows-msvc
  ```
- **Node.js** 20+ and **pnpm** 9+:
  ```
  npm install -g pnpm
  ```
- **Tauri v2 CLI**:
  ```
  pnpm add -g @tauri-apps/cli@^2
  ```
- **WebView2** (pre-installed on Windows 11; download for Windows 10):
  https://developer.microsoft.com/microsoft-edge/webview2/

## Development

```bash
cd ozvil
pnpm install
pnpm tauri dev
```

This opens the app in development mode with hot-reload for the React UI.

## Run Tests

```bash
cd ozvil
cargo test --manifest-path src-tauri/Cargo.toml
```

Tests use a simulated Windows adapter (no real OS calls), so they run on any platform.

## Production Build

```bash
cd ozvil
pnpm tauri build
```

Output: `src-tauri/target/release/bundle/`

- NSIS installer: `src-tauri/target/release/bundle/nsis/Ozvil_0.1.0_x64-setup.exe`
- MSI installer: `src-tauri/target/release/bundle/msi/Ozvil_0.1.0_x64_en-US.msi`

## CLI Usage

After building, the `ozvil.exe` binary also functions as a CLI:

```
ozvil status
ozvil status --agent
ozvil --safe-mode
ozvil profiles list
ozvil start build
ozvil dry-run render
ozvil stop
ozvil restore
ozvil logs export --format json --output logs.json
ozvil logs export --format csv
ozvil profile export "Build Mode" --output build-mode.json
ozvil profile import my-profile.json
```

## Safe Mode

Launch with automation fully disabled:

```
ozvil --safe-mode
ozvil safe-start
```

Or use the **"Ozvil (Safe Mode)"** Start Menu shortcut created by the installer.

In Safe Mode:
- All automatic profile activation is disabled
- No scripts, app pauses, or power actions run
- Profile editing, log export, manual restore, and Global Pause still work

## Start Menu Shortcuts

The Windows installer creates two Start Menu shortcuts:
- **Ozvil** — normal launch
- **Ozvil (Safe Mode)** — passes `--safe-mode`, the primary safety hatch

## Architecture Notes

```
ozvil/
├── src-tauri/          Rust backend (Tauri v2)
│   ├── src/
│   │   ├── core/       Rules engine, session manager, snapshot/restore, activity logger
│   │   ├── db/         SQLite schema, models, migrations
│   │   ├── windows_adapter/   Native Windows API implementations
│   │   │   ├── process_provider.rs    Process enumeration via ToolHelp32
│   │   │   ├── performance_provider.rs  CPU/RAM via PDH + GlobalMemoryStatusEx
│   │   │   ├── power_provider.rs      Power status + powercfg
│   │   │   ├── sleep_provider.rs      SetThreadExecutionState
│   │   │   ├── app_control_provider.rs  Process suspend/resume
│   │   │   └── notification_provider.rs  Best-effort + manual checklist
│   │   ├── profiles/   Built-in profile definitions + repository
│   │   ├── commands/   Tauri command bridge (all invoke handlers)
│   │   └── cli/        clap-based CLI (same binary, same DB)
│   └── Cargo.toml
├── src/                React + TypeScript frontend
│   ├── pages/          Dashboard, ProfileEditor, ActivityLog, DryRunPreview, RestoreCenter, Settings
│   ├── components/     Layout, PressureBar, TopOffenderList, StatusBadge
│   ├── hooks/          useAppContext (state, polling, session management)
│   └── types/          Shared TypeScript types mirroring Rust models
└── tests/              Rust unit tests (mock adapter, no Windows required)
```

## Security Principles

Per spec:
- No hidden system mutations
- No background app control without explicit user approval
- No arbitrary script execution without approval + timeout + log
- No registry cleaning or undocumented registry hacks
- No default telemetry
- No kernel drivers
- Sleep prevention uses non-persistent `SetThreadExecutionState` (releases on exit)
- Power plan switching is capability-detected before attempting

## Phase Status

| Phase | Status |
|---|---|
| 0. Windows Plan Pivot | Completed |
| 1. Windows Product Spec | Completed (Plan.md) |
| 2. App Scaffold | **In Progress** |
| 3. Windows Feasibility Spike | Planned |
| 4. Data Layer | In Progress (SQLite + migrations done) |
| 5. Rules Engine | In Progress (core logic + tests done) |
| 6. Windows Adapter | In Progress (adapters scaffolded) |
| 7. CLI + Log/Profile Export | In Progress (CLI done) |
| 8. Trust UI | In Progress (all pages done) |
| 9. End-to-End Windows MVP | Planned |
| 10. Packaging + Beta Readiness | Planned |
