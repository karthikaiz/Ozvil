/**
 * Minimal semver comparison — avoids a full semver package dependency.
 * Only handles the subset Tauri version strings use: MAJOR.MINOR.PATCH
 * and MAJOR.MINOR.PATCH-prerelease (pre-releases are always < release).
 */

interface ParsedVersion {
  major: number;
  minor: number;
  patch: number;
  pre: string; // "" for stable releases
}

function parse(v: string): ParsedVersion {
  // Strip leading "v"
  const clean = v.replace(/^v/, "");
  const [numeric, pre = ""] = clean.split("-", 2);
  const parts = numeric.split(".").map(Number);
  return {
    major: parts[0] ?? 0,
    minor: parts[1] ?? 0,
    patch: parts[2] ?? 0,
    pre,
  };
}

/**
 * Returns true if `candidate` is strictly greater than `current`.
 */
export function isNewerVersion(candidate: string, current: string): boolean {
  const a = parse(candidate);
  const b = parse(current);

  if (a.major !== b.major) return a.major > b.major;
  if (a.minor !== b.minor) return a.minor > b.minor;
  if (a.patch !== b.patch) return a.patch > b.patch;

  // Equal numeric parts: stable > pre-release
  if (a.pre === "" && b.pre !== "") return true;
  if (a.pre !== "" && b.pre === "") return false;

  // Both pre-release: lexicographic comparison
  return a.pre > b.pre;
}

/**
 * Returns true if `v` looks like a valid semver string.
 */
export function isValidVersion(v: string): boolean {
  return /^v?\d+\.\d+\.\d+(-[\w.]+)?$/.test(v);
}
