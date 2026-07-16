#!/bin/sh
set -eu
bun test
bun run build
test -d dist
echo "Frontend smoke checks passed. Rust/Tauri checks require cargo and macOS." 
