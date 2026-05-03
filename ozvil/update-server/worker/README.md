# Ozvil Update Server — Cloudflare Worker

A zero-maintenance Cloudflare Worker that automatically serves Tauri update
JSON by reading your GitHub Releases. Every `git push v*` tag triggers the
release pipeline; the Worker picks it up within seconds of publish.

## How it works

```
Ozvil app (user's PC)
  │
  │ GET /windows/x86_64/0.1.0
  ▼
Cloudflare Worker (update.ozvil.app)
  │
  ├── KV cache hit?  → return cached response (< 1 ms)
  │
  └── cache miss → GET github.com/repos/.../releases/latest
                    ├── download .sig file contents
                    ├── write to KV (TTL 5 min)
                    └── return Tauri update JSON (or 204)

GitHub Actions release workflow
  └── after publish → POST /admin/purge-cache → instant cache invalidation
```

**Result**: new releases reach users within seconds of publishing, with no
manual manifest files, no S3 buckets, and no update server to maintain.

---

## First-time setup

### 1. Install Wrangler

```bash
npm install -g wrangler
wrangler login
```

### 2. Create the KV namespace

```bash
# Production namespace
wrangler kv namespace create CACHE_KV
# → copy the id into wrangler.toml: [[kv_namespaces]] id = "..."

# Preview namespace (used by wrangler dev)
wrangler kv namespace create CACHE_KV --preview
# → copy the preview_id into wrangler.toml: [[kv_namespaces]] preview_id = "..."

# Staging namespace (optional)
wrangler kv namespace create CACHE_KV --env staging
```

### 3. Configure wrangler.toml

Edit `update-server/worker/wrangler.toml`:

```toml
[vars]
GITHUB_OWNER = "your-github-username-or-org"
GITHUB_REPO  = "ozvil"

[[kv_namespaces]]
binding    = "CACHE_KV"
id         = "paste-production-namespace-id-here"
preview_id = "paste-preview-namespace-id-here"
```

### 4. Set secrets

```bash
cd update-server/worker

# Strongly recommended: GitHub personal access token (read-only, public_repo scope)
# Raises GitHub API rate limit from 60 → 5000 requests/hour.
# Create at: https://github.com/settings/tokens/new?scopes=public_repo
wrangler secret put GITHUB_TOKEN

# Required: random string used to authenticate CI cache purge requests
# Generate one with: openssl rand -base64 32
wrangler secret put PURGE_SECRET
```

### 5. Set up the custom domain (optional but recommended)

In the Cloudflare dashboard:
1. Add `update.ozvil.app` as a DNS record pointing to your Worker
2. Uncomment the `[[routes]]` section in `wrangler.toml`
3. Update `tauri.conf.json` to use `https://update.ozvil.app/...`

Without a custom domain the Worker is available at
`https://ozvil-update-server.<your-cf-subdomain>.workers.dev`.

### 6. Deploy

```bash
cd update-server/worker
npm install

# Deploy to production
npm run deploy

# Or deploy to staging first
npm run deploy:staging
```

### 7. Verify

```bash
# Should return 204 (no update — same version as latest)
curl -I https://update.ozvil.app/windows/x86_64/99.99.99

# Should return 200 with update JSON (very old version)
curl https://update.ozvil.app/windows/x86_64/0.0.1 | jq .

# Health check
curl https://update.ozvil.app/health
```

---

## GitHub Actions secrets

Add these secrets to your GitHub repository
(`Settings → Secrets and variables → Actions`):

| Secret | Required | Value |
|---|---|---|
| `UPDATE_SERVER_PURGE_SECRET` | Yes (for instant updates) | Same value as `PURGE_SECRET` wrangler secret |
| `UPDATE_SERVER_URL` | No | `https://update.ozvil.app` (default) |
| `UPDATE_SERVER_STAGING_PURGE_SECRET` | No | Staging purge secret |
| `UPDATE_SERVER_STAGING_URL` | No | `https://update-staging.ozvil.app` |

Once set, every stable release tag (`v*.*.*` without `beta`/`rc`) will:
1. Build and sign the installers
2. Publish the GitHub Release
3. Call `POST /admin/purge-cache` on your Worker
4. Verify the endpoint returns the new version within 3 seconds

---

## Local development

```bash
cd update-server/worker
npm install
npm run dev
```

Wrangler opens a local server at `http://localhost:8787`. It uses the
`preview_id` KV namespace and talks to the real GitHub API.

Test locally:
```bash
curl http://localhost:8787/windows/x86_64/0.1.0
curl http://localhost:8787/health
curl -X POST http://localhost:8787/admin/purge-cache \
  -H "Authorization: Bearer your-dev-purge-secret"
```

---

## Running tests

```bash
cd update-server/worker
npm install
npm test
```

Tests cover:
- `semver.ts` — version comparison logic (18 cases)
- `index.ts` — all route handlers with mocked release module:
  - 204 on no-update, equal version, older version
  - 200 with correct Tauri JSON on newer version
  - 400 on invalid version strings (including path traversal attempts)
  - 400 on unsupported target/arch
  - 500 on GitHub API error
  - Cache purge: 200 with correct secret, 401 with wrong/missing secret

---

## API reference

### `GET /{target}/{arch}/{current_version}`

Check for an update.

**Path parameters:**

| Parameter | Values |
|---|---|
| `target` | `windows` |
| `arch` | `x86_64` |
| `current_version` | semver string, e.g. `0.1.0` |

**Responses:**

| Status | Meaning |
|---|---|
| `200` | Update available — body is [Tauri update JSON](#tauri-update-json) |
| `204` | No update (current version is already latest) |
| `400` | Invalid path or version string |
| `404` | No releases published yet |
| `500` | GitHub API or asset fetch error |

### Tauri update JSON

```json
{
  "version": "0.2.0",
  "notes": "What changed in this release",
  "pub_date": "2026-06-01T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ...",
      "url": "https://github.com/.../Ozvil_0.2.0_x64-setup.exe"
    }
  }
}
```

### `GET /health`

Returns `{ "status": "ok" }` with HTTP 200. Use for uptime monitoring.

### `POST /admin/purge-cache`

Purges the KV cache. Requires `Authorization: Bearer <PURGE_SECRET>` header.

**Responses:** `200 OK`, `401 Unauthorized`, `405 Method Not Allowed`

---

## Caching

- KV TTL: **5 minutes** — maximum staleness without a purge
- Cache is automatically purged by the release pipeline immediately after publish
- GitHub `.sig` asset contents are cached alongside version metadata — no
  repeated asset downloads per request
- GitHub API calls use `Authorization: Bearer <GITHUB_TOKEN>` if set,
  increasing the rate limit from 60 to 5000 requests/hour

---

## Stable vs. pre-release channel

The Worker's `getLatestRelease` function skips GitHub Releases that are
marked as `draft: true` or `prerelease: true`.

To serve beta releases on a separate endpoint, deploy a second Worker
instance with a separate KV namespace that calls `fetchReleaseByTag` or
the `/releases` list endpoint filtering for pre-releases.

---

## Monitoring

Add a Cloudflare Workers uptime check or use any external monitor:

```bash
# Uptime monitor URL (expects HTTP 200)
https://update.ozvil.app/health

# Update endpoint probe (expects 200 or 204)
https://update.ozvil.app/windows/x86_64/0.0.1
```

View Worker metrics (requests, errors, CPU time) in the
[Cloudflare dashboard](https://dash.cloudflare.com) → Workers & Pages → ozvil-update-server.
