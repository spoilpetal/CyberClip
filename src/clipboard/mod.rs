//! Clipboard abstraction. Platform backends live behind this module.
//!
//! Keep platform checks here rather than spreading `cfg` branches through the
//! CLI. A new OS should get a real backend before it is added to this list.

use anyhow::Result;

#[cfg(target_os = "linux")]
mod linux_wayland;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
mod native;

/// Set clipboard data through the current platform's native clipboard backend.
pub fn set(data: &[u8], mime: Option<&str>) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        // Keep the existing persistent provider on Wayland. On X11, use the
        // cross-platform native clipboard implementation.
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            return linux_wayland::set(data, mime);
        }
        return native::set(data, mime);
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        return native::set(data, mime);
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = (data, mime);
        anyhow::bail!("CyberClip's native clipboard backend does not support this operating system yet")
    }
}

/// Entry point used by the short-lived CLI to serve data from a persistent provider process.
#[cfg(target_os = "linux")]
pub fn serve(payload: &std::path::Path, mime: &str, ready: &std::path::Path) -> Result<()> {
    linux_wayland::serve(payload, mime, ready)
}

/// Read plain text from the current system clipboard.
///
/// This is intentionally separate from clipboard history: `cyberclip paste`
/// reads whatever text is currently available from another application too.
pub fn get_text() -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            return linux_wayland::get_text();
        }
        return native::get_text();
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        return native::get_text();
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        anyhow::bail!("CyberClip's native clipboard backend does not support this operating system yet")
    }
}
