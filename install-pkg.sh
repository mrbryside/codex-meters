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
EXPECTED_BUNDLE_PATH="./Applications/Codex Meters.app/Contents/MacOS/codex-token-meter"
FORBIDDEN_BUNDLE_PATH="^\\./Codex Meters\\.app(/|$)"
EXPECTED_POSTINSTALL_PATH="./postinstall"

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

  local_verify_dir="$tmp_dir/verify"
  mkdir -p "$local_verify_dir"
  payload_file_list="$local_verify_dir/payload-files.txt"
  script_file_list="$local_verify_dir/scripts-files.txt"

  if ! /usr/sbin/pkgutil --payload-files "$pkg_path" > "$payload_file_list"; then
    echo "Downloaded package payload inspection failed: $pkg_path" >&2
    return 1
  fi

  if ! /usr/bin/grep -qxF "$EXPECTED_BUNDLE_PATH" "$payload_file_list"; then
    echo "Payload missing executable bundle path: $EXPECTED_BUNDLE_PATH" >&2
    return 1
  fi

  if /usr/bin/grep -Eq "$FORBIDDEN_BUNDLE_PATH" "$payload_file_list"; then
    echo "Payload still uses relocatable root bundle path: ./Codex Meters.app" >&2
    return 1
  fi

  if ! /usr/bin/xar -xf "$pkg_path" -C "$local_verify_dir" PackageInfo; then
    echo "Unable to extract PackageInfo from package: $pkg_path" >&2
    return 1
  fi

  package_info="$local_verify_dir/PackageInfo"
  if [ ! -r "$package_info" ]; then
    echo "PackageInfo not readable: $pkg_path" >&2
    return 1
  fi

  if ! /usr/bin/xmllint --noout "$package_info" >/dev/null 2>&1; then
    echo "PackageInfo is not readable XML: $pkg_path" >&2
    return 1
  fi

  if ! /usr/bin/grep -q '^<pkg-info ' "$package_info"; then
    echo "PackageInfo missing expected pkg-info metadata: $pkg_path" >&2
    return 1
  fi

  install_location="$(
    /usr/bin/xmllint --xpath 'string(/pkg-info/@install-location)' "$package_info" 2>/dev/null || true
  )"
  if [ "$install_location" != "/" ]; then
    echo "PackageInfo has unexpected install-location: ${install_location:-<missing>} (expected /)" >&2
    return 1
  fi

  if /usr/bin/grep -q "<relocate" "$package_info"; then
    echo "PackageInfo contains relocate metadata; install must be non-relocatable." >&2
    return 1
  fi

  if ! /usr/bin/xar -xf "$pkg_path" -C "$local_verify_dir" Scripts; then
    echo "Unable to extract package scripts from package: $pkg_path" >&2
    return 1
  fi

  if [ ! -r "$local_verify_dir/Scripts" ]; then
    echo "Postinstall script is not embedded in package scripts payload." >&2
    return 1
  fi

  if ! /usr/bin/gunzip -c "$local_verify_dir/Scripts" | /usr/bin/cpio -it > "$script_file_list"; then
    echo "Failed to read embedded package scripts payload: $pkg_path" >&2
    return 1
  fi

  if ! /usr/bin/grep -qxF "$EXPECTED_POSTINSTALL_PATH" "$script_file_list"; then
    echo "postinstall script is not embedded in package scripts payload." >&2
    return 1
  fi

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
