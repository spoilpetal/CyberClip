# CyberClip

**A terminal-first clipboard manager for Linux, Windows, and macOS.**

CyberClip combines a command-line interface (CLI) with a terminal user interface (TUI). It keeps a local clipboard history, lets you restore saved entries, and works with text, images, files, folders, and mixed file/folder selections.

> **Platform transparency:** CyberClip is developed and runtime-tested on Linux (Wayland). Linux X11, Windows, and macOS clipboard backends are implemented in the source code. GitHub Actions is configured to build and test the project on supported Linux, Windows, and macOS runners. However, Windows and macOS clipboard behavior has not yet been verified through hands-on testing on physical devices. Linux X11 also needs independent runtime verification. These platforms should therefore be considered implemented but not fully runtime-verified. BSD is not currently supported.**awaiting independent CI and real-device verification**, not as personally tested. BSD is not supported at this time.

## Contents

- [Features](#features)
- [Screenshots](#screenshots)
- [Platform support and testing status](#platform-support-and-testing-status)
- [Requirements](#requirements)
- [Install from source](#install-from-source)
- [Build manually](#build-manually)
- [Choose your command name](#choose-your-command-name)
- [CLI command reference](#cli-command-reference)
- [TUI controls](#tui-controls)
- [Examples and workflows](#examples-and-workflows)
- [History and data storage](#history-and-data-storage)
- [Uninstall](#uninstall)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [Build and release process](#build-and-release-process)
- [Known limitations](#known-limitations)
- [License](#license)

## Features

- **CLI and TUI:** use commands from a shell or manage history in a terminal interface.
- **Clipboard history:** automatically records supported content copied through CyberClip.
- **History search and filtering:** filter by text, content type, pinned state, and result limit.
- **Restore from history:** restore an entry by its displayed history number without searching for the original item again.
- **Text clipboard:** copy text from the CLI and read the current system clipboard to standard output.
- **Images:** copy supported image files as image clipboard data.
- **Files and folders:** copy filesystem references, including multiple files and folders in one selection.
- **File browser:** browse files and folders from the terminal.
- **ZIP browsing:** inspect ZIP archive contents in the TUI (read-only; this is not an archive editor).
- **Storage report:** view mounted storage capacity and free-space information.
- **Pinning:** keep important history entries outside the rolling unpinned-history limit.
- **Per-user installation:** the default installer does not need administrator/root privileges.
- **Optional system installation:** install commands for all users when you explicitly choose the system installer.



## Screenshots

Explore CyberClip's terminal interface and features through the screenshots below.

### Clipboard History & Preview

![CyberClip History Preview](https://raw.githubusercontent.com/spoilpetal/Picture_Vault/main/CyberClip_pictures/v0.1.0/history_preview.png)

### Image Preview

![CyberClip Image Preview](https://raw.githubusercontent.com/spoilpetal/Picture_Vault/main/CyberClip_pictures/v0.1.0/image_preview.png)

### Archive Viewer

![CyberClip Archive Viewer](https://raw.githubusercontent.com/spoilpetal/Picture_Vault/main/CyberClip_pictures/v0.1.0/archive_view.png)

### File Information

![CyberClip File Information](https://raw.githubusercontent.com/spoilpetal/Picture_Vault/main/CyberClip_pictures/v0.1.0/file_information.png)

### TUI Storage Information

![CyberClip TUI Storage](https://raw.githubusercontent.com/spoilpetal/Picture_Vault/main/CyberClip_pictures/v0.1.0/tui_storage_view.png)

### CLI Storage Information

![CyberClip CLI Storage](https://raw.githubusercontent.com/spoilpetal/Picture_Vault/main/CyberClip_pictures/v0.1.0/cli_storage_view.png)

### Code Highlighting

![CyberClip Code Highlighting](https://raw.githubusercontent.com/spoilpetal/Picture_Vault/main/CyberClip_pictures/v0.1.0/code_highlight.png)

---

*Screenshots are hosted separately in [Picture_Vault](https://github.com/spoilpetal/Picture_Vault/tree/main/CyberClip_pictures/v0.1.0) to keep the CyberClip source repository lightweight.*


## Platform support and testing status

| Platform | Clipboard implementation |  hands-on runtime testing |
|---|---|---|
| Linux + Wayland | Native Wayland protocol backend (`wl-clipboard-rs`) | **Tested** |
| Linux + X11 | `clipboard-rs` X11 backend | Implemented; needs independent X11 runtime verification |
| Windows x86_64 | Native Win32 backend through `clipboard-rs` | Not tested by me; needs Windows CI and real-device verification |
| macOS Intel | Native macOS pasteboard backend through `clipboard-rs` | Not tested by me; needs macOS CI and real-device verification |
| macOS Apple Silicon | Native macOS pasteboard backend through `clipboard-rs` | Not tested by me; needs macOS CI and real-device verification |
| FreeBSD / OpenBSD / NetBSD | No supported backend currently enabled | **Not supported** |

The project includes GitHub Actions workflows intended to check and build Linux, Windows, and macOS targets on native runners. A successful cross-platform build confirms compilation for that target; it does **not** prove that every clipboard operation behaves correctly on every desktop, window manager, compositor, or OS version. Reports and testing from Windows and macOS users are welcome.

Linux needs an active graphical session to access the desktop clipboard. A headless server without a clipboard session is not a supported clipboard runtime.

## Requirements

### Building from source

- Rust toolchain with Cargo (install through [rustup](https://rustup.rs/)).
- Git, if cloning the repository.
- A supported desktop session for runtime clipboard operations.

The repository includes `Cargo.lock`; source installation uses `cargo build --locked --release` so Cargo uses the committed dependency versions.

### Linux

CyberClip uses native Rust clipboard libraries. It does not require the external `wl-copy` or `wl-paste` command-line programs.

- Wayland: a running Wayland session and a compositor that provides the standard clipboard protocols.
- X11: a running X11 desktop session and its clipboard selection service.
- Arch, Fedora, Debian, Ubuntu, and other Linux distributions use the Linux backend selected at compile time.

## Install from source



### Linux — wayland user (recommended)

```bash
git clone https://github.com/spoilpetal/CyberClip.git
cd CyberClip
chmod +x install.sh
./install.sh
```

The installer builds the release binary and asks which command name(s) to install:

1. `cyberclip` only
2. `cpl` only
3. both names
4. cancel

The default installation is under `~/.local/bin`. It does not require `sudo`. If the installer adds this directory to your shell's `PATH`, open a new terminal or reload your shell configuration.

For Bash:

```bash
source ~/.bashrc
```

For Zsh:

```bash
source ~/.zshrc
```

For Fish:

```fish
fish_add_path ~/.local/bin
```

If `~/.local/bin` is already in `PATH`, no shell configuration change is needed.

### Linux — all users (optional)

A system-wide install changes a shared executable directory and requires administrator privileges:

```bash
sudo ./install.sh --system
```

Alternatively, use the system installer directly:

```bash
sudo ./install-system.sh
```

The system install places commands under `/usr/local/bin`. Use it only if you want CyberClip available to all local accounts.

### Windows — current user

Open PowerShell in the extracted project directory:

```powershell
.\install.ps1
```

The installer builds CyberClip for Windows and installs under:

```text
%LOCALAPPDATA%\CyberClip\bin
```

It adds that directory to the current user's `PATH`. Open a new PowerShell or terminal window after installation.

### Windows — all users (optional)

Run PowerShell as Administrator:

```powershell
.\install.ps1 -System
```

This installs under Program Files and updates the machine-level `PATH`.

### macOS

The source includes a macOS clipboard backend. A source build/install must be run on macOS itself with Rust/Cargo installed:

```bash
./install.sh
```

Build from source on a Mac is recommended. macOS runtime behavior has not been hands-on tested by the me yet.

## Build manually

From the project directory:

```bash
cargo check --locked
cargo test --locked
cargo build --locked --release
```

The release executable is written to:

- Linux/macOS: `target/release/cyberclip`
- Windows: `target\release\cyberclip.exe`

Run the program directly without installing it:

```bash
cargo run --locked -- --help
cargo run --locked -- history
cargo run --locked -- ui
```

## Choose your command name

CyberClip can be installed under either command name:

- `cyberclip` — descriptive command name.
- `cpl` — short alias.

Both names run the same application and expose the same commands. `cpl` is not intended to replace the standard `cp` command. The installer lets you select either name or both.

Check the installed version:

```bash
cyberclip --version
cpl --version
```

Use whichever name you selected in the examples below.

## CLI command reference

The examples use `cpl`. Replace it with `cyberclip` if that is the command you installed.

### General help and version

```bash
cpl --help
cpl <COMMAND> --help
cpl --version
```

### `copy` — copy files, folders, or mixed selections

```bash
cpl copy ./report.pdf
cpl copy ./photo.png
cpl copy ./folder
cpl copy ./report.pdf ./photo.png ./folder
```

Copying a single supported image file places image data on the system clipboard. Copying files/folders places native file references on the clipboard so compatible applications can receive the selection.

Options for selecting the direct children of a directory:

```bash
cpl copy --files ./folder
cpl copy --folders ./folder
```

- `--files DIR`: copy files directly inside `DIR` (not recursive).
- `--folders DIR`: copy folders directly inside `DIR` (not recursive).
- Without either option, each supplied path itself is copied; a supplied directory is copied as a folder.
- Multiple paths can be supplied together for a mixed selection.

### `text` — copy text to the system clipboard

```bash
cpl text "Hello from CyberClip"
cpl text "A multi-word note"
```

The text is copied to the system clipboard and added to history.

### `paste` — print current clipboard text to stdout

```bash
cpl paste
```

`paste` reads the **current system clipboard**, not an old history entry. It writes the clipboard text bytes to standard output and does not add a newline. This makes it suitable for shell pipelines and redirection:

```bash
cpl paste | grep "search term"
cpl paste > clipboard.txt
cpl paste | wc -c
```

This command is for text output. It does not simulate keyboard input, type into the shell prompt, or dump image/file binary data into the terminal. To restore a previous history item, use `history copy NUMBER`.

### `history` — list and filter saved entries

```bash
cpl history
cpl history --limit 20
cpl history --search "meeting"
cpl history --type text
cpl history --type image
cpl history --type file
cpl history --type folder
cpl history --type files
cpl history --pinned
cpl history --search "report" --limit 10
```

Available content types are `text`, `image`, `file`, `folder`, and `files`.

History numbers are one-based. The number shown in the output is the number to use with `history copy`, `remove`, or `pin`. When filters are applied, displayed numbers still refer to the original full history list.

### `history copy` — restore an existing history item

```bash
cpl history copy 3
```

This restores the third entry in the full history list to the system clipboard. It works for saved text, images, files, folders, and mixed file selections, subject to the original files still existing at their saved paths.

### `search` — search clipboard history

```bash
cpl search "meeting"
cpl search "report.pdf"
```

Search results show the original history numbers. Restore a result directly:

```bash
cpl history copy 5
```

You do not need to locate or copy the original source again. For file and image history entries, the original path must still exist to restore the content.

### `pin` — toggle an entry's pinned state

```bash
cpl pin 2
```

Run the command again to unpin the same entry. Pinned entries are retained when the rolling unpinned history is trimmed.

### `remove` — remove one history entry

```bash
cpl remove 4
```

This removes the numbered entry from CyberClip's saved history. It does not delete the original file from disk.

### `clear` — clear clipboard history and CyberClip's current file selection

```bash
cpl clear
```

This clears CyberClip's saved history and current file-selection snapshot, and clears the system clipboard through the active backend. It does not delete source files from disk.

### `ui` — open the terminal user interface

```bash
cpl ui
```

Use the TUI for browsing history, previewing entries, searching, restoring, pinning, and removing items.

### `files` — open the file browser

```bash
cpl files
```

Opens CyberClip's terminal file browser for navigating and selecting filesystem items. Exact navigation and preview behavior may depend on the file type and terminal capabilities.

### `storage` — show storage information

```bash
cpl storage
```

Prints a report about detected storage volumes and their capacity/free-space information.

### `get` — extract text from a file and copy it

`get` reads text from a file, copies the selected content to the system clipboard, records it in history, and prints the extracted content.

Copy the whole file:

```bash
cpl get README.md
```

Search for matching lines:

```bash
cpl get "clipboard" README.md
```

Extract one line:

```bash
cpl get line 12 README.md
```

Extract a range of lines (inclusive):

```bash
cpl get line 5:20 README.md
```

Suppress line numbers in search/range output:

```bash
cpl get --no-numbers "clipboard" README.md
cpl get --no-numbers line 5:20 README.md
```

The `get` command is intended for text files. If no content matches, the clipboard is left unchanged.

## TUI controls

Open the interface:

```bash
cpl ui
```

The history TUI supports these controls:

| Key | Action |
|---|---|
| `↑` / `↓` | Move through history entries |
| `Page Up` / `Page Down` | Scroll the preview |
| `/` | Start searching history |
| `Enter` | Restore the selected history entry to the system clipboard |
| `C` | Copy/restore the selected history entry to the system clipboard |
| `P` | Toggle pin/unpin for the selected entry |
| `Delete` | Remove the selected history entry |
| `S` | Toggle the storage information view |
| `Esc` | Exit search mode / leave the current interaction |
| `Q` | Quit the TUI |

`Ctrl+V` is handled by your terminal/application as the normal paste shortcut where supported; it is not the CyberClip history-copy shortcut. Use `C` or `Enter` inside CyberClip to restore the selected history item to the system clipboard, then paste into another application normally.

## Examples and workflows

### Copy text, then retrieve it in a shell

```bash
cpl text "Build completed"
cpl paste
```

### Copy the current clipboard text to a file

```bash
cpl paste > saved-note.txt
```

### Search old history and restore a result

```bash
cpl search "invoice"
cpl history copy 7
```

### Copy a folder's files only

```bash
cpl copy --files ~/Documents
```

### Copy a mixed selection

```bash
cpl copy ~/Documents/report.pdf ~/Pictures/photo.png ~/Projects
```

### Keep an important entry

```bash
cpl pin 3
cpl history --pinned
```

## History and data storage

CyberClip stores its history and file-selection metadata in the operating system's normal per-user data directory, under a `cyberclip` folder. It does not store the history inside the project source directory or install directory.

The exact parent directory is platform-dependent:

- Linux: typically `$XDG_DATA_HOME` or `~/.local/share`.
- Windows: the user's local application data directory.
- macOS: the user's application data directory as resolved by the platform's standard directories library.

History is local to the current user account. It is not synced to a cloud service.

CyberClip keeps pinned entries and up to 100 newest unpinned history entries. Pinned entries are exempt from that rolling unpinned limit. Removing an entry from history does not remove the corresponding source file.

## Uninstall

### Linux — current user

From the extracted project directory:

```bash
./uninstall.sh
```

The uninstaller removes only the commands recorded by the user installer and restores any command backups it made.

### Linux — system install

```bash
sudo ./uninstall-system.sh
```

### Windows

For a current-user install:

```powershell
.\uninstall.ps1
```

For a system install, run PowerShell as Administrator:

```powershell
.\uninstall.ps1 -System
```

Uninstalling the executable does not automatically delete the user's clipboard history/data folder. Back up or remove that data yourself if you want to erase it.

## Troubleshooting

### `cpl` or `cyberclip` command not found

- Open a new terminal after installation.
- Confirm the install directory is in your `PATH`.
- Linux default user install: check `~/.local/bin`.
- Windows user install: check `%LOCALAPPDATA%\CyberClip\bin`.

### Clipboard access fails on Linux

- Confirm you are inside a graphical Wayland or X11 session.
- For Wayland, check that the environment identifies the active session (`WAYLAND_DISPLAY`).
- For X11, check that `DISPLAY` is set and the X server is available.
- Remote/headless shells without access to the desktop clipboard cannot access the desktop selection.

### A history file or image cannot be restored

File and image history stores source paths. If the original file was moved, renamed, or deleted, CyberClip cannot reconstruct it from the path alone. Restore requires the original path to remain accessible.

### Windows or macOS issue

Please include:
- OS version and architecture.
- CyberClip version (`cyberclip --version` or `cpl --version`).
- Exact command or TUI action.
- Full error output.
- Whether the failure occurs in a native terminal session or remote/virtualized session.

Because i don't currently have Windows/macOS machines, detailed reports and independent testing are especially valuable.

## Contributing

Contributions, bug reports, platform testing, documentation improvements, and code review are welcome.

A useful contribution workflow:

```bash
git clone https://github.com/spoilpetal/CyberClip.git
cd CyberClip
cargo check --locked
cargo test --locked
cargo build --locked --release
```



When opening an issue, include reproducible steps, expected behavior, actual behavior, OS/session details, and relevant logs. Please avoid including private clipboard contents, tokens, passwords, or other sensitive data in issue reports.

For platform contributions, please state whether you tested on a real machine, a VM, or CI. In particular, Windows and macOS runtime testing is needed to confirm the native clipboard backends.

## Build and release process

The repository contains GitHub Actions workflows:

- `.github/workflows/ci.yml` — checks, tests, and builds on the configured Linux, Windows, and macOS runners.
- `.github/workflows/release.yml` — builds platform release archives when a version tag is pushed.

To publish a release after pushing the repository:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release workflow is configured for these targets:

- Linux x86_64
- Linux ARM64
- Windows x86_64
- macOS Intel
- macOS Apple Silicon

Check the GitHub Actions run and the resulting Release page before announcing that a platform asset is available. A configured workflow is not the same thing as a completed, successful build. The source installers currently build from source; the platform release archives are downloaded separately from GitHub Releases.

## Known limitations

- Windows and macOS source backends are implemented but have not been hands-on tested by the me. Independent native CI and real-device validation are still required.
- Linux X11 is implemented but should receive independent runtime testing in addition on the  Wayland testing.
- BSD systems are not supported yet.
- `paste` prints text only. It is not a binary export command for images or file selections.
- Restoring file/image history depends on the original source path still existing.
- ZIP archive browsing is read-only.
- Clipboard operations require an active desktop clipboard session.


## License and copyright 
### Take good care of this project (but not permit to sell)

<br>

this software comes with no warranty ! (cuz i had only test it only in Wayland linux and for other system i use rust native lib to function and source from the internet so i can't give guarantee of this software will work on system i mention !!!) 
