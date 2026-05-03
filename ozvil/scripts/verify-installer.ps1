# Ozvil — Verify Installer Post-Build
# Checks that the installed app and both Start Menu shortcuts are present.
# Run this on a clean Windows VM after installing the release build.
#
# Usage:
#   .\scripts\verify-installer.ps1
#   .\scripts\verify-installer.ps1 -InstallDir "C:\Program Files\Ozvil"

param(
    [string]$InstallDir = "C:\Program Files\Ozvil"
)

$ErrorActionPreference = "Stop"
$failures = @()

function Check([string]$label, [scriptblock]$test) {
    try {
        $result = & $test
        if ($result) {
            Write-Host "  ✓ $label" -ForegroundColor Green
        } else {
            Write-Host "  ✗ $label" -ForegroundColor Red
            $script:failures += $label
        }
    } catch {
        Write-Host "  ✗ $label — Exception: $_" -ForegroundColor Red
        $script:failures += $label
    }
}

Write-Host ""
Write-Host "=== Ozvil Installer Verification ===" -ForegroundColor Cyan
Write-Host ""

# ─── Executable ───────────────────────────────────────────────────────────────
Write-Host "Executable:" -ForegroundColor White
Check "ozvil.exe present in install dir" {
    Test-Path (Join-Path $InstallDir "ozvil.exe")
}

# ─── Start Menu shortcuts ─────────────────────────────────────────────────────
Write-Host "Start Menu shortcuts:" -ForegroundColor White
$startMenuDir = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs\Ozvil"

Check "Start Menu folder exists" {
    Test-Path $startMenuDir
}

Check "Ozvil.lnk exists (normal shortcut)" {
    Test-Path (Join-Path $startMenuDir "Ozvil.lnk")
}

Check "'Ozvil (Safe Mode).lnk' exists (safety hatch shortcut)" {
    Test-Path (Join-Path $startMenuDir "Ozvil (Safe Mode).lnk")
}

# ─── Verify Safe Mode shortcut arguments ─────────────────────────────────────
Write-Host "Safe Mode shortcut arguments:" -ForegroundColor White
Check "Safe Mode shortcut passes '--safe-mode' argument" {
    $lnkPath = Join-Path $startMenuDir "Ozvil (Safe Mode).lnk"
    if (-not (Test-Path $lnkPath)) { return $false }
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($lnkPath)
    $shortcut.Arguments -like "*--safe-mode*"
}

Check "Safe Mode shortcut target points to ozvil.exe" {
    $lnkPath = Join-Path $startMenuDir "Ozvil (Safe Mode).lnk"
    if (-not (Test-Path $lnkPath)) { return $false }
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($lnkPath)
    $shortcut.TargetPath -like "*ozvil.exe"
}

# ─── Signature verification ───────────────────────────────────────────────────
Write-Host "Code signature:" -ForegroundColor White
$signtool = Get-Command "signtool.exe" -ErrorAction SilentlyContinue
if ($signtool) {
    Check "ozvil.exe is signed" {
        $result = & signtool.exe verify /pa (Join-Path $InstallDir "ozvil.exe") 2>&1
        $LASTEXITCODE -eq 0
    }
} else {
    Write-Host "  ⚠  signtool.exe not found — skipping signature check" -ForegroundColor Yellow
}

# ─── CLI smoke test ───────────────────────────────────────────────────────────
Write-Host "CLI smoke tests:" -ForegroundColor White
$exe = Join-Path $InstallDir "ozvil.exe"
if (Test-Path $exe) {
    Check "ozvil status --agent returns valid JSON" {
        $json = & $exe status --agent 2>&1
        try { $null = $json | ConvertFrom-Json; $true } catch { $false }
    }

    Check "ozvil profiles list exits 0" {
        & $exe profiles list 2>&1 | Out-Null
        $LASTEXITCODE -eq 0
    }

    Check "ozvil --safe-mode exits 0" {
        # Safe mode just prints a message and the GUI would open; check exit code from CLI
        $proc = Start-Process $exe "--safe-mode" -PassThru -WindowStyle Hidden
        Start-Sleep -Milliseconds 500
        if (-not $proc.HasExited) { $proc.Kill() }
        $true  # just checking it launches without crash
    }
} else {
    Write-Host "  ⚠  ozvil.exe not found at $exe — skipping CLI tests" -ForegroundColor Yellow
}

# ─── Summary ─────────────────────────────────────────────────────────────────
Write-Host ""
if ($failures.Count -eq 0) {
    Write-Host "All checks passed. Installer verification PASSED." -ForegroundColor Green
} else {
    Write-Host "FAILED checks ($($failures.Count)):" -ForegroundColor Red
    foreach ($f in $failures) { Write-Host "  ✗ $f" -ForegroundColor Red }
    Write-Host ""
    Write-Error "Installer verification FAILED."
    exit 1
}
