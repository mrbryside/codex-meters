#!/bin/sh
set -eu

EXPECTED_BUNDLE_PATH="./Applications/Codex Meters.app/Contents/MacOS/codex-token-meter"
FORBIDDEN_BUNDLE_PATH="^\\./Codex Meters\\.app(/|$)"

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
EXPANDED="$TMP_DIR/expanded"
PAYLOAD_FILES="$TMP_DIR/payload-files.txt"
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

if ! /usr/sbin/pkgutil --expand "$PACKAGE_PATH" "$EXPANDED"; then
  echo "Unable to expand package metadata: $PACKAGE_PATH" >&2
  exit 1
fi

PACKAGE_INFO="$EXPANDED/PackageInfo"
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

PAYLOAD_PATH="$EXPANDED/Payload"
if [ ! -r "$PAYLOAD_PATH" ]; then
  echo "Payload not readable: $PACKAGE_PATH" >&2
  exit 1
fi

if /usr/bin/gunzip -c "$PAYLOAD_PATH" | /usr/bin/cpio -it > "$PAYLOAD_FILES"; then
  :
else
  echo "Failed to read package payload: $PACKAGE_PATH" >&2
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

POSTINSTALL_PATH="$EXPANDED/Scripts/postinstall"
if [ ! -x "$POSTINSTALL_PATH" ]; then
  echo "postinstall script is not embedded in package scripts payload." >&2
  exit 1
fi

echo "Package validation passed: $PACKAGE_PATH"
