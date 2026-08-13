#!/usr/bin/env bash
# Build tw-ffi for the host, Android (JNI), or iOS (static XCFramework).
#
# Usage:
#   scripts/build-ffi.sh          # host: .dylib / .so / .dll
#   scripts/build-ffi.sh android  # arm64-v8a, armeabi-v7a, x86_64 → jniLibs/
#   scripts/build-ffi.sh ios      # libtw_ffi.a → ios/Frameworks/tw_ffi.xcframework
set -euo pipefail
cd "$(dirname "$0")/.."

# Xcode Run Script phases use a minimal PATH (no rustup). Ensure cargo is found
# when this script is invoked from "Build Rust FFI" / CI shells.
ensure_cargo_on_path() {
  if command -v cargo >/dev/null 2>&1; then
    return 0
  fi
  # shellcheck disable=SC1090,SC1091
  if [[ -f "$HOME/.cargo/env" ]]; then
    . "$HOME/.cargo/env"
  fi
  local candidates=(
    "$HOME/.cargo/bin"
    /opt/homebrew/bin
    /usr/local/bin
  )
  local dir
  for dir in "${candidates[@]}"; do
    if [[ -x "$dir/cargo" ]]; then
      export PATH="$dir:$PATH"
      break
    fi
  done
  if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found (Xcode PATH has no Rust toolchain)." >&2
    echo "Install rustup (https://rustup.rs) or ensure ~/.cargo/bin is on PATH." >&2
    exit 1
  fi
}

ensure_cargo_on_path

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

copy_host_lib() {
  local lib="$1"
  mkdir -p app/macos/Runner/Frameworks
  cp "$lib" app/macos/Runner/Frameworks/
  cp "$lib" app/macos/Runner/ 2>/dev/null || cp "$lib" app/
  cp "$lib" app/ 2>/dev/null || true
  echo "Built $lib (copied to app/macos/Runner/Frameworks/ and app/)"
}

resolve_ndk_home() {
  if [[ -n "${ANDROID_NDK_HOME:-}" && -d "$ANDROID_NDK_HOME" ]]; then
    return 0
  fi
  local sdk="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-}}"
  if [[ -n "$sdk" && -d "$sdk/ndk" ]]; then
    local latest
    latest=$(ls -1 "$sdk/ndk" 2>/dev/null | sort -V | tail -1)
    if [[ -n "$latest" ]]; then
      ANDROID_NDK_HOME="$sdk/ndk/$latest"
      export ANDROID_NDK_HOME
      return 0
    fi
  fi
  local mac_sdk="$HOME/Library/Android/sdk/ndk"
  if [[ -d "$mac_sdk" ]]; then
    local latest
    latest=$(ls -1 "$mac_sdk" 2>/dev/null | sort -V | tail -1)
    if [[ -n "$latest" ]]; then
      ANDROID_NDK_HOME="$mac_sdk/$latest"
      export ANDROID_NDK_HOME
      return 0
    fi
  fi
  echo "Android NDK not found. Set ANDROID_NDK_HOME or install via Android Studio." >&2
  exit 1
}

ndk_prebuilt_host() {
  case "$(uname -s)" in
    Darwin) echo "darwin-x86_64" ;;
    Linux) echo "linux-x86_64" ;;
    MINGW*|MSYS*|CYGWIN*) echo "windows-x86_64" ;;
    *) echo "unknown" ;;
  esac
}

ensure_rust_target() {
  local triple="$1"
  if ! rustup target list --installed | grep -qx "$triple"; then
    echo "installing rust target $triple"
    rustup target add "$triple"
  fi
}

build_android() {
  resolve_ndk_home
  local host prebuilt api
  host=$(ndk_prebuilt_host)
  if [[ "$host" == "unknown" ]]; then
    echo "unsupported host for Android cross-compile: $(uname -s)" >&2
    exit 1
  fi
  prebuilt="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$host/bin"
  if [[ ! -d "$prebuilt" ]]; then
    echo "NDK toolchain not found: $prebuilt" >&2
    exit 1
  fi
  api="${TW_ANDROID_API_LEVEL:-24}"

  local rows=(
    "aarch64-linux-android:arm64-v8a:aarch64-linux-android"
    "armv7-linux-androideabi:armeabi-v7a:armv7a-linux-androideabi"
    "x86_64-linux-android:x86_64:x86_64-linux-android"
  )

  resolve_target_dir
  local jni_root="app/android/app/src/main/jniLibs"
  local built=0

  for row in "${rows[@]}"; do
    IFS=: read -r triple abi clang_base <<<"$row"
    ensure_rust_target "$triple"
    local clang="$prebuilt/${clang_base}${api}-clang"
    if [[ ! -x "$clang" ]]; then
      echo "missing NDK clang: $clang" >&2
      exit 1
    fi
    export "CC_${triple//-/_}=$clang"
    export "AR_${triple//-/_}=$prebuilt/llvm-ar"
    local triple_upper
    triple_upper=$(echo "$triple" | tr '[:lower:]-' '[:upper:]_')
    export "CARGO_TARGET_${triple_upper}_LINKER=$clang"

    echo "building tw-ffi for $triple ($abi)"
    cargo build -p tw-ffi --release --target "$triple"

    local lib="$TARGET_DIR/$triple/release/libtw_ffi.so"
    if [[ ! -f "$lib" ]]; then
      echo "missing $lib after build" >&2
      exit 1
    fi
    mkdir -p "$jni_root/$abi"
    cp "$lib" "$jni_root/$abi/libtw_ffi.so"
    echo "copied $lib → $jni_root/$abi/libtw_ffi.so"
    built=$((built + 1))
  done

  echo "Android FFI: built $built ABIs under $jni_root"
}

build_ios() {
  if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "iOS FFI must be built on macOS (Xcode SDK required)" >&2
    exit 1
  fi
  if ! command -v xcodebuild >/dev/null 2>&1; then
    echo "xcodebuild not found; install Xcode" >&2
    exit 1
  fi

  export IPHONEOS_DEPLOYMENT_TARGET="${TW_IOS_DEPLOYMENT_TARGET:-13.0}"

  local device_triple="aarch64-apple-ios"
  local sim_triples=("aarch64-apple-ios-sim")
  if [[ "${TW_IOS_INTEL_SIM:-1}" == "1" ]]; then
    sim_triples+=("x86_64-apple-ios")
  fi

  ensure_rust_target "$device_triple"
  for triple in "${sim_triples[@]}"; do
    ensure_rust_target "$triple"
  done

  resolve_target_dir

  echo "building tw-ffi for $device_triple (device)"
  cargo build -p tw-ffi --release --target "$device_triple"

  local device_lib="$TARGET_DIR/$device_triple/release/libtw_ffi.a"
  if [[ ! -f "$device_lib" ]]; then
    echo "missing $device_lib after build" >&2
    exit 1
  fi

  local sim_libs=()
  for triple in "${sim_triples[@]}"; do
    echo "building tw-ffi for $triple (simulator)"
    cargo build -p tw-ffi --release --target "$triple"
    local lib="$TARGET_DIR/$triple/release/libtw_ffi.a"
    if [[ ! -f "$lib" ]]; then
      echo "missing $lib after build" >&2
      exit 1
    fi
    sim_libs+=("$lib")
  done

  local sim_universal="$TARGET_DIR/ios-sim-universal/libtw_ffi.a"
  mkdir -p "$(dirname "$sim_universal")"
  if [[ "${#sim_libs[@]}" -eq 1 ]]; then
    cp "${sim_libs[0]}" "$sim_universal"
  else
    echo "lipo simulator slices: ${sim_libs[*]}"
    lipo -create "${sim_libs[@]}" -output "$sim_universal"
  fi

  local hdr_root="app/ios/tw_ffi/include"
  local xcframework="app/ios/Frameworks/tw_ffi.xcframework"
  mkdir -p "$hdr_root"
  if [[ ! -f "$hdr_root/tw_ffi.h" ]]; then
    echo '/* tw-ffi static library */' >"$hdr_root/tw_ffi.h"
  fi

  rm -rf "$xcframework"
  xcodebuild -create-xcframework \
    -library "$device_lib" -headers "$hdr_root" \
    -library "$sim_universal" -headers "$hdr_root" \
    -output "$xcframework"

  local sim_slice
  sim_slice=$(find "$xcframework" -maxdepth 1 -type d -name 'ios-*-simulator' | head -1)
  sim_slice=${sim_slice##*/}
  if [[ -z "$sim_slice" ]]; then
    echo "could not detect simulator slice in $xcframework" >&2
    exit 1
  fi

  cat >app/ios/Flutter/tw_ffi_generated.xcconfig <<EOF
TW_FFI_SIM_SLICE=$sim_slice
EOF
  echo "wrote app/ios/Flutter/tw_ffi_generated.xcconfig (TW_FFI_SIM_SLICE=$sim_slice)"
  echo "iOS FFI: $xcframework"
}

build_host() {
  cargo build -p tw-ffi --release
  resolve_target_dir

  for name in libtw_ffi.dylib libtw_ffi.so tw_ffi.dll; do
    local lib="$TARGET_DIR/release/$name"
    if [[ -f "$lib" ]]; then
      copy_host_lib "$lib"
      exit 0
    fi
  done

  echo "no FFI library found under $TARGET_DIR/release" >&2
  exit 1
}

case "${1:-}" in
  android) build_android ;;
  ios) build_ios ;;
  "")
    build_host
    ;;
  *)
    echo "usage: $0 [android|ios]" >&2
    exit 1
    ;;
esac
