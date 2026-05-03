# Ozvil — Code Signing & Updater Keys

## Overview

Ozvil uses two separate keys:

| Key | Purpose | Where stored |
|---|---|---|
| **Code signing certificate** (.pfx / thumbprint) | Authenticode — tells Windows the installer is from a known publisher. Prevents SmartScreen warnings. | GitHub secret `WINDOWS_CERTIFICATE` (base64) + `WINDOWS_CERTIFICATE_PASSWORD` |
| **Tauri updater signing key** (ed25519) | Signs update artifacts so the running app can verify downloaded updates haven't been tampered with. | GitHub secret `TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` |

---

## Step 1 — Generate the Tauri updater key pair (one time only)

Run this on a secure machine:

```powershell
.\scripts\generate-keys.ps1
```

This calls `cargo tauri signer generate` and outputs:

```
ozvil-updater.key       ← PRIVATE — never commit this
ozvil-updater.key.pub   ← PUBLIC — paste into tauri.conf.json
```

**Paste the public key** into `tauri.conf.json`:

```json
"plugins": {
  "updater": {
    "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk..."
  }
}
```

**Store the private key** as a GitHub Actions secret:

```
TAURI_SIGNING_PRIVATE_KEY          = <contents of ozvil-updater.key>
TAURI_SIGNING_PRIVATE_KEY_PASSWORD = <password you chose>
```

> ⚠️ Anyone with the private key can push silent updates to all Ozvil users.
> Back it up securely (password manager, HSM, or encrypted vault) and delete
> the file from your dev machine.

---

## Step 2 — Obtain a code signing certificate

### Option A: OV/EV certificate (recommended for release)

Purchase an OV (Organization Validated) or EV (Extended Validation) certificate
from a trusted CA:

- [DigiCert](https://www.digicert.com/code-signing/)
- [Sectigo](https://sectigo.com/code-signing-certificates)
- [GlobalSign](https://www.globalsign.com/en/code-signing-certificate/)

EV certificates bypass SmartScreen reputation requirements immediately.
OV certificates build SmartScreen reputation over time.

Export the certificate as a `.pfx` file with a strong password.

### Option B: Self-signed certificate (dev / internal testing only)

```powershell
# Creates a self-signed cert — NOT trusted by Windows SmartScreen
$cert = New-SelfSignedCertificate `
  -Subject "CN=Ozvil Dev, O=Ozvil, C=US" `
  -Type CodeSigningCert `
  -KeyAlgorithm RSA `
  -KeyLength 4096 `
  -CertStoreLocation "Cert:\CurrentUser\My" `
  -NotAfter (Get-Date).AddYears(3)

$password = ConvertTo-SecureString "YourPassword" -AsPlainText -Force
Export-PfxCertificate `
  -Cert $cert `
  -FilePath ozvil-dev.pfx `
  -Password $password
```

---

## Step 3 — Add the certificate to GitHub Actions

```powershell
# Base64-encode your .pfx file for the GitHub secret
[Convert]::ToBase64String([IO.File]::ReadAllBytes("ozvil.pfx")) | clip
```

Add to GitHub repository secrets:

```
WINDOWS_CERTIFICATE          = <base64 string from above>
WINDOWS_CERTIFICATE_PASSWORD = <pfx password>
```

---

## Step 4 — Configure the timestamp server

`tauri.conf.json` is already set to:

```json
"timestampUrl": "http://timestamp.digicert.com"
```

This ensures signatures remain valid after the certificate expires.
DigiCert's RFC3161 timestamp server is free and highly available.
Alternatives: `http://timestamp.sectigo.com`, `http://tsa.starfieldtech.com`.

---

## Step 5 — Local signing test

Before pushing to CI, test signing locally:

```powershell
# Build without signing
pnpm tauri build --bundles nsis,msi

# Sign with PFX
.\scripts\sign-release.ps1 `
  -PfxPath ".\ozvil.pfx" `
  -PfxPassword "YourPassword"

# Or sign with thumbprint (after importing cert to Windows cert store)
.\scripts\sign-release.ps1 -Thumbprint "ABCDEF1234..."

# Verify
signtool verify /pa src-tauri\target\release\bundle\nsis\*.exe
```

---

## Step 6 — Verify the full CI pipeline

Push a tag to trigger the release workflow:

```bash
git tag v0.1.0-beta.1
git push origin v0.1.0-beta.1
```

The workflow will:
1. Run TypeScript typecheck + Rust unit tests
2. Import the code signing certificate from secrets
3. Build NSIS + MSI installers (Tauri applies Authenticode signing)
4. Sign update artifacts with the Tauri updater private key (produces `.sig` files)
5. Upload all artifacts to the GitHub Release (as draft)

After verifying the draft release, publish it and update your update server's
`latest.json` with the new version info and `.sig` contents.

---

## Key rotation

If either key is compromised:

**Tauri updater key:**
1. Generate a new key pair (`scripts/generate-keys.ps1`)
2. Update `tauri.conf.json` with the new public key
3. Ship a new release — the old public key will no longer accept new update signatures
4. Update GitHub secrets with the new private key

**Code signing certificate:**
1. Revoke the old certificate with your CA
2. Obtain and configure a new certificate
3. Update `WINDOWS_CERTIFICATE` and `WINDOWS_CERTIFICATE_PASSWORD` secrets
4. Re-sign any already-built artifacts if needed

---

## SmartScreen reputation

New certificates (especially OV) start with no SmartScreen reputation.
Windows may show a "Windows protected your PC" warning for the first few
hundred/thousand installations. This clears automatically as installs accumulate.
EV certificates skip this entirely.

To improve SmartScreen trust faster:
- Submit your installer to Microsoft's Defender Intelligence portal for analysis
- Ensure your website and installer URL match the certificate Subject
- Keep the certificate valid and renew before expiry
