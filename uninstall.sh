#!/usr/bin/env sh
set -eu

MODE=user
PURGE_DATA=0
for arg in "$@"; do
  case "$arg" in
    --system) MODE=system ;;
    --purge-data) PURGE_DATA=1 ;;
    -h|--help)
      echo 'Usage: ./uninstall.sh [--system] [--purge-data]'
      echo '  --system      Remove the all-users installation under /usr/local (requires root).'
      echo '  --purge-data  Also delete this user\047s CyberClip history and saved data.'
      exit 0 ;;
    *) echo "Unknown option: $arg" >&2; exit 2 ;;
  esac
done

if [ "$MODE" = system ]; then
  [ "$(id -u)" -eq 0 ] || { echo 'System uninstall requires root; run sudo ./uninstall.sh --system' >&2; exit 1; }
  INSTALL_ROOT=${CYBERCLIP_INSTALL_ROOT:-/usr/local}
else
  INSTALL_ROOT=${CYBERCLIP_INSTALL_ROOT:-$HOME/.local}
fi
BIN_DIR="$INSTALL_ROOT/bin"
STATE_DIR="$INSTALL_ROOT/share/cyberclip"
STATE_FILE="$STATE_DIR/installed-commands"

MANAGED_CYBERCLIP=0
MANAGED_CPL=0
if [ -f "$STATE_FILE" ]; then
  # Remove only names recorded as CyberClip-managed, then restore any command
  # that was backed up before CyberClip replaced it.
  while IFS= read -r rec; do
    case "$rec" in
      cyberclip) MANAGED_CYBERCLIP=1; rm -f "$BIN_DIR/cyberclip" ;;
      cpl) MANAGED_CPL=1; rm -f "$BIN_DIR/cpl" ;;
    esac
  done < "$STATE_FILE"
  while IFS= read -r rec; do
    case "$rec" in
      backup:cyberclip|backup:cpl)
        name=${rec#backup:}
        backup="$STATE_DIR/$name.previous"
        if [ -e "$backup" ] || [ -L "$backup" ]; then
          mkdir -p "$BIN_DIR"
          mv "$backup" "$BIN_DIR/$name"
        fi
        ;;
    esac
  done < "$STATE_FILE"
  rm -f "$STATE_FILE"
else
  echo "No CyberClip install record found at $STATE_FILE."
fi

# Older releases may have left a command without a state record. Ask before
# removing it, and only inspect this install root (never other PATH locations).
for name in cyberclip cpl; do
  managed=0
  [ "$name" = cyberclip ] && managed=$MANAGED_CYBERCLIP
  [ "$name" = cpl ] && managed=$MANAGED_CPL
  if [ "$managed" -eq 0 ] && { [ -e "$BIN_DIR/$name" ] || [ -L "$BIN_DIR/$name" ]; }; then
    printf "Untracked '%s' exists in %s. Remove it as an old CyberClip command? [y/N]: " "$name" "$BIN_DIR"
    IFS= read -r answer
    case "$answer" in y|Y|yes|YES) rm -f "$BIN_DIR/$name" ;; esac
  fi
done

# Remove empty installer-owned directories only; never recursively delete a
# shared prefix such as /usr/local or ~/.local.
rmdir "$STATE_DIR" 2>/dev/null || true
rmdir "$INSTALL_ROOT/share" 2>/dev/null || true

if [ "$MODE" = user ]; then
  # Remove only the exact PATH lines written by the installer. Other PATH
  # customizations and unrelated applications are left untouched.
  for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile" \
            "${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"; do
    [ -f "$rc" ] || continue
    tmp="$rc.cyberclip-uninstall.$$"
    awk '
      $0 == "# CyberClip command-line tools" { skip=1; next }
      skip && ($0 ~ /^export PATH=".*\.local\/bin:\$PATH"$/ || $0 ~ /^fish_add_path ".*\.local\/bin"$/) { skip=0; next }
      { if (skip) skip=0; print }
    ' "$rc" > "$tmp"
    if ! cmp -s "$rc" "$tmp"; then cat "$tmp" > "$rc"; fi
    rm -f "$tmp"
  done
fi

if [ "$PURGE_DATA" -eq 1 ]; then
  if [ "$MODE" = user ]; then
    case "$(uname -s 2>/dev/null || echo unknown)" in
      Darwin) DATA_ROOT="$HOME/Library/Application Support/cyberclip" ;;
      *) DATA_ROOT="${XDG_DATA_HOME:-$HOME/.local/share}/cyberclip" ;;
    esac
    CONFIG_ROOT="${XDG_CONFIG_HOME:-$HOME/.config}/cyberclip"
    CACHE_ROOT="${XDG_CACHE_HOME:-$HOME/.cache}/cyberclip"
    printf 'Permanently delete CyberClip data in:\n  %s\n  %s\n  %s\n' "$DATA_ROOT" "$CONFIG_ROOT" "$CACHE_ROOT"
    printf 'Continue? [y/N]: '
    IFS= read -r answer
    case "$answer" in y|Y|yes|YES) rm -rf "$DATA_ROOT" "$CONFIG_ROOT" "$CACHE_ROOT" ;; *) echo 'User data preserved.' ;; esac
  else
    echo '--purge-data is supported only for the current-user installation.' >&2
    exit 2
  fi
else
  echo 'Clipboard history and user data were preserved. Use --purge-data to remove them too.'
fi

if [ "$MODE" = system ]; then
  echo 'System CyberClip commands and installer metadata removed; previous commands restored where available.'
else
  echo 'Current-user CyberClip commands and installer metadata removed; previous commands restored where available.'
fi
