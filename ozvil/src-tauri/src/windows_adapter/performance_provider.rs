use anyhow::Result;

pub struct PerformanceReading {
    pub cpu_percent: f64,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
}

#[cfg(target_os = "windows")]
pub fn read_performance() -> Result<PerformanceReading> {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let mut mem = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };

    unsafe {
        GlobalMemoryStatusEx(&mut mem)?;
    }

    let total = mem.ullTotalPhys / 1024 / 1024;
    let available = mem.ullAvailPhys / 1024 / 1024;
    let used = total.saturating_sub(available);

    // CPU: use a simple approach via PDH query
    // In production, this would use a cached PDH query with proper timing.
    // Using 0.0 as safe default to avoid PDH complexity in the scaffold.
    let cpu_percent = read_cpu_percent_pdh().unwrap_or(0.0);

    Ok(PerformanceReading {
        cpu_percent,
        ram_used_mb: used,
        ram_total_mb: total,
    })
}

#[cfg(target_os = "windows")]
fn read_cpu_percent_pdh() -> Result<f64> {
    // Simplified: real implementation would maintain a persistent PDH query handle.
    Ok(0.0)
}

#[cfg(not(target_os = "windows"))]
pub fn read_performance() -> Result<PerformanceReading> {
    Ok(PerformanceReading {
        cpu_percent: 42.0,
        ram_used_mb: 12288,
        ram_total_mb: 32768,
    })
}
