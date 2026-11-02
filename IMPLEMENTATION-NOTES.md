# Experimental clipboard backend notes

This source snapshot is an experimental first milestone based on `CyberClip-history-icons.zip`.

## Included

- Replaces `wl-copy` process execution with the `wl-clipboard-rs` Rust library on Linux Wayland.
- Starts a small, user-level clipboard provider process for each clipboard update. That provider remains alive while it owns the Wayland selection, then exits when another application replaces/clears the selection or the session ends.
- Uses the user's existing file permissions; no administrator/root service is installed.
- Supports one directory reference, multiple directories/files together, and immediate-child filters via `copy PATH --files` and `copy PATH --folders`.
- Keeps TUI history restoration on the same shared clipboard function.
- Documents `cpl` as the short command alias; `cp` and `cpp` are left untouched.\n- Normalizes `--files` and `--folders` so they work both before and after positional paths.

## Not included yet

- X11 clipboard backend.
- Windows and macOS native backends.
- Automatic login service. The provider is started on demand for a clipboard selection, which avoids an always-running service when it is not needed.
- Recursive enumeration for `--files` / `--folders`; those options select immediate children. Copying a folder itself still places the folder reference on the clipboard, so the receiving file manager handles its nested contents.
- Shell-independent wildcard expansion. On Linux, the shell expands patterns such as `*.pdf` before CyberClip receives them.

## Verification status

This environment does not have `cargo`, `rustc`, or `rustfmt`, so this snapshot has **not** been compiled or tested here. Run `cargo check` and `cargo build --release` on the target Arch/Wayland machine before replacing the working installation. Treat this as an experimental source update until those checks and clipboard paste tests pass.

## File browser copy/paste behavior
The file browser supports marking entries with Space, copying marked entries (or the current entry when no marks exist) with C, and pasting that in-session selection into the current directory with Ctrl+V. The copy action also writes file URI references through the shared native clipboard backend and records history. Folder pasting is recursive. Existing destination names are treated as an error; no overwrite prompt or merge behavior is implemented. Symbolic links are currently rejected by the in-browser recursive copy routine.

The file browser's Ctrl+V reads the persistent file-selection manifest shared by CLI and TUI. The manifest is authoritative: when a text or image selection clears it, old file history is not reused as a fallback. A single image copied by CyberClip preserves native image data for external applications and its source path for TUI file paste. Reading arbitrary clipboard file selections created by other applications is a separate native-backend feature and is not yet implemented.

## CLI paste behavior

`cyberclip paste` reads the live system clipboard through the platform backend and writes text bytes directly to stdout. It does not select from history, simulate keystrokes, or change TUI shortcuts. Wayland uses `wl-clipboard-rs` directly; X11, Windows, and macOS use `clipboard-rs`.
