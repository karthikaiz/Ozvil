/**
 * Ozvil Update Server — Cloudflare Worker
 *
 * Endpoint: GET /{target}/{arch}/{current_version}
 *   e.g.   GET /windows/x86_64/0.1.0
 *
 * Responses:
 *   204  — no update available (current version is already latest)
 *   200  — update available, body is Tauri update JSON
 *   400  — invalid request (bad version string, unrecognised target)
 *   404  — no releases published yet
 *   500  — GitHub API or asset fetch error
 *
 * Additional routes:
 *   POST /admin/purge-cache    — purge KV cache (called by CI after release)
 */

import type { Env, TauriUpdateResponse, TauriPlatform, UpdateParams } from "./types";
import { getLatestRelease } from "./release";
import { purgeCachedRelease } from "./cache";
import { isNewerVersion, isValidVersion } from "./semver";

// ─── Target → Tauri platform key ─────────────────────────────────────────────

const PLATFORM_MAP: Record<string, TauriPlatform> = {
  "windows/x86_64": "windows-x86_64",
  "darwin/x86_64":  "darwin-x86_64",
  "darwin/aarch64": "darwin-aarch64",
  "linux/x86_64":   "linux-x86_64",
};

// ─── Main handler ─────────────────────────────────────────────────────────────

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const pathname = url.pathname;

    // ── Admin: cache purge ───────────────────────────────────────────────────
    if (pathname === "/admin/purge-cache") {
      return handlePurge(request, env);
    }

    // ── Health check ─────────────────────────────────────────────────────────
    if (pathname === "/health" || pathname === "/") {
      return json({ status: "ok", service: "ozvil-update-server" }, 200);
    }

    // ── Update check: /{target}/{arch}/{version} ──────────────────────────────
    const params = parseUpdatePath(pathname);
    if (!params) {
      return json(
        { error: "Invalid path. Expected /{target}/{arch}/{current_version}" },
        400
      );
    }

    return handleUpdateCheck(params, env);
  },
};

// ─── Route handlers ───────────────────────────────────────────────────────────

async function handleUpdateCheck(
  params: UpdateParams,
  env: Env
): Promise<Response> {
  const { target, arch, version } = params;

  // Validate version string
  if (!isValidVersion(version)) {
    return json({ error: `Invalid version string: ${version}` }, 400);
  }

  // Resolve platform key
  const platformKey = PLATFORM_MAP[`${target}/${arch}`];
  if (!platformKey) {
    return json(
      {
        error: `Unsupported target/arch: ${target}/${arch}`,
        supported: Object.keys(PLATFORM_MAP),
      },
      400
    );
  }

  // Fetch latest release (from KV cache or GitHub)
  let latest;
  try {
    latest = await getLatestRelease(env);
  } catch (err) {
    console.error("Failed to fetch latest release:", err);
    return json({ error: "Failed to fetch release information" }, 500);
  }

  // No releases published yet
  if (!latest) {
    return new Response(null, { status: 204 });
  }

  // No update available: current version is already the latest
  if (!isNewerVersion(latest.version, version)) {
    return new Response(null, { status: 204 });
  }

  // Check if this platform has an update asset
  const platformEntry = latest.platforms[platformKey];
  if (!platformEntry) {
    // Platform supported in general but no asset for this release
    return new Response(null, { status: 204 });
  }

  // Build Tauri update response
  const response: TauriUpdateResponse = {
    version: latest.version,
    notes: latest.notes,
    pub_date: latest.pub_date,
    platforms: {
      [platformKey]: platformEntry,
    },
  };

  return json(response, 200);
}

async function handlePurge(request: Request, env: Env): Promise<Response> {
  if (request.method !== "POST") {
    return json({ error: "Method not allowed" }, 405);
  }

  // Verify the purge secret to prevent unauthenticated cache busting
  const authHeader = request.headers.get("Authorization");
  const expectedToken = `Bearer ${env.PURGE_SECRET}`;

  if (!authHeader || authHeader !== expectedToken) {
    return json({ error: "Unauthorized" }, 401);
  }

  try {
    await purgeCachedRelease(env);
    return json({ ok: true, message: "Cache purged successfully" }, 200);
  } catch (err) {
    console.error("Cache purge failed:", err);
    return json({ error: "Cache purge failed" }, 500);
  }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/**
 * Parse a path like /windows/x86_64/0.1.0 into UpdateParams.
 * Returns null if the path doesn't match.
 */
function parseUpdatePath(pathname: string): UpdateParams | null {
  // Strip leading slash and split
  const parts = pathname.replace(/^\//, "").split("/");
  if (parts.length !== 3) return null;

  const [target, arch, version] = parts;
  if (!target || !arch || !version) return null;

  return { target, arch, version };
}

function json(body: unknown, status: number): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "Content-Type": "application/json",
      "Cache-Control": "no-store",
      "X-Content-Type-Options": "nosniff",
    },
  });
}
