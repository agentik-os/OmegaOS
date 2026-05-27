//! Lightweight system info — read directly from /proc. No external crates.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_load: f32,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub ram_pct: u8,
    pub disk_used_pct: u8,
}

impl SystemStats {
    pub fn read() -> Self {
        Self {
            cpu_load: read_loadavg(),
            ram_used_mb: 0,
            ram_total_mb: 0,
            ram_pct: read_ram_pct(),
            disk_used_pct: read_disk_pct("/"),
        }
        .fill_ram()
    }

    fn fill_ram(mut self) -> Self {
        if let Some((used, total)) = read_meminfo() {
            self.ram_used_mb = used;
            self.ram_total_mb = total;
        }
        self
    }
}

fn read_loadavg() -> f32 {
    if let Ok(s) = std::fs::read_to_string("/proc/loadavg") {
        s.split_whitespace().next()
            .and_then(|n| n.parse::<f32>().ok())
            .unwrap_or(0.0)
    } else {
        0.0
    }
}

fn read_ram_pct() -> u8 {
    if let Some((used, total)) = read_meminfo() {
        if total > 0 {
            return ((used as f64 / total as f64) * 100.0) as u8;
        }
    }
    0
}

fn read_meminfo() -> Option<(u64, u64)> {
    let s = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kb = 0u64;
    let mut avail_kb = 0u64;
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total_kb = rest.trim().split_whitespace().next()?.parse().ok()?;
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            avail_kb = rest.trim().split_whitespace().next()?.parse().ok()?;
        }
    }
    if total_kb == 0 { return None; }
    let used_kb = total_kb.saturating_sub(avail_kb);
    Some((used_kb / 1024, total_kb / 1024))
}

fn read_disk_pct(path: &str) -> u8 {
    // Best-effort: shell out to `df --output=pcent`. Single fork is OK at TUI refresh rate.
    if let Ok(output) = std::process::Command::new("df")
        .args(["--output=pcent", path])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            for line in s.lines().skip(1) {
                let trimmed = line.trim().trim_end_matches('%');
                if let Ok(n) = trimmed.parse::<u8>() {
                    return n;
                }
            }
        }
    }
    0
}
