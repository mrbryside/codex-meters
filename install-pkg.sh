#!/bin/sh
set -eu

RELEASE_TAG="${CODEX_METERS_RELEASE_TAG:-v0.1.0}"
PKG_URL="${1:-${CODEX_METERS_PKG_URL:-https://github.com/mrbryside/codex-meters/releases/download/${RELEASE_TAG}/Codex%20Meters.pkg}}"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

echo "Downloading Codex Meters..."
curl -fL --progress-bar "$PKG_URL" -o "$tmp_dir/Codex-Meters.pkg"

echo "Installing Codex Meters..."
sudo installer -pkg "$tmp_dir/Codex-Meters.pkg" -target /

echo "Codex Meters installed."
