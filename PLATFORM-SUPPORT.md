# CyberClip platform and clipboard backends

CyberClip does not download OS clipboard APIs at install time. Cargo compiles native Rust dependencies for the target platform; the resulting executable contains its backend.

| Platform | Current backend | Status |
|---|---|---|
| Linux + Wayland | `wl-clipboard-rs` native Wayland protocol provider | Implemented |
| Linux + X11 | `clipboard-rs` X11 backend | Implemented |
| Windows | `clipboard-rs` native Win32 backend | Implemented in source; needs Windows CI/runtime verification |
| macOS | `clipboard-rs` native pasteboard backend | Implemented in source; needs macOS CI/runtime verification |
| FreeBSD / OpenBSD / NetBSD | No backend enabled | Not supported yet; installer intentionally refuses to claim support |

Linux distributions such as Arch, Fedora/RHEL, Debian and Ubuntu share the same Linux clipboard backends. The distribution name does not require a separate clipboard API. A graphical session is required for a system clipboard; headless servers do not have one.

## Dependencies / upstream API references

- [`wl-clipboard-rs`](https://crates.io/crates/wl-clipboard-rs): Wayland clipboard protocol implementation.
- [`clipboard-rs`](https://crates.io/crates/clipboard-rs): Windows, macOS and Linux/X11 clipboard API.
- [`sysinfo`](https://crates.io/crates/sysinfo): mounted volume and free-space reporting.

`clipboard-rs` upstream currently documents Windows, macOS and Linux/X11. It does not list BSD in its supported platform table, so CyberClip does not enable that dependency on BSD targets. BSD support needs a separately implemented and tested backend (likely X11 selection ownership and, where available, Wayland data-control support) before it can be advertised.

## Installation scope

- `./install.sh`: current-user install under `~/.local/bin`; no administrator privileges.
- `sudo ./install.sh --system` or `./install-system.sh`: all-users install under `/usr/local/bin`.
- `./uninstall.sh` / `sudo ./uninstall-system.sh`: remove only the commands recorded by the corresponding installer and restore backups.
- Windows PowerShell: `./install.ps1` and `./uninstall.ps1`; installs in `%LOCALAPPDATA%\CyberClip\bin` and updates the current user's PATH.

A system install changes a shared command directory and requires administrator/root privileges. It is optional; the default is safer and does not affect other accounts. On Linux when using sudo, the installer looks for Cargo in the invoking user’s `~/.cargo/bin` as well.

The GitHub Actions workflow checks and builds Linux, Windows and macOS targets on their native runners. It does not claim BSD support; a BSD backend and BSD-native test runner are still required.
