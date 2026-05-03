import type {
  Env,
  GitHubRelease,
  CachedRelease,
  TauriPlatform,
  TauriPlatformEntry,
} from "./types";
import { fetchLatestRelease, fetchAssetText, tagToVersion } from "./github";
import { getCachedRelease, setCachedRelease } from "./cache";

/**
 * Asset name → Tauri platform mappings.
 *
 * Tauri v2 NSIS build produces:
 *   Ozvil_0.2.0_x64-setup.exe      ← installer
 *   Ozvil_0.2.0_x64-setup.exe.sig  ← ed25519 signature
 *
 * MSI build produces:
 *   Ozvil_0.2.0_x64_en-US.msi
 *   Ozvil_0.2.0_x64_en-US.msi.sig
 *
 * We prefer the NSIS .exe for the windows-x86_64 platform entry since it is
 * the recommended installer. If only the MSI is present we fall back to it.
 */
const PLATFORM_PATTERNS: Array<{
  platform: TauriPlatform;
  installerPattern: RegExp;
  sigPattern: RegExp;
}> = [
  {
    platform: "windows-x86_64",
    // NSIS preferred
    installerPattern: /^Ozvil_[\d.]+-setup\.exe$/i,
    sigPattern: /^Ozvil_[\d.]+-setup\.exe\.sig$/i,
  },
  {
    platform: "windows-x86_64",
    // MSI fallback (same platform key — will be skipped if NSIS already found)
    installerPattern: /^Ozvil_[\d.]+_x64_en-US\.msi$/i,
    sigPattern: /^Ozvil_[\d.]+_x64_en-US\.msi\.sig$/i,
  },
];

/**
 * Resolve a release into the CachedRelease format.
 * Downloads .sig file contents for each platform.
 */
async function resolveRelease(
  release: GitHubRelease
): Promise<CachedRelease> {
  const version = tagToVersion(release.tag_name);
  const platforms: Partial<Record<TauriPlatform, TauriPlatformEntry>> = {};

  for (const mapping of PLATFORM_PATTERNS) {
    // Skip if we already have an entry for this platform
    if (platforms[mapping.platform]) continue;

    const installer = release.assets.find((a) =>
      mapping.installerPattern.test(a.name)
    );
    const sigAsset = release.assets.find((a) =>
      mapping.sigPattern.test(a.name)
    );

    if (!installer || !sigAsset) continue;

    try {
      const signature = await fetchAssetText(sigAsset);
      platforms[mapping.platform] = {
        signature: signature.trim(),
        url: installer.browser_download_url,
      };
    } catch (err) {
      // Log but continue — a missing .sig is non-fatal for other platforms
      console.error(
        `Failed to fetch sig for ${mapping.platform}: ${err}`
      );
    }
  }

  return {
    version,
    notes: release.body ?? "",
    pub_date: release.published_at,
    platforms,
    fetched_at: Date.now(),
  };
}

/**
 * Get the latest release, using KV cache when possible.
 * Falls back to a live GitHub API call on cache miss.
 */
export async function getLatestRelease(
  env: Env
): Promise<CachedRelease | null> {
  // 1. Try cache first
  const cached = await getCachedRelease(env);
  if (cached) return cached;

  // 2. Fetch from GitHub
  let ghRelease: GitHubRelease;
  try {
    ghRelease = await fetchLatestRelease(env);
  } catch (err) {
    if ((err as Error).name === "NotFoundError") return null;
    throw err;
  }

  // Skip drafts and pre-releases for stable update channel
  if (ghRelease.draft || ghRelease.prerelease) return null;

  // 3. Resolve .sig files and build the cached entry
  const resolved = await resolveRelease(ghRelease);

  // 4. Write to cache (non-blocking)
  await setCachedRelease(env, resolved);

  return resolved;
}
