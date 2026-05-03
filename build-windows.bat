@echo off
setlocal EnableDelayedExpansion
title Ozvil - Windows Build Setup

echo.
echo ================================================================
echo   Ozvil - One-Shot Windows Build Script
echo   Installs all prerequisites and builds the .exe installer
echo ================================================================
echo.

:: ── Admin check ─────────────────────────────────────────────────
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] This script must be run as Administrator.
    echo Right-click build-windows.bat and choose "Run as administrator".
    echo.
    pause
    exit /b 1
)

:: ── Paths ───────────────────────────────────────────────────────
set "REPO_ROOT=%~dp0"
:: Remove trailing backslash
if "%REPO_ROOT:~-1%"=="\" set "REPO_ROOT=%REPO_ROOT:~0,-1%"
set "OZVIL_DIR=%REPO_ROOT%\ozvil"
set "TAURI_DIR=%OZVIL_DIR%\src-tauri"

echo Repo root : %REPO_ROOT%
echo Ozvil dir : %OZVIL_DIR%
echo.

:: ── Helper: refresh PATH from registry ──────────────────────────
:: (choco refreshenv doesn't always work inside the same cmd session)
:refresh_path
for /f "tokens=2*" %%a in ('reg query "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v "Path" 2^>nul') do set "SYS_PATH=%%b"
for /f "tokens=2*" %%a in ('reg query "HKCU\Environment" /v "Path" 2^>nul') do set "USR_PATH=%%b"
set "PATH=%SYS_PATH%;%USR_PATH%;%PATH%"

:: ════════════════════════════════════════════════════════════════
:: STEP 1 — Chocolatey
:: ════════════════════════════════════════════════════════════════
echo [1/9] Checking Chocolatey...
where choco >nul 2>&1
if %errorLevel% neq 0 (
    echo       Installing Chocolatey...
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))"
    if !errorLevel! neq 0 (
        echo [ERROR] Chocolatey install failed.
        pause & exit /b 1
    )
    call refresh_path
    echo       Chocolatey installed.
) else (
    echo       Chocolatey already installed.
)

:: ════════════════════════════════════════════════════════════════
:: STEP 2 — Visual Studio 2022 Build Tools (MSVC + Windows SDK)
::          Rust on Windows requires the MSVC C++ toolchain.
::          This is the biggest download (~3-5 GB). Skip if already present.
:: ════════════════════════════════════════════════════════════════
echo.
echo [2/9] Checking Visual Studio Build Tools (MSVC C++ toolchain)...
if exist "%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC" (
    echo       MSVC already installed.
) else if exist "%ProgramFiles%\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC" (
    echo       MSVC already installed.
) else (
    echo       Installing VS 2022 Build Tools + Windows 10 SDK...
    echo       ^(This is a large download - ~3 GB. Please wait...^)
    choco install visualstudio2022buildtools -y --no-progress ^
        --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --add Microsoft.VisualStudio.Component.Windows10SDK.19041"
    if !errorLevel! neq 0 (
        echo [ERROR] VS Build Tools install failed. Please install manually:
        echo         https://visualstudio.microsoft.com/visual-cpp-build-tools/
        pause & exit /b 1
    )
    call refresh_path
)

:: ════════════════════════════════════════════════════════════════
:: STEP 3 — Rust
:: ════════════════════════════════════════════════════════════════
echo.
echo [3/9] Checking Rust...
where rustc >nul 2>&1
if %errorLevel% neq 0 (
    echo       Installing Rust via Chocolatey...
    choco install rust -y --no-progress
    if !errorLevel! neq 0 (
        echo [ERROR] Rust install failed. Install manually: https://rustup.rs
        pause & exit /b 1
    )
    call refresh_path
    echo       Rust installed.
) else (
    echo       Rust already installed.
)

:: Ensure MSVC target is present
echo       Adding x86_64-pc-windows-msvc target...
rustup target add x86_64-pc-windows-msvc >nul 2>&1

:: ════════════════════════════════════════════════════════════════
:: STEP 4 — Node.js LTS
:: ════════════════════════════════════════════════════════════════
echo.
echo [4/9] Checking Node.js...
where node >nul 2>&1
if %errorLevel% neq 0 (
    echo       Installing Node.js LTS...
    choco install nodejs-lts -y --no-progress
    if !errorLevel! neq 0 (
        echo [ERROR] Node.js install failed.
        pause & exit /b 1
    )
    call refresh_path
    echo       Node.js installed.
) else (
    for /f "tokens=*" %%v in ('node --version 2^>nul') do echo       Node.js %%v already installed.
)

:: ════════════════════════════════════════════════════════════════
:: STEP 5 — NSIS (for the .exe installer)
:: ════════════════════════════════════════════════════════════════
echo.
echo [5/9] Checking NSIS...
where makensis >nul 2>&1
if %errorLevel% neq 0 (
    echo       Installing NSIS...
    choco install nsis -y --no-progress
    if !errorLevel! neq 0 (
        echo [ERROR] NSIS install failed. Install manually: https://nsis.sourceforge.io/Download
        pause & exit /b 1
    )
    call refresh_path
    echo       NSIS installed.
) else (
    echo       NSIS already installed.
)

:: ════════════════════════════════════════════════════════════════
:: STEP 6 — pnpm + Tauri CLI
:: ════════════════════════════════════════════════════════════════
echo.
echo [6/9] Installing pnpm and Tauri CLI...
where pnpm >nul 2>&1
if %errorLevel% neq 0 (
    echo       Installing pnpm globally...
    call npm install -g pnpm
    call refresh_path
)
where pnpm >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] pnpm not found after install. Check npm global path.
    pause & exit /b 1
)
for /f "tokens=*" %%v in ('pnpm --version 2^>nul') do echo       pnpm %%v

:: Install/upgrade Tauri CLI
echo       Installing @tauri-apps/cli ...
call pnpm add -g @tauri-apps/cli@^2 --silent
call refresh_path

:: ════════════════════════════════════════════════════════════════
:: STEP 7 — Generate required icon and image assets
::          tauri.conf.json references icons that don't exist yet.
::          We create a placeholder source image, then use
::          `tauri icon` to expand it to all required sizes.
:: ════════════════════════════════════════════════════════════════
echo.
echo [7/9] Generating icon assets...

:: Create icons dir
if not exist "%TAURI_DIR%\icons" mkdir "%TAURI_DIR%\icons"

:: Create a 512x512 indigo PNG source icon using PowerShell + System.Drawing
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "Add-Type -AssemblyName System.Drawing; " ^
    "$sz = 512; $bmp = New-Object System.Drawing.Bitmap($sz, $sz); " ^
    "$g = [System.Drawing.Graphics]::FromImage($bmp); " ^
    "$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias; " ^
    "$bg = [System.Drawing.Color]::FromArgb(255, 99, 102, 241); " ^
    "$g.Clear($bg); " ^
    "$pen = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White); " ^
    "$font = New-Object System.Drawing.Font('Segoe UI', 220, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel); " ^
    "$fmt = New-Object System.Drawing.StringFormat; " ^
    "$fmt.Alignment = [System.Drawing.StringAlignment]::Center; " ^
    "$fmt.LineAlignment = [System.Drawing.StringAlignment]::Center; " ^
    "$rect = New-Object System.Drawing.RectangleF(0, 0, $sz, $sz); " ^
    "$g.DrawString('O', $font, $pen, $rect, $fmt); " ^
    "$bmp.Save('%TAURI_DIR%\icons\app-icon.png', [System.Drawing.Imaging.ImageFormat]::Png); " ^
    "$g.Dispose(); $bmp.Dispose(); Write-Host 'Source icon created.'"

if not exist "%TAURI_DIR%\icons\app-icon.png" (
    echo [ERROR] Failed to create source icon.
    pause & exit /b 1
)

:: Run tauri icon to generate all required sizes
cd /d "%OZVIL_DIR%"
echo       Running tauri icon generation...
call pnpm tauri icon src-tauri\icons\app-icon.png
if !errorLevel! neq 0 (
    echo [WARN] tauri icon failed. Attempting manual placeholder icons...
    :: Fallback: copy the source PNG to all required names
    copy "%TAURI_DIR%\icons\app-icon.png" "%TAURI_DIR%\icons\32x32.png" >nul
    copy "%TAURI_DIR%\icons\app-icon.png" "%TAURI_DIR%\icons\128x128.png" >nul
    copy "%TAURI_DIR%\icons\app-icon.png" "%TAURI_DIR%\icons\128x128@2x.png" >nul
    :: Create minimal .ico using PowerShell
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "Add-Type -AssemblyName System.Drawing; " ^
        "$bmp = [System.Drawing.Bitmap]::FromFile('%TAURI_DIR%\icons\app-icon.png'); " ^
        "$icon = [System.Drawing.Icon]::FromHandle($bmp.GetHicon()); " ^
        "$fs = [System.IO.File]::Create('%TAURI_DIR%\icons\icon.ico'); " ^
        "$icon.Save($fs); $fs.Dispose(); $bmp.Dispose(); Write-Host '.ico created.'"
    :: .icns is only needed for macOS; create an empty placeholder
    copy nul "%TAURI_DIR%\icons\icon.icns" >nul
)

:: Create tray icon (copy from 32x32 if tauri icon created it, else from source)
if not exist "%TAURI_DIR%\icons\tray.png" (
    if exist "%TAURI_DIR%\icons\32x32.png" (
        copy "%TAURI_DIR%\icons\32x32.png" "%TAURI_DIR%\icons\tray.png" >nul
    ) else (
        copy "%TAURI_DIR%\icons\app-icon.png" "%TAURI_DIR%\icons\tray.png" >nul
    )
)

:: Create NSIS installer images (header: 150x57, sidebar: 164x314)
echo       Creating NSIS installer images...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "Add-Type -AssemblyName System.Drawing; " ^
    "$c = [System.Drawing.Color]::FromArgb(255, 99, 102, 241); " ^
    "$h = New-Object System.Drawing.Bitmap(150, 57); " ^
    "$g = [System.Drawing.Graphics]::FromImage($h); $g.Clear($c); $g.Dispose(); " ^
    "$h.Save('%TAURI_DIR%\nsis\header.bmp', [System.Drawing.Imaging.ImageFormat]::Bmp); $h.Dispose(); " ^
    "$s = New-Object System.Drawing.Bitmap(164, 314); " ^
    "$g = [System.Drawing.Graphics]::FromImage($s); $g.Clear($c); $g.Dispose(); " ^
    "$s.Save('%TAURI_DIR%\nsis\sidebar.bmp', [System.Drawing.Imaging.ImageFormat]::Bmp); $s.Dispose(); " ^
    "Write-Host 'NSIS images created.'"

:: ════════════════════════════════════════════════════════════════
:: STEP 8 — Install frontend dependencies
:: ════════════════════════════════════════════════════════════════
echo.
echo [8/9] Installing frontend dependencies (pnpm install)...
cd /d "%OZVIL_DIR%"
call pnpm install
if %errorLevel% neq 0 (
    echo [ERROR] pnpm install failed.
    pause & exit /b 1
)

:: ════════════════════════════════════════════════════════════════
:: STEP 9 — Build Ozvil (NSIS .exe installer)
::          --bundles nsis  →  skip MSI (avoids needing WiX Toolset)
::          --target        →  explicit Windows MSVC triple
:: ════════════════════════════════════════════════════════════════
echo.
echo [9/9] Building Ozvil...
echo       This compiles Rust + React and creates the installer.
echo       First build takes 5-15 minutes. Subsequent builds are faster.
echo.
cd /d "%OZVIL_DIR%"
call pnpm tauri build --target x86_64-pc-windows-msvc --bundles nsis
if %errorLevel% neq 0 (
    echo.
    echo [ERROR] Build failed. Common causes:
    echo   - MSVC toolchain not found: make sure VS Build Tools finished installing
    echo     and try running this script again from a fresh Administrator prompt.
    echo   - Rust compiler error: check the output above for details.
    echo   - Missing WebView2: install from https://developer.microsoft.com/microsoft-edge/webview2/
    echo.
    pause & exit /b 1
)

:: ════════════════════════════════════════════════════════════════
:: Done!
:: ════════════════════════════════════════════════════════════════
echo.
echo ================================================================
echo   BUILD SUCCESSFUL
echo ================================================================
echo.
echo   Installer location:
echo   %OZVIL_DIR%\src-tauri\target\release\bundle\nsis\
echo.
echo   Find the file:  Ozvil_0.1.0_x64-setup.exe
echo.
echo   Double-click it to install Ozvil.
echo   After installing, look for "Ozvil" and "Ozvil (Safe Mode)"
echo   in the Windows Start Menu.
echo.

:: Open the bundle folder in Explorer
explorer "%OZVIL_DIR%\src-tauri\target\release\bundle\nsis"

pause
