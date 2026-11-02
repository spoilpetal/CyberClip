//! CyberClip's command-line entry point and shared clipboard/history state.
//!
//! Keep command parsing here and move platform-specific clipboard work into
//! `clipboard`, file-browser behavior into `files`/`tui`, and text extraction
//! into `get`. That separation makes changes easier to test and review.
mod tui;
mod files;
mod get;
mod clipboard;
mod storage;
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(name = "cyberclip", version, about = "CyberClip clipboard manager")]
struct Cli {
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    Copy {
        #[arg(required = true, num_args = 1..)]
        paths: Vec<PathBuf>,
        #[arg(long, conflicts_with = "folders")]
        files: bool,
        #[arg(long)]
        folders: bool,
    },
    #[command(name = "__clipboard-serve", hide = true)]
    ClipboardServe { payload: PathBuf, mime: String, ready: PathBuf },
    Text { content: String },
    Get {
       #[arg(required = true, num_args = 1..=3)]
       args: Vec<String>,
       #[arg(short = 'n')]
       no_numbers: bool,
    },
    History {
        #[command(subcommand)]
        action: Option<HistoryAction>,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        search: Option<String>,
        #[arg(long = "type")]
        kind: Option<String>,
        #[arg(long)]
        pinned: bool,
    },
    Search { query: String },
    /// Print the current system clipboard text to stdout.
    Paste,
    Remove { index: usize },
    Pin { index: usize },
    Clear,
    Ui,
    Files,
    Storage,
}

#[derive(Subcommand)]
enum HistoryAction {
    /// Restore a numbered history/search result to the system clipboard.
    Copy { index: usize },
}

#[derive(Serialize, Deserialize)]
struct Entry {
    kind: String,
    value: String,
    #[serde(default)]
    pinned: bool,
}

// Keep user data in the platform's normal per-user data directory. The
// installer only places executables; it should never own or relocate history.
fn history_path() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .context("Cannot find local data directory")?
        .join("cyberclip");

    fs::create_dir_all(&dir)?;
    Ok(dir.join("history.json"))
}

fn load_history() -> Result<Vec<Entry>> {
    let path = history_path()?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data).unwrap_or_default())
}

fn save_history(entries: &[Entry]) -> Result<()> {
    let path = history_path()?;
    fs::write(path, serde_json::to_string_pretty(entries)?)?;
    Ok(())
}

/// Persistent file/folder clipboard snapshot shared by CLI and every TUI session.
/// It is separate from the history so ordinary text/image history entries cannot
/// accidentally replace the current file selection.
fn file_clipboard_path() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .context("Cannot find local data directory")?
        .join("cyberclip");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("file-clipboard.json"))
}

fn save_file_selection(paths: &[PathBuf]) -> Result<()> {
    let canonical: Vec<PathBuf> = paths.iter()
        .map(|path| fs::canonicalize(path).with_context(|| format!("Copied item no longer exists: {}", path.display())))
        .collect::<Result<_>>()?;
    let destination = file_clipboard_path()?;
    let temporary = destination.with_file_name(format!(
        "file-clipboard.{}-{}.tmp",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()
    ));
    fs::write(&temporary, serde_json::to_vec_pretty(&canonical)?)?;
    // Unix rename replaces atomically, so readers never observe a half-written
    // manifest. Windows requires removing the old destination first.
    #[cfg(windows)]
    if destination.exists() { fs::remove_file(&destination)?; }
    fs::rename(&temporary, &destination).with_context(|| {
        let _ = fs::remove_file(&temporary);
        format!("Cannot update clipboard selection manifest: {}", destination.display())
    })?;
    Ok(())
}

pub(crate) fn clear_file_selection() -> Result<()> {
    let path = file_clipboard_path()?;
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn clipboard(data: &[u8], mime: Option<&str>) -> Result<()> {
    clipboard::set(data, mime)
}

// Pinned entries are intentionally exempt from the rolling history limit.
// This lets users keep important snippets without allowing ordinary history
// to grow forever.
fn add_history(kind: &str, value: String) -> Result<()> {
    let mut entries = load_history()?;

    entries.insert(
        0,
        Entry {
            kind: kind.to_string(),
            value,
            pinned: false,
        },
    );

    // Keep every pinned item and the 100 newest unpinned items.
    let mut unpinned_count = 0;

    entries.retain(|entry| {
        if entry.pinned {
            true
        } else {
            unpinned_count += 1;
            unpinned_count <= 100
        }
    });

    save_history(&entries)
}
fn mime_type(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        "bmp" => Some("image/bmp"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

fn file_uri(path: &Path) -> Result<String> {
    let path = fs::canonicalize(path)
        .with_context(|| format!("File not found: {}", path.display()))?;
    if !path.is_file() && !path.is_dir() {
        bail!("Unsupported filesystem item: {}", path.display());
    }
    #[cfg(unix)]
    let raw = {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes().to_vec()
    };
    #[cfg(not(unix))]
    let raw = path.to_string_lossy().as_bytes().to_vec();

    let mut encoded = String::from("file://");
    for byte in raw {
        if byte.is_ascii_alphanumeric() || b"/-._~:".contains(&byte) {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{:02X}", byte));
        }
    }
    Ok(encoded)
}

// Resolve paths before putting them on the clipboard. A stable, de-duplicated
// selection is easier for other applications to consume and for the TUI to
// restore later. The --files/--folders switches inspect one directory level.
fn expand_copy_selection(paths: Vec<PathBuf>, files_only: bool, folders_only: bool) -> Result<Vec<PathBuf>> {
    let mut selected = Vec::new();
    for input in paths {
        let resolved = fs::canonicalize(&input)
            .with_context(|| format!("Path not found: {}", input.display()))?;
        if resolved.is_dir() && (files_only || folders_only) {
            for item in fs::read_dir(&resolved)
                .with_context(|| format!("Cannot read directory: {}", resolved.display()))?
            {
                let item = item?;
                let path = item.path();
                let metadata = item.file_type()?;
                if (files_only && metadata.is_file()) || (folders_only && metadata.is_dir()) {
                    selected.push(fs::canonicalize(path)?);
                }
            }
        } else if !files_only && !folders_only || resolved.is_file() && files_only || resolved.is_dir() && folders_only {
            selected.push(resolved);
        }
    }

    selected.sort();
    selected.dedup();
    if selected.is_empty() {
        bail!("No matching files or folders were found");
    }
    Ok(selected)
}

pub(crate) fn copy_files(paths: Vec<PathBuf>, files_only: bool, folders_only: bool) -> Result<()> {
    let canonical = expand_copy_selection(paths, files_only, folders_only)?;

    // Preserve native image clipboard data for a single image file.
    if canonical.len() == 1 && canonical[0].is_file() {
        let path = &canonical[0];
        if let Some(mime) = mime_type(path) {
            // Commit the new TUI/CLI selection before publishing clipboard data,
            // so a subsequent Ctrl+V can never see the previous selection.
            save_file_selection(&canonical)?;
            if let Err(error) = clipboard(&fs::read(path)?, Some(mime)) {
                let _ = clear_file_selection();
                return Err(error);
            }
            add_history("image", path.to_string_lossy().into_owned())?;
            println!("CyberClip: image copied successfully.");
            return Ok(());
        }
    }

    let mut uris = String::new();
    for path in &canonical {
        uris.push_str(&file_uri(path)?);
        uris.push_str("\r\n");
    }
    save_file_selection(&canonical)?;
    if let Err(error) = clipboard(uris.as_bytes(), Some("text/uri-list")) {
        let _ = clear_file_selection();
        return Err(error);
    }

    let history = canonical.iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect::<Vec<_>>().join("\n");
    let kind = if canonical.len() > 1 { "files" }
        else if canonical[0].is_dir() { "folder" } else { "file" };
    add_history(kind, history)?;
    let folders = canonical.iter().filter(|p| p.is_dir()).count();
    let files = canonical.len() - folders;
    println!("CyberClip: copied {files} file(s) and {folders} folder(s) as clipboard references.");
    Ok(())
}

pub(crate) fn latest_file_selection() -> Result<Vec<PathBuf>> {
    // The snapshot is the single source of truth. Never fall back to old
    // history entries: doing so can resurrect a previous file selection after
    // the current clipboard has changed to text or an image.
    let snapshot = file_clipboard_path()?;
    if !snapshot.exists() {
        bail!("The current clipboard does not contain copied files or folders");
    }

    let data = fs::read(&snapshot).context("Cannot read persistent file clipboard")?;
    let paths: Vec<PathBuf> = serde_json::from_slice(&data)
        .context("Persistent file clipboard is damaged; copy the files again")?;
    if paths.is_empty() {
        bail!("The persistent file clipboard is empty");
    }
    Ok(paths)
}

pub(crate) fn restore_entry(kind: &str, value: &str) -> Result<()> {
    match kind {
        "text" => {
            clear_file_selection()?;
            clipboard(value.as_bytes(), None)?;
        }
        "image" => {
            let path = Path::new(value);
            let mime = mime_type(path).context("Unknown image format")?;
            save_file_selection(&[path.to_path_buf()])?;
            if let Err(error) = clipboard(&fs::read(path)?, Some(mime)) {
                let _ = clear_file_selection();
                return Err(error);
            }
        }
        "file" | "folder" => {
            let path = PathBuf::from(value);
            let uri = format!("{}\r\n", file_uri(&path)?);
            save_file_selection(&[path])?;
            if let Err(error) = clipboard(uri.as_bytes(), Some("text/uri-list")) {
                let _ = clear_file_selection();
                return Err(error);
            }
        }
        "files" => {
            let paths: Vec<PathBuf> = value.lines()
                .filter(|path| !path.trim().is_empty())
                .map(PathBuf::from)
                .collect();
            let mut uris = String::new();
            for path in &paths {
                uris.push_str(&file_uri(path)?);
                uris.push_str("\r\n");
            }
            save_file_selection(&paths)?;
            if let Err(error) = clipboard(uris.as_bytes(), Some("text/uri-list")) {
                let _ = clear_file_selection();
                return Err(error);
            }
        }
        _ => bail!("Unknown history item type"),
    }
    Ok(())
}

fn restore(entry: &Entry) -> Result<()> {
    restore_entry(&entry.kind, &entry.value)?;
    println!("CyberClip: clipboard restored.");
    Ok(())
}

fn print_history(entries: &[Entry], limit: Option<usize>, search: Option<&str>, kind: Option<&str>, pinned_only: bool) {
    let query = search.map(str::to_lowercase);
    let mut shown = 0usize;
    for (index, entry) in entries.iter().enumerate() {
        if let Some(kind) = kind {
            if !entry.kind.eq_ignore_ascii_case(kind) { continue; }
        }
        if pinned_only && !entry.pinned { continue; }
        if let Some(query) = &query {
            if !entry.value.to_lowercase().contains(query) { continue; }
        }
        if limit.is_some_and(|max| shown >= max) { break; }
        let preview = entry.value.lines().next().unwrap_or("")
            .chars().take(100).collect::<String>();
        let pin = if entry.pinned { " [PINNED]" } else { "" };
        println!("{:>4}  {:<7} {}{}", index + 1, entry.kind.to_uppercase(), preview, pin);
        shown += 1;
    }
    if shown == 0 { println!("No matching clipboard history items."); }
}

fn get_index(index: usize, entries: &[Entry]) -> Result<usize> {
    if index == 0 || index > entries.len() {
        bail!("Invalid history number. Use cyberclip history to see available items.");
    }

    Ok(index - 1)
}

/// Clap treats an unbounded positional path list as the final argument
/// in some parsing situations. Move our two copy filters before the paths
/// so users can write either `copy --files DIR` or `copy DIR --files`.
// Clap's variadic `copy` path argument can otherwise swallow trailing flags.
// Move only our known copy filters ahead of paths, preserving all other args.
fn normalized_args() -> Vec<OsString> {
    let args: Vec<OsString> = env::args_os().collect();
    if args.len() < 2 || args[1].to_str() != Some("copy") {
        return args;
    }

    let mut normalized = vec![args[0].clone(), args[1].clone()];
    let mut options = Vec::new();
    let mut paths = Vec::new();
    let mut after_separator = false;

    for arg in args.into_iter().skip(2) {
        let value = arg.to_str();
        if !after_separator && value == Some("--") {
            after_separator = true;
            paths.push(arg);
        } else if !after_separator && (value == Some("--files") || value == Some("--folders")) {
            options.push(arg);
        } else {
            paths.push(arg);
        }
    }

    normalized.extend(options);
    normalized.extend(paths);
    normalized
}

// `version` in the Clap attribute above comes directly from Cargo.toml, so
// `cpl --version` stays in sync with the package version automatically.
fn main() -> Result<()> {
    let cli = Cli::parse_from(normalized_args());

    match cli.command {
        Action::Copy { paths, files, folders } => copy_files(paths, files, folders)?,
        Action::ClipboardServe { payload, mime, ready } => {
            #[cfg(target_os = "linux")]
            clipboard::serve(&payload, &mime, &ready)?;
            #[cfg(not(target_os = "linux"))]
            bail!("Clipboard provider is not implemented on this platform yet");
        }
        Action::Files => files::run()?,
        Action::Storage => print!("{}", storage::report()),
        Action::Ui => tui::run()?,
        Action::Get { args, no_numbers } => {
            let show_numbers = !no_numbers;
            let result = match args.as_slice() {
                [file] => get::whole_file(Path::new(file))?,
                [query, file] => get::search(Path::new(file), query, show_numbers)?,
                [mode, range, file] if mode == "line" => {
                    let (start, end) = if let Some((start, end)) = range.split_once(':') {
                        (start.parse::<usize>()?, end.parse::<usize>()?)
                    } else {
                        let number = range.parse::<usize>()?;
                        (number, number)
                    };
                    get::extract(Path::new(file), start, end, show_numbers)?
                }
                _ => bail!("Usage: cyberclip get FILE | get QUERY FILE | get line RANGE FILE"),
            };

            if result.matches == 0 || result.text.is_empty() {
                println!("No content found. Clipboard unchanged.");
            } else {
                clear_file_selection()?;
                clipboard(result.text.as_bytes(), None)?;
                add_history("text", result.text.clone())?;
                print!("{}", result.text);
                println!("\n{} line(s) copied to clipboard.", result.matches);
            }
        }
        Action::Text { content } => {
            clear_file_selection()?;
            clipboard(content.as_bytes(), None)?;
            add_history("text", content)?;
            println!("CyberClip: text copied.");
        }

        Action::History { action, limit, search, kind, pinned } => {
            let entries = load_history()?;
            match action {
                Some(HistoryAction::Copy { index }) => {
                    let position = get_index(index, &entries)?;
                    restore(&entries[position])?;
                }
                None => {
                    if entries.is_empty() {
                        println!("Clipboard history is empty.");
                    } else {
                        print_history(&entries, limit, search.as_deref(), kind.as_deref(), pinned);
                        println!("Use `cyberclip history copy NUMBER` to copy a listed item back to the system clipboard.");
                    }
                }
            }
        }

        Action::Search { query } => {
            let entries = load_history()?;
            let before = entries.len();
            print_history(&entries, None, Some(&query), None, false);
            if before > 0 {
                println!("Use `cyberclip history copy NUMBER` to copy a result. Numbers refer to the original history list.");
            }
        }

        Action::Paste => {
            let text = clipboard::get_text()?;
            use std::io::Write;
            std::io::stdout().write_all(text.as_bytes())?;
        }

        Action::Remove { index } => {
            let mut entries = load_history()?;
            let position = get_index(index, &entries)?;

            entries.remove(position);
            save_history(&entries)?;

            println!("CyberClip: item {} removed.", index);
        }

        Action::Pin { index } => {
            let mut entries = load_history()?;
            let position = get_index(index, &entries)?;

            entries[position].pinned = !entries[position].pinned;
            let pinned = entries[position].pinned;

            save_history(&entries)?;

            if pinned {
                println!("CyberClip: item {} pinned.", index);
            } else {
                println!("CyberClip: item {} unpinned.", index);
            }
        }

        Action::Clear => {
            clear_file_selection()?;
            clipboard(b"", None)?;
            save_history(&[])?;
            println!("CyberClip: clipboard history cleared.");
        }
    }

    Ok(())
}
