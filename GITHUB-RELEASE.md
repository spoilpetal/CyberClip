# GitHub binary release procedure

This repository is configured to build native binaries when a version tag is pushed.

1. Push the contents of this source archive to the repository.
2. Confirm the repository's Actions tab shows the `Build and test` workflow passing.
3. Create and push a version tag matching the Cargo.toml version:
   ```sh
   git tag v0.1.0
   git push origin v0.1.0
   ```
4. The `Build platform release binaries` workflow builds five targets and publishes archives plus SHA-256 files to the GitHub Release.

Targets:
- `linux-x86_64` — GNU/Linux x86_64
- `linux-aarch64` — GNU/Linux ARM64
- `windows-x86_64` — Windows 64-bit
- `macos-x86_64` — Intel macOS
- `macos-aarch64` — Apple Silicon macOS

The workflow builds on native GitHub-hosted runners. It does not claim BSD support. A CI build confirms compilation, not clipboard behavior on every desktop/compositor; real-desktop validation remains necessary.

The current source installer (`install.sh` / `install.ps1`) remains a build-from-source installer. These release assets are currently intended for manual download from GitHub Releases; automatic prebuilt installation is a separate installer change.
