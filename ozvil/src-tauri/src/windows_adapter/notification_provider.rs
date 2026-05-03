use crate::db::models::ActionResult;
use anyhow::Result;

/// Best-effort interruption reduction.
/// Windows does not expose a reliable third-party API for Focus Assist.
/// This attempts to set Focus Assist via powershell registry path,
/// but must not silently pretend to succeed if unsupported.
#[cfg(target_os = "windows")]
pub fn reduce_interruptions() -> Result<ActionResult> {
    // Focus Assist registry path: HKCU\Software\Microsoft\Windows\CurrentVersion\CloudStore\Store\...
    // This is undocumented and brittle. Per spec, do NOT use undocumented registry hacks.
    // Return a documented "best-effort unsupported" result instead.
    Ok(ActionResult::UnsupportedCapability {
        reason: "Windows does not expose a reliable third-party Focus Assist API. \
                 Use the manual interruption checklist as a fallback.".to_string(),
    })
}

#[cfg(not(target_os = "windows"))]
pub fn reduce_interruptions() -> Result<ActionResult> {
    Ok(ActionResult::UnsupportedCapability {
        reason: "Interruption reduction is only relevant on Windows.".to_string(),
    })
}

/// Returns a checklist for manual interruption reduction the user can follow
/// when automated focus control is unavailable.
pub fn manual_interruption_checklist() -> Vec<&'static str> {
    vec![
        "Turn on Windows Focus Assist manually (Start → Settings → System → Focus Assist)",
        "Silence phone notifications or enable Do Not Disturb",
        "Close or mute email and chat apps (Teams, Slack, Outlook)",
        "Disable browser notification pop-ups",
        "Turn off desktop notification badges in taskbar settings",
        "If on a call or stream, mute non-essential hardware inputs",
    ]
}
