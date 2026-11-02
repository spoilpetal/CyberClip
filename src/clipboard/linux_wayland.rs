//! Wayland clipboard provider. The provider process remains alive while it owns the selection.
//!
//! The CLI stages bytes in a private runtime file and starts a detached copy of
//! itself. That helper owns the Wayland selection after the short-lived command
//! returns, which is why this code must not simply exit after setting the data.

use anyhow::{Context, Result, bail};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use wl_clipboard_rs::copy::{MimeType, Options, Source};

fn runtime_dir() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .context("XDG_RUNTIME_DIR is not set; start CyberClip inside your Wayland user session")?;
    Ok(PathBuf::from(base).join("cyberclip"))
}

/// Start a provider process. It owns the selection after this CLI invocation exits.
pub fn set(data: &[u8], mime: Option<&str>) -> Result<()> {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        bail!("No Wayland session detected. The X11 clipboard backend is not implemented yet.");
    }

    let dir = runtime_dir()?;
    fs::create_dir_all(&dir).context("Could not create CyberClip runtime directory")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
    }
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let base = format!("{}-{stamp}", std::process::id());
    let payload = dir.join(format!("{base}.data"));
    let ready = dir.join(format!("{base}.ready"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        use std::fs::OpenOptions;
        use std::io::Write;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&payload)
            .context("Could not create private clipboard staging file")?;
        file.write_all(data).context("Could not stage clipboard data")?;
    }
    #[cfg(not(unix))]
    fs::write(&payload, data).context("Could not stage clipboard data")?;

    let mime_arg = mime.unwrap_or("__CYBERCLIP_TEXT__");
    let exe = std::env::current_exe().context("Could not locate CyberClip executable")?;
    let mut child = Command::new(exe)
        .arg("__clipboard-serve")
        .arg(&payload)
        .arg(mime_arg)
        .arg(&ready)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Could not start CyberClip clipboard provider")?;

    // Wait until the provider has successfully prepared its native clipboard offer.
    // Don't report success until the helper has prepared its offer. A bounded
    // wait avoids leaving the user with a command that appears to hang forever.
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if ready.exists() {
            let _ = fs::remove_file(&ready);
            return Ok(());
        }
        if let Some(status) = child.try_wait()? {
            let _ = fs::remove_file(&payload);
            bail!("Native Wayland clipboard provider exited early ({status}). Check compositor clipboard protocol support.");
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_file(&payload);
            bail!("Timed out while starting the native clipboard provider");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

/// Runs in a detached process and serves clipboard requests until another app replaces us.
pub fn serve(payload: &Path, mime: &str, ready: &Path) -> Result<()> {
    let data = fs::read(payload).context("Could not read staged clipboard data")?;
    let _ = fs::remove_file(payload);

    let mut options = Options::new();
    options.foreground(true);
    let prepared = if mime == "__CYBERCLIP_TEXT__" {
        options.prepare_copy(Source::Bytes(data.into()), MimeType::Text)
    } else {
        options.prepare_copy(
            Source::Bytes(data.into()),
            MimeType::Specific(mime.to_string()),
        )
    }
    .context("Could not prepare native Wayland clipboard offer")?;

    fs::write(ready, b"ready").context("Could not notify CyberClip that clipboard is ready")?;
    // foreground(true) keeps this process alive until the selection is replaced or cleared.
    prepared.serve().context("Could not serve native Wayland clipboard data")?;
    let _ = fs::remove_file(ready);
    Ok(())
}

/// Read plain text from the current Wayland regular clipboard selection.
pub fn get_text() -> Result<String> {
    use std::io::Read;
    use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};

    let (mut pipe, _mime_type) = get_contents(
        ClipboardType::Regular,
        Seat::Unspecified,
        MimeType::Text,
    )
    .map_err(|e| anyhow::anyhow!("The Wayland clipboard is empty or does not contain text: {e}"))?;

    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes)
        .context("Could not read text from the Wayland clipboard")?;
    String::from_utf8(bytes).context("The clipboard text is not valid UTF-8")
}
