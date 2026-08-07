#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -p tw-ffi --release

# Resolve the build directory from cargo rather than assuming ./target: when
# CARGO_TARGET_DIR is set (CI caches, sandboxes, a global cargo config) the
# hardcoded path holds a stale library from some earlier build, and copying it
# ships an engine missing whatever exports were just added.
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps \
  | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
if [[ -z "$TARGET_DIR" ]]; then
  echo "could not resolve cargo target_directory" >&2
  exit 1
fi

copy_lib() {
  local lib="$1"
  mkdir -p app/macos/Runner/Frameworks
  cp "$lib" app/macos/Runner/Frameworks/
  cp "$lib" app/macos/Runner/ 2>/dev/null || cp "$lib" app/
  cp "$lib" app/ 2>/dev/null || true
  echo "Built $lib (copied to app/macos/Runner/Frameworks/ and app/)"
}

for name in libtw_ffi.dylib libtw_ffi.so; do
  LIB="$TARGET_DIR/release/$name"
  if [[ -f "$LIB" ]]; then
    copy_lib "$LIB"
    exit 0
  fi
done

echo "no FFI library found under $TARGET_DIR/release" >&2
exit 1
