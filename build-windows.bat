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
if "%REPO_ROOT:~-1%"=="\" set "REPO_ROOT=%REPO_ROOT:~0,-1%"
set "OZVIL_DIR=%REPO_ROOT%\ozvil"
set "TAURI_DIR=%OZVIL_DIR%\src-tauri"

echo Repo root : %REPO_ROOT%
echo Ozvil dir : %OZVIL_DIR%
echo.

:: ════════════════════════════════════════════════════════════════
:: STEP 1 — Chocolatey
:: ════════════════════════════════════════════════════════════════
echo [1/8] Checking Chocolatey...
where choco >nul 2>&1
if %errorLevel% neq 0 (
    echo       Installing Chocolatey...
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))"
    if !errorLevel! neq 0 (
        echo [ERROR] Chocolatey install failed.
        pause & exit /b 1
    )
    call :refresh_path
    echo       Chocolatey installed.
) else (
    echo       Chocolatey already installed.
)

:: ════════════════════════════════════════════════════════════════
:: STEP 2 — Visual Studio 2022 Build Tools (MSVC + Windows SDK)
:: ════════════════════════════════════════════════════════════════
echo.
echo [2/8] Checking Visual Studio Build Tools (MSVC C++ toolchain)...
if exist "%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC" (
    echo       MSVC already installed.
) else if exist "%ProgramFiles%\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC" (
    echo       MSVC already installed.
) else (
    echo       Installing VS 2022 Build Tools + Windows 10 SDK...
    echo       ^(Large download ~3 GB. Please wait...^)
    choco install visualstudio2022buildtools -y --no-progress ^
        --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --add Microsoft.VisualStudio.Component.Windows10SDK.19041"
    if !errorLevel! neq 0 (
        echo [ERROR] VS Build Tools install failed. Install manually:
        echo         https://visualstudio.microsoft.com/visual-cpp-build-tools/
        pause & exit /b 1
    )
    call :refresh_path
)

:: ════════════════════════════════════════════════════════════════
:: STEP 3 — Rust via rustup (MSVC toolchain — NOT GNU)
:: ════════════════════════════════════════════════════════════════
echo.
echo [3/8] Checking Rust (MSVC toolchain)...

:: ── Force correct paths FIRST — Chocolatey shims in ProgramData\chocolatey\bin
:: ── can shadow the real rustup/cargo in %USERPROFILE%\.cargo\bin.
:: ── Setting these before any rustup call ensures the right home is used.
set "RUSTUP_HOME=%USERPROFILE%\.rustup"
set "CARGO_HOME=%USERPROFILE%\.cargo"
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

where rustup >nul 2>&1
if %errorLevel% neq 0 (
    echo       Installing Rustup...
    choco install rustup.install -y --no-progress
    if !errorLevel! neq 0 (
        echo [ERROR] Rustup install failed. Install manually: https://rustup.rs
        pause & exit /b 1
    )
    call :refresh_path
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

echo       RUSTUP_HOME : %RUSTUP_HOME%
echo       CARGO_HOME  : %CARGO_HOME%
echo       rustup from :
where rustup 2>nul

echo       Setting MSVC toolchain as default...
rustup default stable-msvc
if !errorLevel! neq 0 (
    echo [ERROR] Failed to set stable-msvc as default toolchain.
    pause & exit /b 1
)

echo       Adding x86_64-pc-windows-msvc target...
rustup target add x86_64-pc-windows-msvc
if !errorLevel! neq 0 (
    echo [ERROR] Failed to add x86_64-pc-windows-msvc target.
    pause & exit /b 1
)

:: ── Verify sysroot .rlib files actually exist (not just the directory) ───────
echo       Verifying sysroot files on disk...
set "_SYSROOT=%RUSTUP_HOME%\toolchains\stable-x86_64-pc-windows-msvc"
set "_TARGET_LIB=%_SYSROOT%\lib\rustlib\x86_64-pc-windows-msvc\lib"

echo       Sysroot: %_SYSROOT%
dir "%_TARGET_LIB%\libcore*.rlib" >nul 2>&1
if !errorLevel! neq 0 (
    echo       [WARN] libcore not found in sysroot — reinstalling toolchain...
    rustup toolchain uninstall stable-x86_64-pc-windows-msvc
    rustup toolchain install stable-x86_64-pc-windows-msvc
    dir "%_TARGET_LIB%\libcore*.rlib" >nul 2>&1
    if !errorLevel! neq 0 (
        echo [ERROR] libcore still missing after reinstall: %_TARGET_LIB%
        pause & exit /b 1
    )
    echo       [OK] Toolchain reinstalled — libcore found.
) else (
    echo       [OK] libcore found in sysroot.
)

:: ── Point tauri/cargo to the real cargo binary (bypasses rustup shim chain) ─
:: ── This prevents the Node.js subprocess from resolving a wrong cargo      ─
set "CARGO=%_SYSROOT%\bin\cargo.exe"
set "RUSTC=%_SYSROOT%\bin\rustc.exe"
echo       CARGO: %CARGO%

for /f "tokens=*" %%v in ('"%RUSTC%" --version 2^>nul') do echo       %%v

:: ════════════════════════════════════════════════════════════════
:: STEP 4 — Node.js LTS
:: ════════════════════════════════════════════════════════════════
echo.
echo [4/8] Checking Node.js...
where node >nul 2>&1
if %errorLevel% neq 0 (
    echo       Installing Node.js LTS...
    choco install nodejs-lts -y --no-progress
    if !errorLevel! neq 0 (
        echo [ERROR] Node.js install failed.
        pause & exit /b 1
    )
    call :refresh_path
    echo       Node.js installed.
) else (
    for /f "tokens=*" %%v in ('node --version 2^>nul') do echo       Node.js %%v already installed.
)

:: ════════════════════════════════════════════════════════════════
:: STEP 5 — NSIS (for the .exe installer)
:: ════════════════════════════════════════════════════════════════
echo.
echo [5/8] Checking NSIS...
where makensis >nul 2>&1
if %errorLevel% neq 0 (
    echo       Installing NSIS...
    choco install nsis -y --no-progress
    if !errorLevel! neq 0 (
        echo [ERROR] NSIS install failed. https://nsis.sourceforge.io/Download
        pause & exit /b 1
    )
    call :refresh_path
    echo       NSIS installed.
) else (
    echo       NSIS already installed.
)

:: ════════════════════════════════════════════════════════════════
:: STEP 6 — npm install inside ozvil/ (NOT pnpm — avoids workspace
::          symlink issues on exFAT/FAT32 drives entirely)
::          This installs @tauri-apps/cli locally so tauri is
::          available as node_modules\.bin\tauri in the next step.
:: ════════════════════════════════════════════════════════════════
echo.
echo [6/8] Installing ozvil frontend dependencies...
echo       Using npm (avoids pnpm workspace symlink issues on exFAT)...
cd /d "%OZVIL_DIR%"
call npm install
if %errorLevel% neq 0 (
    echo [ERROR] npm install failed.
    pause & exit /b 1
)

:: ════════════════════════════════════════════════════════════════
:: STEP 7 — Generate required icon and image assets
::          Now that node_modules exists, tauri CLI is available
::          locally at node_modules\.bin\tauri
:: ════════════════════════════════════════════════════════════════
echo.
echo [7/8] Generating icon assets...

if not exist "%TAURI_DIR%\icons" mkdir "%TAURI_DIR%\icons"

:: Create a 512x512 indigo PNG source icon
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "Add-Type -AssemblyName System.Drawing; $sz=512; $bmp=New-Object System.Drawing.Bitmap($sz,$sz); $g=[System.Drawing.Graphics]::FromImage($bmp); $g.SmoothingMode=[System.Drawing.Drawing2D.SmoothingMode]::AntiAlias; $g.Clear([System.Drawing.Color]::FromArgb(255,99,102,241)); $pen=New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White); $font=New-Object System.Drawing.Font('Segoe UI',220,[System.Drawing.FontStyle]::Bold,[System.Drawing.GraphicsUnit]::Pixel); $fmt=New-Object System.Drawing.StringFormat; $fmt.Alignment=[System.Drawing.StringAlignment]::Center; $fmt.LineAlignment=[System.Drawing.StringAlignment]::Center; $g.DrawString('O',$font,$pen,(New-Object System.Drawing.RectangleF(0,0,$sz,$sz)),$fmt); $bmp.Save('%TAURI_DIR%\icons\app-icon.png',[System.Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $bmp.Dispose(); Write-Host 'Source icon created.'"

if not exist "%TAURI_DIR%\icons\app-icon.png" (
    echo [ERROR] Failed to create source icon.
    pause & exit /b 1
)

:: Use local tauri (from node_modules) to generate all icon sizes
cd /d "%OZVIL_DIR%"
echo       Running tauri icon generation...
call node_modules\.bin\tauri icon "src-tauri\icons\app-icon.png"
if !errorLevel! neq 0 (
    echo [WARN] tauri icon failed - using manual placeholders...
    copy "%TAURI_DIR%\icons\app-icon.png" "%TAURI_DIR%\icons\32x32.png" >nul
    copy "%TAURI_DIR%\icons\app-icon.png" "%TAURI_DIR%\icons\128x128.png" >nul
    copy "%TAURI_DIR%\icons\app-icon.png" "%TAURI_DIR%\icons\128x128@2x.png" >nul
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "Add-Type -AssemblyName System.Drawing; $bmp=[System.Drawing.Bitmap]::FromFile('%TAURI_DIR%\icons\app-icon.png'); $icon=[System.Drawing.Icon]::FromHandle($bmp.GetHicon()); $fs=[System.IO.File]::Create('%TAURI_DIR%\icons\icon.ico'); $icon.Save($fs); $fs.Dispose(); $bmp.Dispose(); Write-Host '.ico created.'"
    copy nul "%TAURI_DIR%\icons\icon.icns" >nul
)

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
    "Add-Type -AssemblyName System.Drawing; $c=[System.Drawing.Color]::FromArgb(255,99,102,241); $h=New-Object System.Drawing.Bitmap(150,57); $g=[System.Drawing.Graphics]::FromImage($h); $g.Clear($c); $g.Dispose(); $h.Save('%TAURI_DIR%\nsis\header.bmp',[System.Drawing.Imaging.ImageFormat]::Bmp); $h.Dispose(); $s=New-Object System.Drawing.Bitmap(164,314); $g=[System.Drawing.Graphics]::FromImage($s); $g.Clear($c); $g.Dispose(); $s.Save('%TAURI_DIR%\nsis\sidebar.bmp',[System.Drawing.Imaging.ImageFormat]::Bmp); $s.Dispose(); Write-Host 'NSIS images created.'"

:: ════════════════════════════════════════════════════════════════
:: STEP 8 — Build Ozvil using local tauri CLI
:: ════════════════════════════════════════════════════════════════
echo.
echo [8/8] Building Ozvil...
echo       Compiling Rust + React. First build takes 5-15 minutes.
echo.

:: Lock the toolchain so cargo cannot pick a different one
set "RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-msvc"

cd /d "%OZVIL_DIR%"
call node_modules\.bin\tauri build --target x86_64-pc-windows-msvc --bundles nsis
if %errorLevel% neq 0 (
    echo.
    echo [ERROR] Build failed. Common causes:
    echo   - Wrong Rust toolchain: run  rustup default stable-msvc
    echo   - MSVC linker missing:  re-run this script to reinstall VS Build Tools
    echo   - Missing WebView2:     https://developer.microsoft.com/microsoft-edge/webview2/
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
echo   Installer: %OZVIL_DIR%\src-tauri\target\release\bundle\nsis\Ozvil_0.1.0_x64-setup.exe
echo.
echo   Double-click to install. Look for "Ozvil" and "Ozvil (Safe Mode)"
echo   in your Windows Start Menu.
echo.

explorer "%OZVIL_DIR%\src-tauri\target\release\bundle\nsis"
pause
goto :eof

:: ════════════════════════════════════════════════════════════════
:: Subroutine: refresh PATH from registry
:: ════════════════════════════════════════════════════════════════
:refresh_path
for /f "tokens=2*" %%a in ('reg query "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v "Path" 2^>nul') do set "SYS_PATH=%%b"
for /f "tokens=2*" %%a in ('reg query "HKCU\Environment" /v "Path" 2^>nul') do set "USR_PATH=%%b"
set "PATH=%SYS_PATH%;%USR_PATH%;%PATH%"
goto :eof
