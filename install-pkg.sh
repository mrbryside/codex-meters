#!/bin/sh
set -eu

RELEASE_TAG="${CODEX_METERS_RELEASE_TAG:-v0.1.0}"
ASSET_BASE_URL="${CODEX_METERS_ASSET_BASE_URL:-https://raw.githubusercontent.com/mrbryside/codex-meters/main/releases/${RELEASE_TAG}}"
PKG_URL="${1:-${CODEX_METERS_PKG_URL:-${ASSET_BASE_URL}/Codex%20Meters.pkg}}"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

echo "Downloading Codex Meters..."
curl -fL --progress-bar "$PKG_URL" -o "$tmp_dir/Codex-Meters.pkg"

echo "Installing Codex Meters..."
sudo installer -pkg "$tmp_dir/Codex-Meters.pkg" -target /

echo "Codex Meters installed."
