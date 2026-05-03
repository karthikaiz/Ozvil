import { describe, it, expect, vi, beforeEach } from "vitest";
import type { Env, CachedRelease } from "./types";

// ─── Mock the release module ──────────────────────────────────────────────────
vi.mock("./release", () => ({
  getLatestRelease: vi.fn(),
}));

vi.mock("./cache", () => ({
  purgeCachedRelease: vi.fn(),
  getCachedRelease: vi.fn(),
  setCachedRelease: vi.fn(),
}));

import { getLatestRelease } from "./release";
import { purgeCachedRelease } from "./cache";
import worker from "./index";

// ─── Test fixtures ────────────────────────────────────────────────────────────

function makeEnv(overrides: Partial<Env> = {}): Env {
  return {
    GITHUB_OWNER: "ozvil",
    GITHUB_REPO: "ozvil",
    PURGE_SECRET: "test-secret-123",
    CACHE_KV: {} as KVNamespace,
    ...overrides,
  };
}

const LATEST_RELEASE: CachedRelease = {
  version: "0.2.0",
  notes: "Bug fixes and performance improvements",
  pub_date: "2026-06-01T12:00:00Z",
  platforms: {
    "windows-x86_64": {
      signature: "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk=",
      url: "https://github.com/ozvil/ozvil/releases/download/v0.2.0/Ozvil_0.2.0_x64-setup.exe",
    },
  },
  fetched_at: Date.now(),
};

function req(path: string, method = "GET"): Request {
  return new Request(`https://update.ozvil.app${path}`, { method });
}

// ─── Tests ────────────────────────────────────────────────────────────────────

describe("GET /{target}/{arch}/{version}", () => {
  const env = makeEnv();

  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("returns 204 when no releases are published", async () => {
    vi.mocked(getLatestRelease).mockResolvedValue(null);
    const res = await worker.fetch(req("/windows/x86_64/0.1.0"), env);
    expect(res.status).toBe(204);
  });

  it("returns 204 when current version equals latest", async () => {
    vi.mocked(getLatestRelease).mockResolvedValue(LATEST_RELEASE);
    const res = await worker.fetch(req("/windows/x86_64/0.2.0"), env);
    expect(res.status).toBe(204);
  });

  it("returns 204 when current version is newer than latest (rollback case)", async () => {
    vi.mocked(getLatestRelease).mockResolvedValue(LATEST_RELEASE);
    const res = await worker.fetch(req("/windows/x86_64/0.3.0"), env);
    expect(res.status).toBe(204);
  });

  it("returns 200 with update JSON when newer version exists", async () => {
    vi.mocked(getLatestRelease).mockResolvedValue(LATEST_RELEASE);
    const res = await worker.fetch(req("/windows/x86_64/0.1.0"), env);
    expect(res.status).toBe(200);

    const body = await res.json() as Record<string, unknown>;
    expect(body.version).toBe("0.2.0");
    expect(body.notes).toBe("Bug fixes and performance improvements");
    expect(body.pub_date).toBe("2026-06-01T12:00:00Z");
    expect(body.platforms).toHaveProperty("windows-x86_64");

    const platformEntry = (body.platforms as Record<string, unknown>)["windows-x86_64"] as Record<string, string>;
    expect(platformEntry.url).toContain("Ozvil_0.2.0_x64-setup.exe");
    expect(platformEntry.signature).toBeTruthy();
  });

  it("returns 204 when platform has no assets in this release", async () => {
    const releaseWithoutPlatform: CachedRelease = {
      ...LATEST_RELEASE,
      platforms: {},
    };
    vi.mocked(getLatestRelease).mockResolvedValue(releaseWithoutPlatform);
    const res = await worker.fetch(req("/windows/x86_64/0.1.0"), env);
    expect(res.status).toBe(204);
  });

  it("returns 400 for unsupported target/arch", async () => {
    vi.mocked(getLatestRelease).mockResolvedValue(LATEST_RELEASE);
    const res = await worker.fetch(req("/android/arm64/0.1.0"), env);
    expect(res.status).toBe(400);
    const body = await res.json() as { error: string };
    expect(body.error).toContain("Unsupported target/arch");
  });

  it("returns 400 for invalid version string", async () => {
    const res = await worker.fetch(req("/windows/x86_64/latest"), env);
    expect(res.status).toBe(400);
  });

  it("returns 400 for invalid version — path traversal attempt", async () => {
    const res = await worker.fetch(req("/windows/x86_64/../etc"), env);
    expect(res.status).toBe(400);
  });

  it("returns 400 for paths with wrong segment count", async () => {
    const res = await worker.fetch(req("/windows/0.1.0"), env);
    expect(res.status).toBe(400);
    const res2 = await worker.fetch(req("/windows/x86_64/0.1.0/extra"), env);
    expect(res2.status).toBe(400);
  });

  it("returns 500 when GitHub API throws", async () => {
    vi.mocked(getLatestRelease).mockRejectedValue(new Error("Network timeout"));
    const res = await worker.fetch(req("/windows/x86_64/0.1.0"), env);
    expect(res.status).toBe(500);
  });
});

describe("GET /health", () => {
  it("returns 200 with status ok", async () => {
    const res = await worker.fetch(req("/health"), makeEnv());
    expect(res.status).toBe(200);
    const body = await res.json() as { status: string };
    expect(body.status).toBe("ok");
  });
});

describe("POST /admin/purge-cache", () => {
  const env = makeEnv();

  beforeEach(() => vi.resetAllMocks());

  it("purges cache with correct secret", async () => {
    vi.mocked(purgeCachedRelease).mockResolvedValue(undefined);
    const res = await worker.fetch(
      new Request("https://update.ozvil.app/admin/purge-cache", {
        method: "POST",
        headers: { Authorization: "Bearer test-secret-123" },
      }),
      env
    );
    expect(res.status).toBe(200);
    expect(purgeCachedRelease).toHaveBeenCalledOnce();
  });

  it("returns 401 with wrong secret", async () => {
    const res = await worker.fetch(
      new Request("https://update.ozvil.app/admin/purge-cache", {
        method: "POST",
        headers: { Authorization: "Bearer wrong-secret" },
      }),
      env
    );
    expect(res.status).toBe(401);
    expect(purgeCachedRelease).not.toHaveBeenCalled();
  });

  it("returns 401 with no auth header", async () => {
    const res = await worker.fetch(
      new Request("https://update.ozvil.app/admin/purge-cache", {
        method: "POST",
      }),
      env
    );
    expect(res.status).toBe(401);
  });

  it("returns 405 for GET /admin/purge-cache", async () => {
    const res = await worker.fetch(req("/admin/purge-cache"), env);
    expect(res.status).toBe(405);
  });
});
