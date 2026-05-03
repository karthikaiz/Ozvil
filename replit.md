# Workspace

## Overview

pnpm workspace monorepo using TypeScript. Each package manages its own dependencies.

## Ozvil — Windows Desktop App

A full Tauri v2 + Rust + React desktop application lives in `ozvil/`. It is **not part of the monorepo** and is not run here — it is built on a Windows machine using the Tauri toolchain.

See `ozvil/BUILD.md` for full build and development instructions.

### Ozvil Stack
- **Runtime**: Tauri v2 (Rust + WebView2)
- **Backend**: Rust — `src-tauri/`
- **Frontend**: React 18 + TypeScript + Vite
- **Database**: SQLite via `rusqlite` (bundled)
- **Windows APIs**: `windows` crate (ToolHelp32, PDH, SetThreadExecutionState, powercfg)
- **CLI**: `clap` v4

### Ozvil Key Files
- `ozvil/src-tauri/src/core/` — Rules engine, session manager, snapshot/restore, activity logger
- `ozvil/src-tauri/src/windows_adapter/` — All native Windows OS integrations
- `ozvil/src-tauri/src/profiles/builtin.rs` — 6 built-in workload profiles
- `ozvil/src-tauri/src/cli/mod.rs` — Full CLI (`ozvil status`, `start`, `dry-run`, `restore`, `logs export`, etc.)
- `ozvil/src-tauri/src/commands/mod.rs` — All Tauri `invoke` command handlers
- `ozvil/src/pages/` — Dashboard, ProfileEditor, ActivityLog, DryRunPreview, RestoreCenter, Settings
- `ozvil/tests/` — Unit tests using mock Windows adapter (run on any platform)

## Stack

- **Monorepo tool**: pnpm workspaces
- **Node.js version**: 24
- **Package manager**: pnpm
- **TypeScript version**: 5.9
- **API framework**: Express 5
- **Database**: PostgreSQL + Drizzle ORM
- **Validation**: Zod (`zod/v4`), `drizzle-zod`
- **API codegen**: Orval (from OpenAPI spec)
- **Build**: esbuild (CJS bundle)

## Key Commands

- `pnpm run typecheck` — full typecheck across all packages
- `pnpm run build` — typecheck + build all packages
- `pnpm --filter @workspace/api-spec run codegen` — regenerate API hooks and Zod schemas from OpenAPI spec
- `pnpm --filter @workspace/db run push` — push DB schema changes (dev only)
- `pnpm --filter @workspace/api-server run dev` — run API server locally

See the `pnpm-workspace` skill for workspace structure, TypeScript setup, and package details.
