//! System stats (CPU / RAM / disk / network / uptime) for the Stats panel.
//! Independent of Tauri so it can be unit-checked without the webkit stack.

use std::time::Duration;

use serde::Serialize;
use sysinfo::{Disks, Networks, System};

#[derive(Clone, Debug, Serialize)]
pub struct SystemStats {
    /// Aggregate CPU usage, 0..100.
    pub cpu_usage: f32,
    /// Memory usage, 0..100.
    pub memory_usage: f32,
    /// Used memory, bytes.
    pub memory_used: u64,
    /// Total memory, bytes.
    pub memory_total: u64,
    /// Primary disk usage, 0..100.
    pub disk_usage: f32,
    /// Disk used, bytes.
    pub disk_used: u64,
    /// Disk total, bytes.
    pub disk_total: u64,
    /// Total bytes received across all interfaces.
    pub network_rx: u64,
    /// Total bytes transmitted across all interfaces.
    pub network_tx: u64,
    /// Host uptime in seconds.
    pub uptime: u64,
}

/// Sample the system. Takes a short blocking pause to get a meaningful CPU
/// reading (CPU usage is computed between two samples).
pub fn sample() -> SystemStats {
    let mut sys = System::new();
    // First CPU sample.
    sys.refresh_cpu_usage();
    std::thread::sleep(Duration::from_millis(250));
    sys.refresh_cpu_usage();
    let cpu_usage = if sys.cpus().is_empty() {
        0.0
    } else {
        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32
    };

    sys.refresh_memory();
    let memory_total = sys.total_memory();
    let memory_used = sys.used_memory();
    let memory_usage = if memory_total > 0 {
        (memory_used as f32 / memory_total as f32) * 100.0
    } else {
        0.0
    };

    // Pick the largest mounted disk as the "primary" one for the bar.
    let disks = Disks::new_with_refreshed_list();
    let primary = disks
        .iter()
        .max_by_key(|d| d.total_space());
    let (disk_total, disk_used) = match primary {
        Some(d) => {
            let total = d.total_space();
            let used = total.saturating_sub(d.available_space());
            (total, used)
        }
        None => (0, 0),
    };
    let disk_usage = if disk_total > 0 {
        (disk_used as f32 / disk_total as f32) * 100.0
    } else {
        0.0
    };

    let networks = Networks::new_with_refreshed_list();
    let (network_rx, network_tx) = networks
        .iter()
        .map(|(_, n)| (n.total_received(), n.total_transmitted()))
        .fold((0u64, 0u64), |(rx, tx), (r, t)| (rx + r, tx + t));

    let uptime = System::uptime();

    SystemStats {
        cpu_usage,
        memory_usage,
        memory_used,
        memory_total,
        disk_usage,
        disk_used,
        disk_total,
        network_rx,
        network_tx,
        uptime,
    }
}