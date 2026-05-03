# Ozvil — Full Release Build
# Runs typecheck, Rust tests, Tauri build, and optionally code signing.
#
# Usage:
#   .\scripts\build-release.ps1                        # build only
#   .\scripts\build-release.ps1 -Sign -Thumbprint "..." # build + sign
#   .\scripts\build-release.ps1 -DryRun                # validate without building
#
# Run from the ozvil/ project root.

param(
    [switch]$Sign,
    [string]$Thumbprint   = $env:OZVIL_CERT_THUMBPRINT,
    [string]$PfxPath      = $env:OZVIL_PFX_PATH,
    [string]$PfxPassword  = $env:OZVIL_PFX_PASSWORD,
    [switch]$DryRun,
    [switch]$SkipTests
)

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot\..

function Step([string]$name, [scriptblock]$block) {
    Write-Host ""
    Write-Host "── $name ──" -ForegroundColor Cyan
    & $block
    if ($LASTEXITCODE -and $LASTEXITCODE -ne 0) {
        Write-Error "$name failed (exit $LASTEXITCODE)"
        exit $LASTEXITCODE
    }
}

Write-Host ""
Write-Host "╔══════════════════════════════════╗" -ForegroundColor Blue
Write-Host "║      Ozvil Release Builder       ║" -ForegroundColor Blue
Write-Host "╚══════════════════════════════════╝" -ForegroundColor Blue

if ($DryRun) {
    Write-Host "[DRY RUN] Validation only — no artifacts will be produced" -ForegroundColor Yellow
}

# ─── Prerequisites ────────────────────────────────────────────────────────────
Step "Check prerequisites" {
    foreach ($cmd in @("cargo", "pnpm", "node")) {
        if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
            Write-Error "$cmd not found in PATH"
            exit 1
        }
    }
    Write-Host "  cargo : $(cargo --version)"
    Write-Host "  pnpm  : $(pnpm --version)"
    Write-Host "  node  : $(node --version)"
}

# ─── Install JS dependencies ──────────────────────────────────────────────────
Step "Install JS dependencies" {
    pnpm install --frozen-lockfile
}

# ─── TypeScript typecheck ─────────────────────────────────────────────────────
Step "TypeScript typecheck" {
    pnpm tsc --noEmit
}

# ─── Rust tests (mock adapter, no Windows OS calls required) ─────────────────
if (-not $SkipTests) {
    Step "Rust tests" {
        cargo test --manifest-path src-tauri/Cargo.toml --all
    }
}

if ($DryRun) {
    Write-Host ""
    Write-Host "[DRY RUN] All checks passed. No build produced." -ForegroundColor Green
    exit 0
}

# ─── Tauri production build ───────────────────────────────────────────────────
Step "Tauri production build (NSIS + MSI)" {
    $env:TAURI_ENV_PLATFORM = "windows"
    pnpm tauri build --target x86_64-pc-windows-msvc --bundles nsis,msi
}

# ─── Log artifacts ────────────────────────────────────────────────────────────
Step "Collect artifacts" {
    $bundleDir = "src-tauri\target\release\bundle"
    $artifacts = @()
    $artifacts += Get-Item "$bundleDir\nsis\*.exe" -ErrorAction SilentlyContinue
    $artifacts += Get-Item "$bundleDir\msi\*.msi"  -ErrorAction SilentlyContinue

    if ($artifacts.Count -eq 0) {
        Write-Error "No artifacts found after build"
        exit 1
    }

    foreach ($a in $artifacts) {
        $size = [math]::Round($a.Length / 1MB, 1)
        Write-Host "  $($a.Name) — ${size} MB" -ForegroundColor Green
    }
}

# ─── Optional: code signing ───────────────────────────────────────────────────
if ($Sign) {
    Step "Code signing" {
        $signArgs = @("-ArtifactsDir", "src-tauri\target\release\bundle")
        if ($Thumbprint)  { $signArgs += @("-Thumbprint", $Thumbprint) }
        if ($PfxPath)     { $signArgs += @("-PfxPath", $PfxPath) }
        if ($PfxPassword) { $signArgs += @("-PfxPassword", $PfxPassword) }
        & .\scripts\sign-release.ps1 @signArgs
    }
}

Write-Host ""
Write-Host "Release build complete." -ForegroundColor Green
Write-Host "Artifacts: src-tauri\target\release\bundle\" -ForegroundColor White
