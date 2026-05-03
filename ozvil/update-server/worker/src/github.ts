import type { Env, GitHubRelease, GitHubAsset } from "./types";

const GITHUB_API = "https://api.github.com";

/**
 * Fetch the latest non-draft, non-prerelease GitHub Release for the repo.
 * Uses the GITHUB_TOKEN secret if present to avoid hitting the 60 req/h
 * unauthenticated rate limit.
 */
export async function fetchLatestRelease(env: Env): Promise<GitHubRelease> {
  const url = `${GITHUB_API}/repos/${env.GITHUB_OWNER}/${env.GITHUB_REPO}/releases/latest`;

  const headers: Record<string, string> = {
    "Accept": "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
    "User-Agent": "ozvil-update-server/1.0",
  };

  if (env.GITHUB_TOKEN) {
    headers["Authorization"] = `Bearer ${env.GITHUB_TOKEN}`;
  }

  const res = await fetch(url, { headers });

  if (res.status === 404) {
    throw new NotFoundError("No releases published yet");
  }

  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`GitHub API error ${res.status}: ${body}`);
  }

  return res.json() as Promise<GitHubRelease>;
}

/**
 * Fetch a specific release by tag (e.g. "v0.2.0").
 */
export async function fetchReleaseByTag(
  env: Env,
  tag: string
): Promise<GitHubRelease> {
  const url = `${GITHUB_API}/repos/${env.GITHUB_OWNER}/${env.GITHUB_REPO}/releases/tags/${tag}`;

  const headers: Record<string, string> = {
    "Accept": "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
    "User-Agent": "ozvil-update-server/1.0",
  };

  if (env.GITHUB_TOKEN) {
    headers["Authorization"] = `Bearer ${env.GITHUB_TOKEN}`;
  }

  const res = await fetch(url, { headers });

  if (res.status === 404) {
    throw new NotFoundError(`Release ${tag} not found`);
  }

  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`GitHub API error ${res.status}: ${body}`);
  }

  return res.json() as Promise<GitHubRelease>;
}

/**
 * Download the text contents of a release asset (used for .sig files).
 */
export async function fetchAssetText(asset: GitHubAsset): Promise<string> {
  const res = await fetch(asset.browser_download_url, {
    headers: { "User-Agent": "ozvil-update-server/1.0" },
  });

  if (!res.ok) {
    throw new Error(
      `Failed to download asset ${asset.name}: HTTP ${res.status}`
    );
  }

  return res.text();
}

/**
 * Strip the leading "v" from a tag name and return a plain semver string.
 * "v0.2.0" → "0.2.0"
 */
export function tagToVersion(tag: string): string {
  return tag.replace(/^v/, "");
}

export class NotFoundError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "NotFoundError";
  }
}
