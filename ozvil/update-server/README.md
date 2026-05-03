# Ozvil Update Server

The auto-update infrastructure for Ozvil. Handles delivering new versions to
installed users automatically via the Tauri updater plugin.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  GitHub Release (*.exe, *.msi, *.sig)                   │
└─────────────────────────────────────┬───────────────────┘
                                      │
                                      │ reads via GitHub API
                                      ▼
┌─────────────────────────────────────────────────────────┐
│  Cloudflare Worker  (update-server/worker/)             │
│                                                         │
│  GET /windows/x86_64/{version}                          │
│    ├── KV cache hit → return JSON/204 instantly         │
│    └── cache miss  → fetch GitHub → write KV → respond  │
│                                                         │
│  POST /admin/purge-cache  ← called by CI after release  │
└─────────────────────────────────────┬───────────────────┘
                                      │
                                      │ Tauri updater poll
                                      ▼
┌─────────────────────────────────────────────────────────┐
│  Ozvil app (user's Windows PC)                          │
│  tauri-plugin-updater checks on each launch             │
└─────────────────────────────────────────────────────────┘
```

## Files

```
update-server/
├── worker/                 Cloudflare Worker source
│   ├── src/
│   │   ├── index.ts        Main request handler + route dispatch
│   │   ├── release.ts      GitHub release resolver + asset downloader
│   │   ├── github.ts       GitHub API client (latest/tag release, asset text)
│   │   ├── cache.ts        Workers KV read/write/purge helpers
│   │   ├── semver.ts       Minimal semver comparator (no dependencies)
│   │   ├── types.ts        Shared TypeScript types
│   │   ├── semver.test.ts  Unit tests — semver comparison (18 cases)
│   │   └── index.test.ts   Unit tests — all routes (mocked release module)
│   ├── wrangler.toml       Cloudflare Worker configuration
│   ├── package.json
│   ├── tsconfig.json
│   ├── vitest.config.ts
│   └── README.md           ← Full setup and deployment guide
└── update.json.example     Tauri updater response format reference
```

## Quick start

See **[worker/README.md](worker/README.md)** for the complete setup guide.

The short version:

```bash
cd update-server/worker
npm install

# 1. Create KV namespace
wrangler kv namespace create CACHE_KV

# 2. Set secrets
wrangler secret put GITHUB_TOKEN   # optional but recommended
wrangler secret put PURGE_SECRET   # matches UPDATE_SERVER_PURGE_SECRET in GitHub Actions

# 3. Edit wrangler.toml with your GitHub org/repo and KV namespace IDs

# 4. Deploy
npm run deploy
```

## Release pipeline integration

The GitHub Actions release workflow
(`.github/workflows/release.yml`) automatically:

1. Builds and signs the Tauri installers
2. Publishes the GitHub Release (with `.exe`, `.msi`, `.sig` assets)
3. Calls `POST /admin/purge-cache` to invalidate the Worker's KV cache
4. Verifies the update endpoint returns the new version

**Required secrets in your GitHub repository:**

| Secret | Purpose |
|---|---|
| `UPDATE_SERVER_PURGE_SECRET` | Authenticates the cache-purge request |
| `UPDATE_SERVER_URL` | Optional override (default: `https://update.ozvil.app`) |

## How the Tauri app uses this

`tauri.conf.json`:
```json
"plugins": {
  "updater": {
    "pubkey": "YOUR_ED25519_PUBLIC_KEY",
    "endpoints": [
      "https://update.ozvil.app/{{target}}/{{arch}}/{{current_version}}"
    ]
  }
}
```

On every non-Safe-Mode launch, `src-tauri/src/updater.rs` calls the updater
plugin, which hits this endpoint. If the Worker returns 200, the Tauri dialog
appears asking the user to update.
