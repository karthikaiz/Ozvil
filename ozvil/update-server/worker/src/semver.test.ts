import { describe, it, expect } from "vitest";
import { isNewerVersion, isValidVersion } from "./semver";

describe("isNewerVersion", () => {
  it("detects a patch bump", () => {
    expect(isNewerVersion("0.1.1", "0.1.0")).toBe(true);
  });

  it("detects a minor bump", () => {
    expect(isNewerVersion("0.2.0", "0.1.9")).toBe(true);
  });

  it("detects a major bump", () => {
    expect(isNewerVersion("1.0.0", "0.9.9")).toBe(true);
  });

  it("returns false when versions are equal", () => {
    expect(isNewerVersion("0.1.0", "0.1.0")).toBe(false);
  });

  it("returns false when candidate is older", () => {
    expect(isNewerVersion("0.1.0", "0.2.0")).toBe(false);
    expect(isNewerVersion("0.1.0", "1.0.0")).toBe(false);
  });

  it("handles leading v prefix", () => {
    expect(isNewerVersion("v0.2.0", "0.1.0")).toBe(true);
    expect(isNewerVersion("0.2.0", "v0.1.0")).toBe(true);
  });

  it("stable release is newer than pre-release of same version", () => {
    expect(isNewerVersion("0.2.0", "0.2.0-beta.1")).toBe(true);
  });

  it("pre-release is NOT newer than same stable", () => {
    expect(isNewerVersion("0.2.0-beta.1", "0.2.0")).toBe(false);
  });

  it("later pre-release beats earlier pre-release", () => {
    expect(isNewerVersion("0.2.0-beta.2", "0.2.0-beta.1")).toBe(true);
  });

  it("handles multi-digit version numbers", () => {
    expect(isNewerVersion("1.10.0", "1.9.0")).toBe(true);
    expect(isNewerVersion("2.0.0", "1.99.99")).toBe(true);
  });
});

describe("isValidVersion", () => {
  it("accepts standard semver", () => {
    expect(isValidVersion("0.1.0")).toBe(true);
    expect(isValidVersion("1.0.0")).toBe(true);
    expect(isValidVersion("10.20.30")).toBe(true);
  });

  it("accepts pre-release versions", () => {
    expect(isValidVersion("0.1.0-beta.1")).toBe(true);
    expect(isValidVersion("1.0.0-rc.2")).toBe(true);
  });

  it("accepts versions with leading v", () => {
    expect(isValidVersion("v0.1.0")).toBe(true);
  });

  it("rejects invalid strings", () => {
    expect(isValidVersion("")).toBe(false);
    expect(isValidVersion("abc")).toBe(false);
    expect(isValidVersion("1.0")).toBe(false);
    expect(isValidVersion("latest")).toBe(false);
    expect(isValidVersion("../etc/passwd")).toBe(false);
  });
});
