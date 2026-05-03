use crate::db::models::ActionResult;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};

static SLEEP_PREVENTED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
pub fn prevent_sleep() -> Result<ActionResult> {
    use windows::Win32::System::Power::{SetThreadExecutionState, ES_CONTINUOUS, ES_SYSTEM_REQUIRED};

    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED);
    }

    SLEEP_PREVENTED.store(true, Ordering::SeqCst);
    Ok(ActionResult::Ok)
}

#[cfg(target_os = "windows")]
pub fn release_sleep_prevention() -> Result<()> {
    use windows::Win32::System::Power::{SetThreadExecutionState, ES_CONTINUOUS};
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS);
    }
    SLEEP_PREVENTED.store(false, Ordering::SeqCst);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn prevent_sleep() -> Result<ActionResult> {
    SLEEP_PREVENTED.store(true, Ordering::SeqCst);
    Ok(ActionResult::Ok)
}

#[cfg(not(target_os = "windows"))]
pub fn release_sleep_prevention() -> Result<()> {
    SLEEP_PREVENTED.store(false, Ordering::SeqCst);
    Ok(())
}

pub fn is_sleep_prevented() -> bool {
    SLEEP_PREVENTED.load(Ordering::SeqCst)
}
