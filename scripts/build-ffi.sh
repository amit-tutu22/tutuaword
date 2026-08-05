#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -p tw-ffi --release
LIB="target/release/libtw_ffi.dylib"
if [[ -f "$LIB" ]]; then
  mkdir -p app/macos/Runner/Frameworks
  cp "$LIB" app/macos/Runner/Frameworks/
  cp "$LIB" app/macos/Runner/ 2>/dev/null || cp "$LIB" app/
  echo "Built $LIB (copied to app/macos/Runner/Frameworks/)"
else
  LIB="target/release/libtw_ffi.so"
  cp "$LIB" app/ 2>/dev/null || true
  echo "Built FFI library"
fi
