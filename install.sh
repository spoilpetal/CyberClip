#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
MODE=user
case "${1:-}" in
  "") ;;
  --system) MODE=system ;;
  -h|--help)
    echo "Usage: ./install.sh [--system]"
    echo "Default: install for the current user in ~/.local/bin."
    echo "--system: install for all users in /usr/local/bin (requires root)."
    exit 0 ;;
  *) echo "Unknown option: $1" >&2; exit 2 ;;
esac

OS_NAME=$(uname -s 2>/dev/null || echo unknown)
case "$OS_NAME" in
  Linux)
    if [ -n "${WAYLAND_DISPLAY:-}" ]; then BACKEND="Linux / Wayland (wl-clipboard-rs)";
    elif [ -n "${DISPLAY:-}" ]; then BACKEND="Linux / X11 (clipboard-rs)";
    else BACKEND="Linux / no graphical session detected"; fi ;;
  Darwin) BACKEND="macOS (clipboard-rs / native pasteboard)" ;;
  MINGW*|MSYS*|CYGWIN*)
    echo "For native Windows installation, open PowerShell and run .\\install.ps1" >&2
    exit 1 ;;
  FreeBSD|OpenBSD|NetBSD)
    echo "Detected $OS_NAME, but CyberClip's clipboard backend is not implemented for BSD yet." >&2
    echo "The current clipboard-rs backend officially targets Linux/X11, Windows and macOS; Wayland uses wl-clipboard-rs." >&2
    echo "Installation stopped rather than installing a build with non-working clipboard support." >&2
    exit 1 ;;
  *) echo "Unsupported OS: $OS_NAME. No clipboard backend is configured." >&2; exit 1 ;;
esac
printf 'Detected platform: %s\n' "$BACKEND"

if [ "$MODE" = system ]; then
  if [ "$(id -u)" -ne 0 ]; then
    echo "System installation requires administrator privileges." >&2
    echo "Run: sudo ./install.sh --system" >&2
    exit 1
  fi
  INSTALL_ROOT=${CYBERCLIP_INSTALL_ROOT:-/usr/local}
else
  INSTALL_ROOT=${CYBERCLIP_INSTALL_ROOT:-$HOME/.local}
fi
BIN_DIR="$INSTALL_ROOT/bin"
STATE_DIR="$INSTALL_ROOT/share/cyberclip"
STATE_FILE="$STATE_DIR/installed-commands"

CARGO_BIN=$(command -v cargo 2>/dev/null || true)
if [ -z "$CARGO_BIN" ] && [ "$MODE" = system ] && [ -n "${SUDO_USER:-}" ]; then
  SUDO_HOME=$(getent passwd "$SUDO_USER" 2>/dev/null | cut -d: -f6 || true)
  if [ -n "$SUDO_HOME" ] && [ -x "$SUDO_HOME/.cargo/bin/cargo" ]; then
    CARGO_BIN="$SUDO_HOME/.cargo/bin/cargo"
    export CARGO_HOME="$SUDO_HOME/.cargo" RUSTUP_HOME="$SUDO_HOME/.rustup"
    PATH="$SUDO_HOME/.cargo/bin:$PATH"; export PATH
  fi
fi
if [ -z "$CARGO_BIN" ]; then
  echo "Error: Rust/Cargo is required to build from source." >&2
  echo "For a non-technical installation, use a prebuilt release package when available." >&2
  exit 1
fi

echo "Building CyberClip for this machine..."
"$CARGO_BIN" build --locked --release --manifest-path "$SCRIPT_DIR/Cargo.toml"
BUILT_BINARY="$SCRIPT_DIR/target/release/cyberclip"
[ -x "$BUILT_BINARY" ] || { echo "Release binary missing: $BUILT_BINARY" >&2; exit 1; }
mkdir -p "$BIN_DIR" "$STATE_DIR"

# Read our previous install record so switching from both names to one removes
# the old CyberClip-managed command without touching unrelated commands.
OLD_CYBERCLIP=0; OLD_CPL=0; OLD_BACKUP_CYBERCLIP=0; OLD_BACKUP_CPL=0
if [ -f "$STATE_FILE" ]; then
  while IFS= read -r rec; do
    case "$rec" in
      cyberclip) OLD_CYBERCLIP=1 ;;
      cpl) OLD_CPL=1 ;;
      backup:cyberclip) OLD_BACKUP_CYBERCLIP=1 ;;
      backup:cpl) OLD_BACKUP_CPL=1 ;;
    esac
  done < "$STATE_FILE"
fi

is_present() { [ -e "$BIN_DIR/$1" ] || [ -L "$BIN_DIR/$1" ]; }
find_existing() {
  name=$1
  found=$(command -v "$name" 2>/dev/null || true)
  if [ -n "$found" ]; then printf '  %s: %s\n' "$name" "$found"; return 0; fi
  if is_present "$name"; then printf '  %s: %s\n' "$name" "$BIN_DIR/$name"; return 0; fi
  printf '  %s: not found\n' "$name"; return 1
}
echo "Checking existing commands:"
find_existing cyberclip || true
find_existing cpl || true
printf '\nChoose command name(s) to install:\n  1) cyberclip only\n  2) cpl only\n  3) both\n  4) cancel\nChoose [1-4]: '
IFS= read -r CHOICE
case "$CHOICE" in
  1) WANT_CYBERCLIP=1; WANT_CPL=0 ;;
  2) WANT_CYBERCLIP=0; WANT_CPL=1 ;;
  3) WANT_CYBERCLIP=1; WANT_CPL=1 ;;
  *) echo "Installation cancelled; no commands were changed."; exit 0 ;;
esac

# Older installer versions may have left a CyberClip executable without a state
# record. Ask before removing an untracked sibling command from our install bin.
STALE_REMOVE_CYBERCLIP=0; STALE_REMOVE_CPL=0
if [ "$WANT_CYBERCLIP" -eq 0 ] && [ "$OLD_CYBERCLIP" -eq 0 ] && is_present cyberclip; then
  printf "An untracked 'cyberclip' command exists in %s. Remove it as an older CyberClip install? [y/N]: " "$BIN_DIR"
  IFS= read -r stale_answer
  case "$stale_answer" in y|Y|yes|YES) STALE_REMOVE_CYBERCLIP=1 ;; esac
fi
if [ "$WANT_CPL" -eq 0 ] && [ "$OLD_CPL" -eq 0 ] && is_present cpl; then
  printf "An untracked 'cpl' command exists in %s. Remove it as an older CyberClip install? [y/N]: " "$BIN_DIR"
  IFS= read -r stale_answer
  case "$stale_answer" in y|Y|yes|YES) STALE_REMOVE_CPL=1 ;; esac
fi

# A command already managed by this installer is an upgrade, not a foreign conflict.
CONFLICT=0
for name in cyberclip cpl; do
  case "$name" in
    cyberclip) want=$WANT_CYBERCLIP; old=$OLD_CYBERCLIP ;;
    cpl) want=$WANT_CPL; old=$OLD_CPL ;;
  esac
  if [ "$want" -eq 1 ] && [ "$old" -eq 0 ]; then
    found=$(command -v "$name" 2>/dev/null || true)
    if [ -n "$found" ] || is_present "$name"; then
      echo "WARNING: '$name' already exists at ${found:-$BIN_DIR/$name}."
      echo "The CyberClip command in $BIN_DIR may take priority for this user/system."
      CONFLICT=1
    fi
  fi
done
if [ "$CONFLICT" -eq 1 ]; then
  printf 'Continue and replace/override the selected command name(s)? [y/N]: '
  IFS= read -r CONFIRM
  case "$CONFIRM" in y|Y|yes|YES) ;; *) echo "Cancelled; existing commands were left untouched."; exit 0 ;; esac
fi

NEW_STATE="$STATE_FILE.new"
: > "$NEW_STATE"
restore_old_name() {
  name=$1
  old=$2
  had_backup=$3
  [ "$old" -eq 1 ] || return 0
  rm -f "$BIN_DIR/$name"
  if [ "$had_backup" -eq 1 ] && { [ -e "$STATE_DIR/$name.previous" ] || [ -L "$STATE_DIR/$name.previous" ]; }; then
    mv "$STATE_DIR/$name.previous" "$BIN_DIR/$name"
  fi
}
# Remove previously installed names that the user no longer selected.
[ "$WANT_CYBERCLIP" -eq 1 ] || restore_old_name cyberclip "$OLD_CYBERCLIP" "$OLD_BACKUP_CYBERCLIP"
[ "$WANT_CPL" -eq 1 ] || restore_old_name cpl "$OLD_CPL" "$OLD_BACKUP_CPL"
[ "$STALE_REMOVE_CYBERCLIP" -eq 0 ] || rm -f "$BIN_DIR/cyberclip"
[ "$STALE_REMOVE_CPL" -eq 0 ] || rm -f "$BIN_DIR/cpl"

install_name() {
  name=$1
  old=$2
  had_backup=$3
  target="$BIN_DIR/$name"
  backup="$STATE_DIR/$name.previous"
  if [ "$old" -eq 1 ]; then
    rm -f "$target"
    if [ "$had_backup" -eq 1 ]; then printf 'backup:%s\n' "$name" >> "$NEW_STATE"; fi
  elif is_present "$target"; then
    # Preserve an existing file in the install directory before replacing it.
    if [ -e "$backup" ] || [ -L "$backup" ]; then
      echo "Refusing to overwrite an existing backup: $backup" >&2; exit 1
    fi
    mv "$target" "$backup"
    printf 'backup:%s\n' "$name" >> "$NEW_STATE"
  fi
  if [ "$name" = cpl ] && [ "$WANT_CYBERCLIP" -eq 1 ]; then
    ln -s cyberclip "$target"
  else
    cp "$BUILT_BINARY" "$target"
    chmod 755 "$target"
  fi
  printf '%s\n' "$name" >> "$NEW_STATE"
}
[ "$WANT_CYBERCLIP" -eq 0 ] || install_name cyberclip "$OLD_CYBERCLIP" "$OLD_BACKUP_CYBERCLIP"
[ "$WANT_CPL" -eq 0 ] || install_name cpl "$OLD_CPL" "$OLD_BACKUP_CPL"
mv "$NEW_STATE" "$STATE_FILE"

if [ "$MODE" = user ]; then
  case ":${PATH}:" in *":$BIN_DIR:"*) : ;; *)
    SHELL_NAME=$(basename "${SHELL:-sh}")
    case "$SHELL_NAME" in
      fish) RC_FILE="${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"; mkdir -p "$(dirname "$RC_FILE")"; LINE="fish_add_path \"$BIN_DIR\"" ;;
      zsh) RC_FILE="$HOME/.zshrc"; LINE="export PATH=\"$BIN_DIR:\$PATH\"" ;;
      bash) RC_FILE="$HOME/.bashrc"; LINE="export PATH=\"$BIN_DIR:\$PATH\"" ;;
      *) RC_FILE="$HOME/.profile"; LINE="export PATH=\"$BIN_DIR:\$PATH\"" ;;
    esac
    touch "$RC_FILE"
    grep -Fqx "$LINE" "$RC_FILE" || printf '\n# CyberClip command-line tools\n%s\n' "$LINE" >> "$RC_FILE"
    echo "Added $BIN_DIR to PATH in $RC_FILE. Reload your shell." ;;
  esac
else
  echo "System commands installed in $BIN_DIR for all users."
fi
printf 'CyberClip installed command(s):'
[ "$WANT_CYBERCLIP" -eq 0 ] || printf ' cyberclip'
[ "$WANT_CPL" -eq 0 ] || printf ' cpl'
printf '\n'
