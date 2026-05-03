import type { Env, CachedRelease } from "./types";

const CACHE_KEY = "latest_release";
const TTL_SECONDS = 300; // 5 minutes — reduces GitHub API calls significantly

/**
 * Read the cached release from Workers KV.
 * Returns null on miss or if the entry is stale.
 */
export async function getCachedRelease(
  env: Env
): Promise<CachedRelease | null> {
  try {
    const raw = await env.CACHE_KV.get(CACHE_KEY, { type: "json" });
    if (!raw) return null;

    const cached = raw as CachedRelease;

    // Double-check age client-side (KV TTL handles server-side expiry)
    const ageMs = Date.now() - cached.fetched_at;
    if (ageMs > TTL_SECONDS * 1000) return null;

    return cached;
  } catch {
    return null;
  }
}

/**
 * Write a release to Workers KV with a TTL so it auto-expires.
 */
export async function setCachedRelease(
  env: Env,
  release: CachedRelease
): Promise<void> {
  try {
    await env.CACHE_KV.put(CACHE_KEY, JSON.stringify(release), {
      expirationTtl: TTL_SECONDS,
    });
  } catch {
    // Cache write failure is non-fatal — the response can still be served
  }
}

/**
 * Purge the KV cache entry. Called by the CI purge endpoint after a release.
 */
export async function purgeCachedRelease(env: Env): Promise<void> {
  await env.CACHE_KV.delete(CACHE_KEY);
}
