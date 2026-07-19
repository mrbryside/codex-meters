#!/bin/sh
set -eu

EXPECTED_BUNDLE_PATH="./Applications/Codex Meters.app/Contents/MacOS/codex-token-meter"
FORBIDDEN_BUNDLE_PATH="^\\./Codex Meters\\.app(/|$)"
EXPECTED_POSTINSTALL_PATH="./postinstall"

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <path-to-package>" >&2
  exit 1
fi

PACKAGE_PATH=$1

if [ ! -f "$PACKAGE_PATH" ]; then
  echo "Package not found: $PACKAGE_PATH" >&2
  exit 1
fi

if [ ! -s "$PACKAGE_PATH" ]; then
  echo "Package is empty: $PACKAGE_PATH" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
PAYLOAD_FILES="$TMP_DIR/payload-files.txt"
SCRIPT_FILES="$TMP_DIR/scripts-files.txt"
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM
mkdir -p "$TMP_DIR"

if ! /usr/sbin/pkgutil --payload-files "$PACKAGE_PATH" > "$PAYLOAD_FILES"; then
  echo "Unable to read package payload list: $PACKAGE_PATH" >&2
  exit 1
fi

if ! /usr/bin/grep -qxF "$EXPECTED_BUNDLE_PATH" "$PAYLOAD_FILES"; then
  echo "Payload missing executable bundle path: $EXPECTED_BUNDLE_PATH" >&2
  exit 1
fi

if /usr/bin/grep -Eq "$FORBIDDEN_BUNDLE_PATH" "$PAYLOAD_FILES"; then
  echo "Payload still uses relocatable root bundle path: ./Codex Meters.app" >&2
  exit 1
fi

if ! /usr/bin/xar -xf "$PACKAGE_PATH" -C "$TMP_DIR" PackageInfo; then
  echo "Unable to extract PackageInfo: $PACKAGE_PATH" >&2
  exit 1
fi

PACKAGE_INFO="$TMP_DIR/PackageInfo"
if [ ! -r "$PACKAGE_INFO" ]; then
  echo "PackageInfo not readable: $PACKAGE_PATH" >&2
  exit 1
fi

if ! /usr/bin/xmllint --noout "$PACKAGE_INFO" >/dev/null 2>&1; then
  echo "PackageInfo is not readable XML: $PACKAGE_PATH" >&2
  exit 1
fi

if ! /usr/bin/grep -q '^<pkg-info ' "$PACKAGE_INFO"; then
  echo "PackageInfo missing expected pkg-info metadata: $PACKAGE_PATH" >&2
  exit 1
fi

INSTALL_LOCATION="$(
  /usr/bin/xmllint --xpath 'string(/pkg-info/@install-location)' "$PACKAGE_INFO" 2>/dev/null || true
)"

if [ "$INSTALL_LOCATION" != "/" ]; then
  echo "PackageInfo has unexpected install-location: ${INSTALL_LOCATION:-<missing>} (expected /)" >&2
  exit 1
fi

if /usr/bin/grep -q "<relocate" "$PACKAGE_INFO"; then
  echo "PackageInfo contains relocate metadata; install must be non-relocatable." >&2
  exit 1
fi

if ! /usr/bin/xar -xf "$PACKAGE_PATH" -C "$TMP_DIR" Scripts; then
  echo "Unable to extract package scripts payload: $PACKAGE_PATH" >&2
  exit 1
fi

if [ ! -r "$TMP_DIR/Scripts" ]; then
  echo "Postinstall script is not embedded in package scripts payload." >&2
  exit 1
fi

if ! /usr/bin/gunzip -c "$TMP_DIR/Scripts" | /usr/bin/cpio -it > "$SCRIPT_FILES"; then
  echo "Failed to read package scripts payload: $PACKAGE_PATH" >&2
  exit 1
fi

if ! /usr/bin/grep -qxF "$EXPECTED_POSTINSTALL_PATH" "$SCRIPT_FILES"; then
  echo "postinstall script is not embedded in package scripts payload." >&2
  exit 1
fi

echo "Package validation passed: $PACKAGE_PATH"
