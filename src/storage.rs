//! Cross-platform disk capacity reporting shared by the CLI and TUIs.
//!
//! Keep the calculations here so the terminal report and TUI preview show the
//! same numbers. `available_space` is used for free space, which may differ
//! from total space minus used space on filesystems with reserved blocks.
use sysinfo::Disks;

#[derive(Clone, Debug)]
pub struct DiskUsage {
    pub name: String,
    pub mount: String,
    pub filesystem: String,
    pub total: u64,
    pub available: u64,
}

impl DiskUsage {
    pub fn used(&self) -> u64 { self.total.saturating_sub(self.available) }
    pub fn percent_used(&self) -> u64 {
        if self.total == 0 { 0 } else { ((self.used() as f64 / self.total as f64) * 100.0).round() as u64 }
    }
}

pub fn disks() -> Vec<DiskUsage> {
    let disks = Disks::new_with_refreshed_list();
    let mut result: Vec<DiskUsage> = disks.list().iter().map(|disk| DiskUsage {
        name: {
            let name = disk.name().to_string_lossy().trim().to_string();
            if name.is_empty() { disk.mount_point().display().to_string() } else { name }
        },
        mount: disk.mount_point().display().to_string(),
        filesystem: disk.file_system().to_string_lossy().to_string(),
        total: disk.total_space(),
        available: disk.available_space(),
    }).filter(|disk| disk.total > 0).collect();
    result.sort_by(|a, b| a.mount.cmp(&b.mount));
    result
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 { value /= 1024.0; unit += 1; }
    if unit == 0 { format!("{} {}", bytes, UNITS[unit]) } else { format!("{value:.2} {}", UNITS[unit]) }
}

pub fn progress_bar(percent: u64, width: usize) -> String {
    let filled = ((percent.min(100) as usize * width) + 50) / 100;
    format!("{}{}", "█".repeat(filled), "░".repeat(width.saturating_sub(filled)))
}

pub fn report() -> String {
    let disks = disks();
    if disks.is_empty() {
        return "No mounted storage devices were reported by the operating system.".into();
    }
    let mut lines = vec!["CYBERCLIP // SYSTEM STORAGE".to_string(), String::new()];
    for (index, disk) in disks.iter().enumerate() {
        lines.push(format!("{}. {} ({})", index + 1, disk.name, disk.filesystem));
        lines.push(format!("   Mount: {}", disk.mount));
        lines.push(format!("   [{}] {}% used", progress_bar(disk.percent_used(), 28), disk.percent_used()));
        lines.push(format!("   Used: {} / {}", format_bytes(disk.used()), format_bytes(disk.total)));
        lines.push(format!("   Free: {}", format_bytes(disk.available)));
        lines.push(String::new());
    }
    lines.join("\n")
}
