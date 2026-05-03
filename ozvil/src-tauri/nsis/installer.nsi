; Ozvil NSIS Installer
; Extends Tauri's default NSIS template with the Safe Mode Start Menu shortcut.
; See: https://v2.tauri.app/distribute/windows-installer/
;
; Tauri injects the standard sections (Files, Uninstaller, etc.).
; This script adds the Safe Mode shortcut in the .onInstSuccess callback
; and removes it in the un.onUninstSuccess callback.

; ─── Safe Mode Shortcut ───────────────────────────────────────────────────────

; Called by Tauri after the main installation completes successfully.
!macro customInstall
  ; Create "Ozvil (Safe Mode)" in the same Start Menu folder as the main shortcut.
  ; Per spec: "The Windows installer must create a dedicated Start Menu shortcut
  ; named 'Ozvil (Safe Mode)' that passes the safe-mode flag."

  SetShellVarContext all

  CreateShortcut \
    "$SMPROGRAMS\Ozvil\Ozvil (Safe Mode).lnk" \
    "$INSTDIR\ozvil.exe" \
    "--safe-mode" \
    "$INSTDIR\ozvil.exe" \
    0 \
    SW_SHOWNORMAL \
    "" \
    "Launch Ozvil with automation disabled. Use this if a profile or script is causing problems."

  ; Also write a registry value so repair/update runs can verify the shortcut was created.
  WriteRegStr HKLM "Software\Ozvil" "SafeModeShortcutPath" "$SMPROGRAMS\Ozvil\Ozvil (Safe Mode).lnk"
!macroend

; Called by Tauri during uninstall.
!macro customUninstall
  SetShellVarContext all

  ; Remove the Safe Mode shortcut.
  Delete "$SMPROGRAMS\Ozvil\Ozvil (Safe Mode).lnk"

  ; Clean up registry key.
  DeleteRegValue HKLM "Software\Ozvil" "SafeModeShortcutPath"

  ; Remove the Start Menu folder if it is now empty.
  RMDir "$SMPROGRAMS\Ozvil"
!macroend

; ─── Per-machine install (required for Start Menu "all users") ────────────────

!macro customHeader
  !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_KEY "Software\Ozvil"
  !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_VALUENAME "InstallMode"
  !define MULTIUSER_INSTALLMODE_INSTDIR "Ozvil"
  !define MULTIUSER_INSTALLMODE_INSTDIR_REGISTRY_KEY "Software\Ozvil"
  !define MULTIUSER_INSTALLMODE_INSTDIR_REGISTRY_VALUENAME "InstallDir"
!macroend
