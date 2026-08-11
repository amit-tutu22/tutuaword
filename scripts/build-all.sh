#!/usr/bin/env bash
# Build the Rust workspace and every tw-ffi / tw-wasm target available on this host.
#
# Usage:
#   scripts/build-all.sh                 # host + mobile (if tools exist) + wasm
#   scripts/build-all.sh --skip-mobile     # skip Android / iOS FFI
#   scripts/build-all.sh --skip-wasm       # skip wasm32 build
#   scripts/build-all.sh --with-smoke     # also run scripts/wasm-smoke.sh
#   scripts/build-all.sh --with-flutter    # flutter build for host desktop/mobile
#   scripts/build-all.sh --no-verify       # skip verify-ffi-artifact.sh
#   scripts/build-all.sh --help
set -euo pipefail
cd "$(dirname "$0")/.."

SKIP_HOST=0
SKIP_MOBILE=0
SKIP_WASM=0
WITH_SMOKE=0
WITH_FLUTTER=0
VERIFY=1

usage() {
  sed -n '3,11p' "$0" | sed 's/^# \{0,1\}//'
}

for arg in "$@"; do
  case "$arg" in
    --skip-host) SKIP_HOST=1 ;;
    --skip-mobile) SKIP_MOBILE=1 ;;
    --skip-wasm) SKIP_WASM=1 ;;
    --with-smoke) WITH_SMOKE=1 ;;
    --with-flutter) WITH_FLUTTER=1 ;;
    --no-verify) VERIFY=0 ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $arg (try --help)" >&2
      exit 1
      ;;
  esac
done

step() {
  echo ""
  echo "==> $1"
}

have_android_ndk() {
  if [[ -n "${ANDROID_NDK_HOME:-}" && -d "$ANDROID_NDK_HOME" ]]; then
    return 0
  fi
  local sdk="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-}}"
  if [[ -n "$sdk" && -d "$sdk/ndk" ]]; then
    return 0
  fi
  if [[ -d "$HOME/Library/Android/sdk/ndk" ]]; then
    return 0
  fi
  return 1
}

have_ios_tools() {
  [[ "$(uname -s)" == "Darwin" ]] && command -v xcodebuild >/dev/null 2>&1
}

have_flutter() {
  command -v flutter >/dev/null 2>&1
}

step "Rust workspace (release)"
cargo build --workspace --release

if [[ "$SKIP_HOST" -eq 0 ]]; then
  step "Host FFI ($(uname -s))"
  bash scripts/build-ffi.sh
  if [[ "$VERIFY" -eq 1 ]]; then
    bash scripts/verify-ffi-artifact.sh
  fi
fi

if [[ "$SKIP_MOBILE" -eq 0 ]]; then
  if have_android_ndk; then
    step "Android FFI (jniLibs)"
    bash scripts/build-ffi.sh android
    if [[ "$VERIFY" -eq 1 ]]; then
      bash scripts/verify-ffi-artifact.sh android
    fi
  else
    echo "skip Android FFI (NDK not found — set ANDROID_NDK_HOME or install via Android Studio)"
  fi

  if have_ios_tools; then
    step "iOS FFI (tw_ffi.xcframework)"
    bash scripts/build-ffi.sh ios
    if [[ "$VERIFY" -eq 1 ]]; then
      bash scripts/verify-ffi-artifact.sh ios
    fi
  else
    echo "skip iOS FFI (requires macOS + Xcode)"
  fi
fi

if [[ "$SKIP_WASM" -eq 0 ]]; then
  step "WASM (tw-wasm wasm32 release)"
  if ! rustup target list --installed | grep -qx "wasm32-unknown-unknown"; then
    echo "installing rust target wasm32-unknown-unknown"
    rustup target add wasm32-unknown-unknown
  fi
  cargo build -p tw-wasm --target wasm32-unknown-unknown --release --features wasm-bindgen
  cargo check -p tw-wasm --target wasm32-unknown-unknown
  step "WASM web bindings (app/web/wasm)"
  bash scripts/build-web.sh

  if [[ "$WITH_SMOKE" -eq 1 ]]; then
    step "WASM smoke test"
    bash scripts/wasm-smoke.sh
  fi
fi

if [[ "$WITH_FLUTTER" -eq 1 ]]; then
  if ! have_flutter; then
    echo "skip Flutter builds (flutter not in PATH)" >&2
  else
    step "Flutter pub get"
    (cd app && flutter pub get)

    case "$(uname -s)" in
      Darwin)
        step "Flutter build macOS"
        (cd app && flutter build macos --no-pub)
        step "Flutter build web"
        (cd app && flutter build web --no-pub)
        if have_ios_tools; then
          step "Flutter build iOS (simulator, no codesign)"
          (cd app && flutter build ios --simulator --no-codesign --no-pub)
        fi
        if have_android_ndk; then
          step "Flutter build Android APK"
          (cd app && flutter build apk --no-pub)
        fi
        ;;
      Linux)
        step "Flutter build Linux"
        (cd app && flutter build linux --no-pub)
        if have_android_ndk; then
          step "Flutter build Android APK"
          (cd app && flutter build apk --no-pub)
        fi
        ;;
      MINGW*|MSYS*|CYGWIN*)
        step "Flutter build Windows"
        (cd app && flutter build windows --no-pub)
        ;;
      *)
        echo "skip Flutter platform builds (unknown host OS)"
        ;;
    esac
  fi
fi

echo ""
echo "build-all: done"
