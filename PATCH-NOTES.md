# Patch notes — command parsing and alias

- `--files` and `--folders` are moved before positional copy paths before Clap parses the command. Both `cpl copy DIR --files` and `cpl copy --files DIR` are intended to work.
- The README now recommends `cpl` as the short symlink alias.
- `cp` and `cpp` are not changed or replaced.
- Existing directory selection, mixed file/folder URI-list creation, image copying, history, and the Wayland provider are otherwise unchanged.

Build and test this extracted source in a new directory. This patch has not been compiled in this environment.

## File-browser clipboard update
- Added Space-based multi-selection in the file browser.
- Added C to copy the highlighted item or all marked items through CyberClip's native clipboard and history path.
- Added Ctrl+V / Ctrl+Shift+V to recursively paste the current file-browser copy buffer into the open directory.
- Paste refuses to overwrite existing destinations and attempts rollback if a copy fails.
- Added folder and mixed-selection previews in clipboard history instead of showing “Unknown clipboard item”.
- File selections are now stored in a persistent per-user manifest shared by CLI and TUI; importing arbitrary file selections copied by other applications remains future work.

## TUI delete and key-handler cleanup
- Removed the unreachable `C/c` key pattern by placing the content-search copy handler before the general file-copy handler.
- Added `Del` in the file browser to request deletion of the highlighted item.
- Deletion requires `Y` confirmation; `N` or `Esc` cancels. Directories are removed recursively, regular files are removed, and symlinks are unlinked without following their targets.
- The directory listing and preview refresh after a successful deletion.


## Cross-platform clipboard engine milestone

- Added `clipboard-rs` as the native backend for X11, Windows and macOS.
- Preserved the existing persistent Wayland provider to avoid regressing the tested Arch/Hyprland flow.
- Routed plain text, image/custom MIME payloads and file URI lists through the shared clipboard module.
- On X11, file URI lists are offered as `text/uri-list`; Windows/macOS use the library's native file-list API.
- This update is source-integrated but not compiled in this environment. Run the Linux build and verify paste in a file manager before replacing the current working build. Cross-compile/test Windows and macOS on those platforms.

## Persistent file clipboard + per-user installation

- Added a durable file-selection manifest shared by CLI and TUI, so Ctrl+V can use a copied file/folder selection after the TUI is closed and reopened.
- CLI `copy`, TUI `C`, and restoring file/folder history entries update the same manifest.
- CyberClip text/image copies and `clear` remove the file-selection manifest to avoid pasting stale file selections.
- The manifest is now the only source of truth for TUI file paste. It no longer falls back to old history entries, which could resurrect a stale selection after copying text or an image.
- A single-image copy keeps both native image clipboard data (for external applications) and its source path in the manifest (for TUI file paste).
- Removed the obsolete session-local `copy_buffer`, eliminating its unused-assignment compiler warning.
- The installer builds first, detects existing `cyberclip` and `cpl` commands, asks before overwriting selected names, and offers cyberclip-only, cpl-only, or both. It remains per-user and does not require sudo.


## Clipboard selection consistency and installer collision handling

- Fixed stale file-selection fallback: clearing the manifest now truly means there is no file selection to paste; older file entries in history are not silently reused.
- Single-image copies retain their original path in the file manifest while the native image payload remains available to external applications.
- Removed the unused TUI `copy_buffer` assignment.
- Reworked installation to detect existing command names, request explicit overwrite consent, and let users choose `cyberclip`, `cpl`, or both.
- Updated uninstall to remove only command names recorded by this installer.

## Clipboard selection consistency and installer collision handling

- Fixed stale file-selection fallback: clearing the manifest now truly means there is no file selection to paste; older file entries in history are not silently reused.
- Single-image copies retain their original path in the file manifest while the native image payload remains available to external applications.
- Removed the unused TUI `copy_buffer` assignment.
- Reworked installation to detect existing command names, request explicit overwrite consent, and let users choose `cyberclip`, `cpl`, or both.
- Updated uninstall to remove only command names recorded by this installer.


## Storage and preview rendering

- Added `storage` CLI subcommand with per-mounted-volume total, used, available and percentage-used values plus a progress bar.
- Added the same storage report to both TUIs using the `S` shortcut; press `S` again to return to the selected preview.
- Clear each preview panel before rendering its current content to prevent stale text/image cells from remaining when a new preview is shorter or changes type.
- Added the cross-platform `sysinfo` disk enumeration dependency.

## Platform-aware installation and uninstall recovery

- Installer detects Linux Wayland/X11, macOS, Windows shell environments and unsupported BSD/other targets, and explains that Cargo compiles the native Rust backend for the host.
- Installer supports Windows `.exe` build output when run from a compatible shell.
- Existing commands outside the per-user install directory are never modified; the per-user command shadows them when its PATH entry has priority.
- Existing command files inside the install directory are backed up and restored by uninstall. Uninstall removes managed command names first, then restores backups.

## CLI history export and release preparation

- Improved `history` output with `--limit`, `--search`, `--type`, and `--pinned` filters.
- Added `history copy INDEX`, which restores the exact numbered history entry to the native system clipboard.
- Search output keeps original history indexes; use that number with `history copy`.
- TUI history supports `C` to copy/restore the selected entry to the system clipboard.
- Corrected `paste`: it now reads the current native system clipboard and writes plain text to stdout without adding a newline. It works with text copied by other applications and supports shell pipes/redirection. Non-text clipboard contents produce a clear error rather than binary terminal output. History restoration remains `history copy INDEX`; TUI Ctrl+V behavior is unchanged.
- Added the committed Cargo.lock and locked source-installer/CI builds.
- Added a GitHub tag-triggered native release workflow for Linux x86_64/ARM64, Windows x86_64, macOS Intel, and Apple Silicon. Each asset receives a SHA-256 checksum.
- This environment has no Rust/Cargo toolchain, so Rust compilation and native platform binaries were not produced here. GitHub Actions must perform those native builds after the source archive is pushed and a version tag is pushed.
