export interface Env {
  GITHUB_OWNER: string;
  GITHUB_REPO: string;
  GITHUB_TOKEN?: string;          // optional — raises rate limit from 60 to 5000 req/h
  CACHE_KV: KVNamespace;
  PURGE_SECRET: string;           // used by CI to purge the cache after a release
  ALLOWED_ORIGINS?: string;       // comma-separated CORS origins, optional
}

// ─── GitHub API types ────────────────────────────────────────────────────────

export interface GitHubRelease {
  tag_name: string;               // e.g. "v0.2.0"
  name: string;
  body: string | null;
  published_at: string;           // ISO 8601
  prerelease: boolean;
  draft: boolean;
  assets: GitHubAsset[];
  html_url: string;
}

export interface GitHubAsset {
  name: string;
  browser_download_url: string;
  size: number;
  content_type: string;
}

// ─── Tauri updater response ───────────────────────────────────────────────────

export interface TauriUpdateResponse {
  version: string;
  notes: string;
  pub_date: string;
  platforms: Partial<Record<TauriPlatform, TauriPlatformEntry>>;
}

export type TauriPlatform =
  | "windows-x86_64"
  | "darwin-x86_64"
  | "darwin-aarch64"
  | "linux-x86_64";

export interface TauriPlatformEntry {
  signature: string;
  url: string;
}

// ─── Internal cached release ─────────────────────────────────────────────────

export interface CachedRelease {
  version: string;                // without leading "v", e.g. "0.2.0"
  notes: string;
  pub_date: string;
  platforms: Partial<Record<TauriPlatform, TauriPlatformEntry>>;
  fetched_at: number;             // Date.now()
}

// ─── Route params ─────────────────────────────────────────────────────────────

export interface UpdateParams {
  target: string;   // e.g. "windows"
  arch: string;     // e.g. "x86_64"
  version: string;  // e.g. "0.1.0"
}
