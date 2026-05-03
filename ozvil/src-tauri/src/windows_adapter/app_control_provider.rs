use crate::db::models::ActionResult;
use anyhow::Result;

/// Suspend (pause) a user-approved process by name.
/// Uses NtSuspendProcess on Windows. Non-Windows builds return an unsupported result.
#[cfg(target_os = "windows")]
pub fn pause_app(app_id: &str) -> Result<ActionResult> {
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_SUSPEND_RESUME, SuspendThread,
    };
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32, TH32CS_SNAPTHREAD,
    };
    use windows::Win32::Foundation::CloseHandle;

    // Find the process PID by name first
    let processes = super::process_provider::list_processes()?;
    let target = processes
        .iter()
        .find(|p| p.name.to_lowercase() == app_id.to_lowercase());

    let pid = match target {
        Some(p) => p.pid,
        None => {
            return Ok(ActionResult::Failed {
                reason: format!("Process not found: {}", app_id),
            })
        }
    };

    // Suspend all threads of the process
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0)?;
        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };

        if Thread32First(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32OwnerProcessID == pid {
                    if let Ok(thread_handle) = OpenProcess(PROCESS_SUSPEND_RESUME, false, entry.th32ThreadID) {
                        let _ = SuspendThread(thread_handle);
                        let _ = CloseHandle(thread_handle);
                    }
                }
                if Thread32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }

    Ok(ActionResult::Ok)
}

#[cfg(target_os = "windows")]
pub fn resume_app(app_id: &str) -> Result<ActionResult> {
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SUSPEND_RESUME, ResumeThread};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32, TH32CS_SNAPTHREAD,
    };
    use windows::Win32::Foundation::CloseHandle;

    let processes = super::process_provider::list_processes()?;
    let target = processes
        .iter()
        .find(|p| p.name.to_lowercase() == app_id.to_lowercase());

    let pid = match target {
        Some(p) => p.pid,
        None => return Ok(ActionResult::Ok), // already gone, fine
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0)?;
        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };

        if Thread32First(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32OwnerProcessID == pid {
                    if let Ok(thread_handle) = OpenProcess(PROCESS_SUSPEND_RESUME, false, entry.th32ThreadID) {
                        let _ = ResumeThread(thread_handle);
                        let _ = CloseHandle(thread_handle);
                    }
                }
                if Thread32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }

    Ok(ActionResult::Ok)
}

#[cfg(not(target_os = "windows"))]
pub fn pause_app(app_id: &str) -> Result<ActionResult> {
    Ok(ActionResult::UnsupportedCapability {
        reason: format!("App control is only supported on Windows. app_id={}", app_id),
    })
}

#[cfg(not(target_os = "windows"))]
pub fn resume_app(_app_id: &str) -> Result<ActionResult> {
    Ok(ActionResult::UnsupportedCapability {
        reason: "App resume is only supported on Windows".to_string(),
    })
}
