#!/bin/sh
set -eu

APP_NAME="Codex Meters.app"
RELEASE_TAG="${CODEX_METERS_RELEASE_TAG:-v0.1.0}"
DMG_URL="${1:-${CODEX_METERS_DMG_URL:-https://github.com/mrbryside/codex-meters/releases/download/${RELEASE_TAG}/Codex%20Meters_0.1.0_aarch64.dmg}}"

if [ -z "$DMG_URL" ]; then
  echo "Usage: curl -fsSL https://raw.githubusercontent.com/mrbryside/codex-meters/main/install.sh | sh"
  echo "Or set CODEX_METERS_DMG_URL before running the installer."
  exit 1
fi

if [ "$(uname -m)" != "arm64" ]; then
  echo "Codex Meters currently supports Apple Silicon Macs only."
  exit 1
fi

tmp_dir="$(mktemp -d)"
mount_point="$tmp_dir/mount"
mkdir -p "$mount_point"

cleanup() {
  hdiutil detach "$mount_point" -quiet >/dev/null 2>&1 || true
  rm -rf "$tmp_dir"
}
trap cleanup EXIT INT TERM

echo "Downloading Codex Meters..."
curl -fL --progress-bar "$DMG_URL" -o "$tmp_dir/Codex-Meters.dmg"
hdiutil attach "$tmp_dir/Codex-Meters.dmg" -nobrowse -readonly -mountpoint "$mount_point" >/dev/null

app_source="$mount_point/$APP_NAME"
if [ ! -d "$app_source" ]; then
  echo "Could not find $APP_NAME in the downloaded DMG."
  exit 1
fi

echo "Where should Codex Meters be installed?"
echo "  1) /Applications"
echo "  2) ~/Applications"
echo "  3) Choose another folder"
printf "Choose [1-3]: "
read -r choice < /dev/tty

case "$choice" in
  1) install_dir="/Applications" ;;
  2) install_dir="$HOME/Applications" ;;
  3)
    printf "Enter destination folder: "
    read -r install_dir < /dev/tty
    install_dir="${install_dir/#\~/$HOME}"
    ;;
  *) echo "Invalid choice."; exit 1 ;;
esac

mkdir -p "$install_dir"
destination="$install_dir/$APP_NAME"

if [ -e "$destination" ]; then
  printf "$APP_NAME already exists in %s. Replace it? [y/N]: " "$install_dir"
  read -r replace < /dev/tty
  case "$replace" in
    y|Y|yes|YES) ;;
    *) echo "Installation cancelled."; exit 0 ;;
  esac
fi

if [ -w "$install_dir" ]; then
  rm -rf "$destination"
  ditto "$app_source" "$destination"
else
  echo "Administrator permission is required for $install_dir."
  sudo rm -rf "$destination"
  sudo ditto "$app_source" "$destination"
fi

echo "Installed Codex Meters to $destination"
