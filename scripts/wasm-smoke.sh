#!/usr/bin/env bash
# R3.1 exit gate: build tw-wasm for the browser target and open a real document
# through it from JavaScript. Compilation alone proves little here — threads,
# sleeps and the system font scan all compile for wasm32 and only fail once
# something actually runs — so this harness is what verifies the web path.
set -euo pipefail
cd "$(dirname "$0")/.."

TARGET_DIR=$(cargo metadata --format-version 1 --no-deps \
  | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
if [[ -z "$TARGET_DIR" ]]; then
  echo "could not resolve cargo target_directory" >&2
  exit 1
fi

OUT_DIR="$TARGET_DIR/wasm-smoke"
WASM="$TARGET_DIR/wasm32-unknown-unknown/release/tw_wasm.wasm"

cargo build -p tw-wasm --target wasm32-unknown-unknown --release --features wasm-bindgen

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen CLI not found; install with:" >&2
  echo "  cargo install wasm-bindgen-cli --version \$(cargo metadata --format-version 1 | sed -n 's/.*\"name\":\"wasm-bindgen\",\"version\":\"\([^\"]*\)\".*/\1/p' | head -1)" >&2
  exit 1
fi

rm -rf "$OUT_DIR"
wasm-bindgen "$WASM" --target nodejs --out-dir "$OUT_DIR"

TW_WASM_PKG="$OUT_DIR" node crates/tw-wasm/js/smoke.mjs
