use crate::windows_adapter::WindowsAdapter;
use std::time::Duration;

/// Adaptive monitoring loop. Polls faster when active sessions or watched
/// processes are detected; backs off during idle to stay low-overhead.
///
/// Active interval = half of base_interval, clamped to [1 000 ms, 5 000 ms].
/// Idle interval   = base_interval, minimum 10 000 ms.
pub struct MonitoringLoop {
    idle_interval_ms: u64,
    active_interval_ms: u64,
}

impl MonitoringLoop {
    pub fn new(base_interval_ms: u64) -> Self {
        MonitoringLoop {
            idle_interval_ms: base_interval_ms.max(10_000),
            active_interval_ms: (base_interval_ms / 2).max(1_000).min(5_000),
        }
    }

    pub fn interval_for(&self, has_active_session: bool, watched_procs_running: bool) -> Duration {
        if has_active_session || watched_procs_running {
            Duration::from_millis(self.active_interval_ms)
        } else {
            Duration::from_millis(self.idle_interval_ms)
        }
    }
}
