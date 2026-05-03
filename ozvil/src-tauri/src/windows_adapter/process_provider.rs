use crate::db::models::ProcessInfo;
use anyhow::Result;

#[cfg(target_os = "windows")]
pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::Foundation::CloseHandle;

    let mut processes = vec![];

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name_raw = &entry.szExeFile;
                let name_len = name_raw.iter().position(|&c| c == 0).unwrap_or(name_raw.len());
                let name = String::from_utf16_lossy(&name_raw[..name_len]);

                processes.push(ProcessInfo {
                    pid: entry.th32ProcessID,
                    name,
                    cpu_percent: 0.0,
                    ram_mb: 0,
                });

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
    }

    Ok(processes)
}

#[cfg(not(target_os = "windows"))]
pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    // Stub for non-Windows builds (dev/test only)
    Ok(vec![
        ProcessInfo { pid: 1000, name: "davinci_resolve.exe".to_string(), cpu_percent: 45.0, ram_mb: 2048 },
        ProcessInfo { pid: 1001, name: "chrome.exe".to_string(), cpu_percent: 8.0, ram_mb: 512 },
        ProcessInfo { pid: 1002, name: "OneDrive.exe".to_string(), cpu_percent: 2.0, ram_mb: 128 },
    ])
}
