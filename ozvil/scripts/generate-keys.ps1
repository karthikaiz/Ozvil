# Ozvil — Generate Updater Signing Keys
# Run this ONCE on a secure machine before your first release.
# Keep the private key safe and never commit it to source control.
#
# Usage:
#   .\scripts\generate-keys.ps1
#   .\scripts\generate-keys.ps1 -OutputDir "C:\secrets\ozvil-keys"
#
# After running, copy the public key into tauri.conf.json > plugins.updater.pubkey
# and store the private key + password as GitHub Actions secrets.

param(
    [string]$OutputDir = "$PSScriptRoot\..\keys"
)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "=== Ozvil Updater Key Generation ===" -ForegroundColor Cyan
Write-Host ""

# Check that cargo/tauri-cli is available
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Error "cargo not found. Install Rust from https://rustup.rs"
    exit 1
}

# Install tauri-cli if not present
$tauriCli = cargo install --list 2>$null | Select-String "tauri-cli"
if (-not $tauriCli) {
    Write-Host "Installing tauri-cli..." -ForegroundColor Yellow
    cargo install tauri-cli --version "^2"
}

# Create output directory
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
$privateKeyPath = Join-Path $OutputDir "ozvil-updater.key"
$publicKeyPath  = Join-Path $OutputDir "ozvil-updater.key.pub"

if (Test-Path $privateKeyPath) {
    Write-Warning "Key already exists at $privateKeyPath"
    $overwrite = Read-Host "Overwrite? (y/N)"
    if ($overwrite -ne "y" -and $overwrite -ne "Y") {
        Write-Host "Aborted." -ForegroundColor Yellow
        exit 0
    }
}

Write-Host "Generating key pair..." -ForegroundColor Green
$password = Read-Host "Enter a strong password for the private key" -AsSecureString
$passwordPlain = [Runtime.InteropServices.Marshal]::PtrToStringAuto(
    [Runtime.InteropServices.Marshal]::SecureStringToBSTR($password)
)

$env:TAURI_KEY_PASSWORD = $passwordPlain
cargo tauri signer generate -w $privateKeyPath 2>&1

Write-Host ""
Write-Host "=== Keys Generated ===" -ForegroundColor Green
Write-Host ""
Write-Host "Private key : $privateKeyPath" -ForegroundColor Yellow
Write-Host "Public key  : $publicKeyPath" -ForegroundColor Yellow
Write-Host ""

$pubKey = Get-Content $publicKeyPath -Raw
Write-Host "PUBLIC KEY (paste into tauri.conf.json > plugins.updater.pubkey):" -ForegroundColor Cyan
Write-Host $pubKey -ForegroundColor White

Write-Host ""
Write-Host "=== Next Steps ===" -ForegroundColor Cyan
Write-Host "1. Copy the public key above into tauri.conf.json > plugins.updater.pubkey"
Write-Host "2. Add these GitHub Actions secrets:"
Write-Host "   TAURI_SIGNING_PRIVATE_KEY  = (contents of $privateKeyPath)"
Write-Host "   TAURI_SIGNING_PRIVATE_KEY_PASSWORD = (the password you just entered)"
Write-Host "3. Delete the private key from this machine after backing it up securely"
Write-Host "4. NEVER commit the private key or password to source control"
Write-Host ""

# Security reminder
Write-Warning "The private key at $privateKeyPath must be kept secret."
Write-Warning "Anyone with this key can push silent updates to all Ozvil users."
