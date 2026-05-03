# Ozvil — Sign Release Artifacts
# Signs the built NSIS/MSI/EXE installers with a code-signing certificate.
#
# Usage (certificate thumbprint, recommended for CI):
#   .\scripts\sign-release.ps1 -Thumbprint "ABCD1234..." -ArtifactsDir ".\src-tauri\target\release\bundle"
#
# Usage (PFX file, local signing):
#   .\scripts\sign-release.ps1 -PfxPath "C:\certs\ozvil.pfx" -PfxPassword "mypassword" -ArtifactsDir ".\src-tauri\target\release\bundle"
#
# Notes:
#   - signtool.exe must be in PATH (part of Windows SDK)
#   - TimestampUrl uses DigiCert RFC3161 by default (change if using another CA)
#   - Run from the ozvil/ project root

param(
    [string]$Thumbprint      = $env:OZVIL_CERT_THUMBPRINT,
    [string]$PfxPath         = $env:OZVIL_PFX_PATH,
    [string]$PfxPassword     = $env:OZVIL_PFX_PASSWORD,
    [string]$TimestampUrl    = "http://timestamp.digicert.com",
    [string]$ArtifactsDir    = ".\src-tauri\target\release\bundle",
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

# ─── Locate signtool ──────────────────────────────────────────────────────────
$signtool = Get-Command "signtool.exe" -ErrorAction SilentlyContinue
if (-not $signtool) {
    # Try common Windows SDK paths
    $sdkPaths = @(
        "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe",
        "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe"
    )
    foreach ($p in $sdkPaths) {
        if (Test-Path $p) { $signtool = $p; break }
    }
    if (-not $signtool) {
        Write-Error "signtool.exe not found. Install the Windows SDK: https://developer.microsoft.com/windows/downloads/windows-sdk/"
        exit 1
    }
} else {
    $signtool = $signtool.Source
}

Write-Host "signtool: $signtool" -ForegroundColor Gray

# ─── Find artifacts ───────────────────────────────────────────────────────────
$patterns = @(
    "$ArtifactsDir\nsis\*.exe",
    "$ArtifactsDir\msi\*.msi"
)

$artifacts = @()
foreach ($pat in $patterns) {
    $artifacts += Get-Item $pat -ErrorAction SilentlyContinue
}

if ($artifacts.Count -eq 0) {
    Write-Error "No artifacts found in $ArtifactsDir. Run 'pnpm tauri build' first."
    exit 1
}

Write-Host ""
Write-Host "=== Artifacts to sign ===" -ForegroundColor Cyan
foreach ($a in $artifacts) { Write-Host "  $($a.FullName)" }
Write-Host ""

if ($DryRun) {
    Write-Host "[DRY RUN] No signing performed." -ForegroundColor Yellow
    exit 0
}

# ─── Build signtool arguments ─────────────────────────────────────────────────
$baseArgs = @(
    "sign",
    "/fd", "sha256",
    "/tr", $TimestampUrl,
    "/td", "sha256",
    "/d", "Ozvil",
    "/du", "https://ozvil.app"
)

if ($Thumbprint) {
    $baseArgs += @("/sha1", $Thumbprint)
} elseif ($PfxPath) {
    if (-not (Test-Path $PfxPath)) {
        Write-Error "PFX file not found: $PfxPath"
        exit 1
    }
    $baseArgs += @("/f", $PfxPath)
    if ($PfxPassword) {
        $baseArgs += @("/p", $PfxPassword)
    }
} else {
    Write-Error "Provide either -Thumbprint or -PfxPath"
    exit 1
}

# ─── Sign each artifact ───────────────────────────────────────────────────────
$failed = @()
foreach ($artifact in $artifacts) {
    Write-Host "Signing: $($artifact.Name) ..." -ForegroundColor Yellow
    $args = $baseArgs + @($artifact.FullName)
    & $signtool @args
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "Failed to sign: $($artifact.FullName)"
        $failed += $artifact.FullName
    } else {
        Write-Host "  ✓ Signed" -ForegroundColor Green
    }
}

# ─── Verify signatures ────────────────────────────────────────────────────────
Write-Host ""
Write-Host "=== Verifying signatures ===" -ForegroundColor Cyan
foreach ($artifact in $artifacts) {
    & $signtool verify /pa /v $artifact.FullName 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ $($artifact.Name)" -ForegroundColor Green
    } else {
        Write-Warning "  ✗ Verification failed: $($artifact.Name)"
        $failed += $artifact.FullName
    }
}

if ($failed.Count -gt 0) {
    Write-Host ""
    Write-Error "Signing failed for $($failed.Count) artifact(s):`n$($failed -join "`n")"
    exit 1
}

Write-Host ""
Write-Host "All artifacts signed and verified successfully." -ForegroundColor Green
