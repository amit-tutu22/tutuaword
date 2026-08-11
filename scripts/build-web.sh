#!/usr/bin/env bash
# Build tw-wasm for Flutter web (Chrome) and emit wasm-bindgen browser bindings.
#
# Usage:
#   scripts/build-web.sh                    # release → app/web/wasm/
#   scripts/build-web.sh --debug            # dev profile (faster compile, larger wasm)
#   scripts/build-web.sh --skip-cargo       # wasm-bindgen only (cargo build already done)
#   scripts/build-web.sh --out-dir PATH     # custom output directory
#   scripts/build-web.sh --help
#
# After building:
#   cd app && flutter run -d chrome
#
# Requires: rustup target wasm32-unknown-unknown, wasm-bindgen-cli
set -euo pipefail
cd "$(dirname "$0")/.."

PROFILE=release
SKIP_CARGO=0
OUT_DIR="app/web/wasm"

usage() {
  cat <<'EOF'
Build tw-wasm for Flutter web and write browser bindings for app/web/.

Usage:
  scripts/build-web.sh [options]

Options:
  --debug           Build with cargo dev profile (faster, larger .wasm)
  --skip-cargo      Skip cargo build; only run wasm-bindgen on existing artifact
  --out-dir PATH    Output directory (default: app/web/wasm)
  -h, --help        Show this help and exit

Output:
  OUT_DIR/tw_wasm.js          ES module loaded by app/web/tw_wasm_loader.js
  OUT_DIR/tw_wasm_bg.wasm     tw-wasm binary (inline document engine)

Run the app:
  cd app && flutter run -d chrome

Prerequisites:
  rustup target add wasm32-unknown-unknown
  cargo install wasm-bindgen-cli
  (match wasm-bindgen-cli to the wasm-bindgen crate version in the workspace)

This script is also invoked by scripts/build-all.sh when wasm is not skipped.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --debug) PROFILE=dev; shift ;;
    --skip-cargo) SKIP_CARGO=1; shift ;;
    --out-dir)
      if [[ $# -lt 2 ]]; then
        echo "error: --out-dir requires a path argument" >&2
        exit 1
      fi
      OUT_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --*)
      echo "unknown option: $1 (try --help)" >&2
      exit 1
      ;;
    *)
      echo "unexpected argument: $1 (try --help)" >&2
      exit 1
      ;;
  esac
done

step() {
  echo ""
  echo "==> $1"
}

resolve_target_dir() {
  local dir
  dir=$(cargo metadata --format-version 1 --no-deps \
    | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
  if [[ -z "$dir" ]]; then
    echo "could not resolve cargo target_directory" >&2
    exit 1
  fi
  TARGET_DIR="$dir"
}

ensure_rust_target() {
  local triple="wasm32-unknown-unknown"
  if ! rustup target list --installed | grep -qx "$triple"; then
    step "Installing rust target $triple"
    rustup target add "$triple"
  fi
}

wasm_bindgen_install_hint() {
  local version
  version=$(cargo metadata --format-version 1 --no-deps 2>/dev/null \
    | sed -n 's/.*"name":"wasm-bindgen","version":"\([^"]*\)".*/\1/p' | head -1)
  echo "wasm-bindgen CLI not found. Install with:" >&2
  if [[ -n "$version" ]]; then
    echo "  cargo install wasm-bindgen-cli --version $version" >&2
  else
    echo "  cargo install wasm-bindgen-cli" >&2
  fi
}

ensure_wasm_bindgen() {
  if ! command -v wasm-bindgen >/dev/null 2>&1; then
    wasm_bindgen_install_hint
    exit 1
  fi
}

resolve_target_dir
WASM="$TARGET_DIR/wasm32-unknown-unknown/$PROFILE/tw_wasm.wasm"

if [[ "$SKIP_CARGO" -eq 0 ]]; then
  ensure_rust_target
  step "cargo build tw-wasm ($PROFILE, wasm32, wasm-bindgen feature)"
  cargo build -p tw-wasm --target wasm32-unknown-unknown --"$PROFILE" --features wasm-bindgen
else
  step "Skipping cargo build (--skip-cargo)"
  if [[ ! -f "$WASM" ]]; then
    echo "error: $WASM not found; run without --skip-cargo or build tw-wasm first" >&2
    exit 1
  fi
fi

if [[ ! -f "$WASM" ]]; then
  echo "error: expected wasm artifact at $WASM" >&2
  exit 1
fi

ensure_wasm_bindgen

step "wasm-bindgen --target web → $OUT_DIR"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
wasm-bindgen "$WASM" --target web --out-dir "$OUT_DIR" --out-name tw_wasm

for required in tw_wasm.js tw_wasm_bg.wasm; do
  if [[ ! -f "$OUT_DIR/$required" ]]; then
    echo "error: wasm-bindgen did not produce $OUT_DIR/$required" >&2
    exit 1
  fi
done

wasm_size=$(wc -c < "$OUT_DIR/tw_wasm_bg.wasm" | tr -d ' ')
echo ""
echo "WASM web bindings written to $OUT_DIR/"
echo "  tw_wasm.js, tw_wasm_bg.wasm (${wasm_size} bytes)"
echo ""
echo "Next: cd app && flutter run -d chrome"
