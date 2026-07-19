#!/bin/sh
set -eu

RELEASE_TAG="${CODEX_METERS_RELEASE_TAG:-v0.1.0}"
ASSET_BASE_URL="${CODEX_METERS_ASSET_BASE_URL:-https://raw.githubusercontent.com/mrbryside/codex-meters/main/releases/${RELEASE_TAG}}"
PKG_URL="${1:-${CODEX_METERS_PKG_URL:-${ASSET_BASE_URL}/Codex%20Meters.pkg}}"
APP_PATH="/Applications/Codex Meters.app"
INFO_PLIST="$APP_PATH/Contents/Info.plist"
BINARY_PATH="$APP_PATH/Contents/MacOS/codex-token-meter"
EXPECTED_BUNDLE_ID="com.codex.tokenmeter"
PACKAGE_IDENTIFIER="com.codex.tokenmeter"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VERIFY_PKG_SCRIPT="$SCRIPT_DIR/scripts/verify-pkg.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM
pkg_path="$tmp_dir/Codex-Meters.pkg"

check_bundle() {
  if [ ! -d "$APP_PATH" ]; then
    return 1
  fi

  if [ -L "$APP_PATH" ]; then
    echo "App path is a symlink, expected a real bundle: $APP_PATH" >&2
    return 1
  fi

  if [ ! -f "$INFO_PLIST" ] || [ ! -f "$BINARY_PATH" ]; then
    return 1
  fi

  actual_bundle_id="$(/usr/bin/defaults read "$INFO_PLIST" CFBundleIdentifier 2>/dev/null || true)"
  [ "$actual_bundle_id" = "$EXPECTED_BUNDLE_ID" ]
}

validate_package() {
  if [ ! -s "$pkg_path" ]; then
    echo "Downloaded package is missing or empty: $pkg_path" >&2
    return 1
  fi

  if [ -x "$VERIFY_PKG_SCRIPT" ]; then
    "$VERIFY_PKG_SCRIPT" "$pkg_path"
    return 0
  fi

  verify_root="$(mktemp -d)"
  if ! /usr/sbin/pkgutil --expand "$pkg_path" "$verify_root"; then
    echo "Downloaded package failed to expand: $pkg_path" >&2
    rm -rf "$verify_root"
    return 1
  fi

  if [ ! -r "$verify_root/PackageInfo" ] || [ ! -r "$verify_root/Payload" ] || [ ! -f "$verify_root/Scripts/postinstall" ]; then
    echo "Downloaded package metadata is incomplete: $pkg_path" >&2
    rm -rf "$verify_root"
    return 1
  fi

  rm -rf "$verify_root"
  return 0
}

run_install() {
  sudo /usr/sbin/installer -pkg "$pkg_path" -target /
}

echo "Downloading Codex Meters..."
if ! /usr/bin/curl -fL --progress-bar "$PKG_URL" -o "$pkg_path"; then
  echo "Failed to download package from: $PKG_URL" >&2
  exit 1
fi

if ! validate_package; then
  exit 1
fi

echo "Installing Codex Meters..."
if ! run_install; then
  echo "Installer failed." >&2
  exit 1
fi

if [ ! -d "/Applications/Codex Meters.app" ]; then
  if /usr/sbin/pkgutil --pkg-info "$PACKAGE_IDENTIFIER" >/dev/null 2>&1; then
    echo "Installer did not place the app in /Applications; retrying after clearing stale receipt." >&2
    /usr/sbin/pkgutil --forget "$PACKAGE_IDENTIFIER" >/dev/null 2>&1 || true
    if ! run_install; then
      echo "Retry installation failed." >&2
      exit 1
    fi
  fi
fi

if ! check_bundle; then
  echo "Install validation failed. Expected app at $APP_PATH with bundle id $EXPECTED_BUNDLE_ID." >&2
  exit 1
fi

if [ ! -x "$BINARY_PATH" ]; then
  echo "Missing executable in installed app bundle: $BINARY_PATH" >&2
  exit 1
fi

echo "Codex Meters installed."
