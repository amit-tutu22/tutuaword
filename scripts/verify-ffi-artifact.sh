#!/usr/bin/env bash
# R3.2: confirm the native FFI artifact for this runner was built.
set -euo pipefail
cd "$(dirname "$0")/.."

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
