#!/usr/bin/env bash
# R3.2: confirm the native FFI artifact for this runner was built.
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ "${1:-}" == "ios" ]]; then
  XCFW="app/ios/Frameworks/tw_ffi.xcframework"
  missing=0
  for slice in ios-arm64 ios-arm64-simulator ios-arm64_x86_64-simulator; do
    lib="$XCFW/$slice/libtw_ffi.a"
    if [[ -f "$lib" ]]; then
      echo "R3.2 ok: $lib ($(wc -c <"$lib" | tr -d ' ') bytes)"
    fi
  done
  if [[ ! -f "$XCFW/ios-arm64/libtw_ffi.a" ]]; then
    echo "missing iOS device FFI: $XCFW/ios-arm64/libtw_ffi.a" >&2
    missing=1
  fi
  sim_lib=$(find "$XCFW" -path '*/libtw_ffi.a' ! -path '*/ios-arm64/*' | head -1)
  if [[ -z "$sim_lib" ]]; then
    echo "missing iOS simulator FFI under $XCFW" >&2
    missing=1
  fi
  if [[ "$missing" -ne 0 ]]; then
    exit 1
  fi
  exit 0
fi

if [[ "${1:-}" == "android" ]]; then
  JNI_ROOT="app/android/app/src/main/jniLibs"
  missing=0
  for abi in arm64-v8a armeabi-v7a x86_64; do
    lib="$JNI_ROOT/$abi/libtw_ffi.so"
    if [[ ! -f "$lib" ]]; then
      echo "missing Android FFI: $lib" >&2
      missing=1
    else
      echo "R3.2 ok: $lib ($(wc -c <"$lib" | tr -d ' ') bytes)"
    fi
  done
  if [[ "$missing" -ne 0 ]]; then
    exit 1
  fi
  exit 0
fi

TARGET_DIR=$(cargo metadata --format-version 1 --no-deps \
  | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
if [[ -z "$TARGET_DIR" ]]; then
  echo "could not resolve cargo target_directory" >&2
  exit 1
fi

if [[ -n "${TW_FFI_EXPECT:-}" ]]; then
  EXPECT_NAME="$TW_FFI_EXPECT"
elif [[ "${RUNNER_OS:-}" == "Windows" ]]; then
  EXPECT_NAME="tw_ffi.dll"
elif [[ "$(uname -s)" == "Linux" ]]; then
  EXPECT_NAME="libtw_ffi.so"
elif [[ "$(uname -s)" == "Darwin" ]]; then
  EXPECT_NAME="libtw_ffi.dylib"
else
  echo "cannot determine expected FFI library name (set TW_FFI_EXPECT)" >&2
  exit 1
fi

LIB="$TARGET_DIR/release/$EXPECT_NAME"
if [[ ! -f "$LIB" ]]; then
  echo "missing FFI library: $LIB" >&2
  exit 1
fi

echo "R3.2 ok: $LIB ($(wc -c <"$LIB" | tr -d ' ') bytes)"
