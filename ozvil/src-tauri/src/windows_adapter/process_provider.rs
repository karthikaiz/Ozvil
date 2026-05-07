use crate::db::models::ProcessInfo;
use anyhow::Result;

#[cfg(target_os = "windows")]
use {
    once_cell::sync::Lazy,
    parking_lot::Mutex,
    std::collections::HashMap,
};

#[cfg(target_os = "windows")]
struct CpuSnapshot {
    sys_time: u64,
    proc_times: HashMap<u32, u64>,
}

#[cfg(target_os = "windows")]
static PREV_CPU: Lazy<Mutex<Option<CpuSnapshot>>> = Lazy::new(|| Mutex::new(None));

#[cfg(target_os = "windows")]
pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    use std::collections::HashMap;
    use windows::Win32::Foundation::{CloseHandle, FILETIME};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::{
        GetProcessTimes, GetSystemTimes, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };

    fn ft_u64(ft: &FILETIME) -> u64 {
        ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64
    }

    // Enumerate all running processes
    let mut pid_names: Vec<(u32, String)> = vec![];
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                let raw = &entry.szExeFile;
                let len = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
                pid_names.push((entry.th32ProcessID, String::from_utf16_lossy(&raw[..len])));
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }

    // System-wide CPU time (summed across all logical CPUs)
    let sys_time = unsafe {
        let mut idle = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        let _ = GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user));
        ft_u64(&kernel) + ft_u64(&user)
    };

    // Per-process CPU times and RAM (working set)
    let mut proc_cpu: HashMap<u32, u64> = HashMap::new();
    let mut proc_ram: HashMap<u32, u64> = HashMap::new();

    for &(pid, _) in &pid_names {
        unsafe {
            let access = PROCESS_QUERY_INFORMATION | PROCESS_VM_READ;
            if let Ok(handle) = OpenProcess(access, false, pid) {
                let (mut c, mut e, mut k, mut u) = (
                    FILETIME::default(), FILETIME::default(),
                    FILETIME::default(), FILETIME::default(),
                );
                if GetProcessTimes(handle, &mut c, &mut e, &mut k, &mut u).is_ok() {
                    proc_cpu.insert(pid, ft_u64(&k) + ft_u64(&u));
                }

                let mut mem = PROCESS_MEMORY_COUNTERS {
                    cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                    ..Default::default()
                };
                if GetProcessMemoryInfo(handle, &mut mem, mem.cb).is_ok() {
                    proc_ram.insert(pid, mem.WorkingSetSize as u64 / 1024 / 1024);
                }

                let _ = CloseHandle(handle);
            }
        }
    }

    // Compute CPU% from delta vs previous snapshot (delta spans the polling interval ~5s)
    let mut prev_guard = PREV_CPU.lock();
    let cpu_pcts: HashMap<u32, f64> = match prev_guard.as_ref() {
        Some(prev) => {
            let sys_delta = sys_time.saturating_sub(prev.sys_time);
            if sys_delta > 0 {
                proc_cpu.iter().map(|(&pid, &curr)| {
                    let prev_t = prev.proc_times.get(&pid).copied().unwrap_or(0);
                    let proc_delta = curr.saturating_sub(prev_t);
                    let pct = (proc_delta as f64 / sys_delta as f64 * 100.0 * 10.0).round() / 10.0;
                    (pid, pct.min(100.0))
                }).collect()
            } else {
                HashMap::new()
            }
        }
        None => HashMap::new(),
    };
    *prev_guard = Some(CpuSnapshot { sys_time, proc_times: proc_cpu });
    drop(prev_guard);

    Ok(pid_names.into_iter().map(|(pid, name)| ProcessInfo {
        pid,
        name,
        cpu_percent: cpu_pcts.get(&pid).copied().unwrap_or(0.0),
        ram_mb: proc_ram.get(&pid).copied().unwrap_or(0),
    }).collect())
}

#[cfg(not(target_os = "windows"))]
pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    Ok(vec![
        ProcessInfo { pid: 1000, name: "davinci_resolve.exe".to_string(), cpu_percent: 45.0, ram_mb: 2048 },
        ProcessInfo { pid: 1001, name: "chrome.exe".to_string(), cpu_percent: 8.0, ram_mb: 512 },
        ProcessInfo { pid: 1002, name: "OneDrive.exe".to_string(), cpu_percent: 2.0, ram_mb: 128 },
    ])
}
