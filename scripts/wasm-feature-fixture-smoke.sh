#!/usr/bin/env bash
# Open the Word-compatible feature fixture through the wasm engine.
set -euo pipefail
cd "$(dirname "$0")/.."

TARGET_DIR=$(cargo metadata --format-version 1 --no-deps \
  | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
OUT_DIR="$TARGET_DIR/wasm-smoke"
WASM="$TARGET_DIR/wasm32-unknown-unknown/release/tw_wasm.wasm"

if [[ ! -f "$OUT_DIR/tw_wasm.js" ]]; then
  NEED_BINDGEN=1
else
  NEED_BINDGEN=0
fi

cargo build -p tw-wasm --target wasm32-unknown-unknown --release --features wasm-bindgen
if [[ "$NEED_BINDGEN" -eq 1 ]] || [[ "$WASM" -nt "$OUT_DIR/tw_wasm.js" ]]; then
  if ! command -v wasm-bindgen >/dev/null 2>&1; then
    echo "wasm-bindgen CLI not found" >&2
    exit 1
  fi
  rm -rf "$OUT_DIR"
  wasm-bindgen "$WASM" --target nodejs --out-dir "$OUT_DIR"
fi

TW_WASM_PKG="$OUT_DIR" node crates/tw-wasm/js/feature_fixture_smoke.mjs
