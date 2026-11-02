//! Cross-platform native clipboard backend for X11, Windows and macOS.
//!
//! Wayland remains on the dedicated wl-clipboard-rs provider because it must
//! keep serving the selection after the short-lived CLI exits.
//!
//! When changing MIME handling, test both text and file references in another
//! desktop application; successful API calls alone do not prove interoperability.

use anyhow::{Context, Result, bail};
use clipboard_rs::{Clipboard, ClipboardContext};

pub fn set(data: &[u8], mime: Option<&str>) -> Result<()> {
    let clipboard = ClipboardContext::new()
        .map_err(|e| anyhow::anyhow!("Could not open the native system clipboard: {e}"))?;

    match mime.unwrap_or("text/plain") {
        "text/uri-list" => {
            let value = std::str::from_utf8(data)
                .context("File clipboard data is not valid UTF-8")?;
            let files: Vec<String> = value
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_owned)
                .collect();
            if files.is_empty() {
                bail!("No file paths were supplied to the native clipboard backend");
            }
            #[cfg(target_os = "linux")]
            clipboard
                .set_buffer("text/uri-list", data.to_vec())
                .map_err(|e| anyhow::anyhow!("Could not set X11 file URI clipboard data: {e}"))?;
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            clipboard
                .set_files(files)
                .map_err(|e| anyhow::anyhow!("Could not set native file clipboard data: {e}"))?;
        }
        "text/plain" | "text/plain;charset=utf-8" => {
            let text = String::from_utf8(data.to_vec())
                .context("Clipboard text is not valid UTF-8")?;
            clipboard
                .set_text(text)
                .map_err(|e| anyhow::anyhow!("Could not set native text clipboard data: {e}"))?;
        }
        format => {
            clipboard
                .set_buffer(format, data.to_vec())
                .map_err(|e| anyhow::anyhow!("Could not set native clipboard format {format}: {e}"))?;
        }
    }

    Ok(())
}

/// Read plain text from the native X11, Windows, or macOS clipboard.
pub fn get_text() -> Result<String> {
    let clipboard = ClipboardContext::new()
        .map_err(|e| anyhow::anyhow!("Could not open the native system clipboard: {e}"))?;
    clipboard
        .get_text()
        .map_err(|e| anyhow::anyhow!("The system clipboard does not contain readable text: {e}"))
}
