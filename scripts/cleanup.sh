#!/usr/bin/env bash
# Remove local build artifacts and copied FFI libraries to reclaim disk space.
#
# Usage:
#   scripts/cleanup.sh            # Flutter outputs + copied native libs (fast)
#   scripts/cleanup.sh --all        # also cargo target/, Gradle, ephemeral Flutter glue
#   scripts/cleanup.sh --dry-run    # print what would be removed
set -euo pipefail
cd "$(dirname "$0")/.."

DRY_RUN=0
ALL=0

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --all) ALL=1 ;;
    -h|--help)
      sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "unknown option: $arg (try --help)" >&2
      exit 1
      ;;
  esac
done

bytes_before=0
if command -v du >/dev/null 2>&1; then
  bytes_before=$(du -sk . 2>/dev/null | awk '{print $1}')
fi

removed=0

rm_path() {
  local path="$1"
  if [[ -e "$path" ]]; then
    if [[ "$DRY_RUN" -eq 1 ]]; then
      echo "would remove: $path"
    else
      rm -rf "$path"
      echo "removed: $path"
    fi
    removed=$((removed + 1))
  fi
}

rm_glob() {
  local pattern="$1"
  local match
  shopt -s nullglob
  for match in $pattern; do
    rm_path "$match"
  done
  shopt -u nullglob
}

# Copied FFI artifacts (rebuild with scripts/build-ffi.sh)
rm_glob "app/libtw_ffi.*"
rm_glob "app/macos/Runner/libtw_ffi.*"
rm_glob "app/macos/Runner/Frameworks/libtw_ffi.*"
rm_glob "app/android/app/src/main/jniLibs/*/libtw_ffi.so"
rm_path "app/ios/Frameworks/tw_ffi.xcframework"
rm_path "app/ios/Flutter/tw_ffi_generated.xcconfig"

# Flutter
rm_path "app/.dart_tool"
rm_path "app/build"
rm_path "app/.flutter-plugins"
rm_path "app/.flutter-plugins-dependencies"

if [[ "$ALL" -eq 1 ]]; then
  # Rust (largest footprint — all cross-compile targets)
  rm_path "target"

  # Flutter platform glue regenerated on next build
  rm_path "app/macos/Flutter/ephemeral"
  rm_path "app/linux/flutter/ephemeral"
  rm_path "app/windows/flutter/ephemeral"
  rm_path "app/ios/Flutter/ephemeral"

  # Android Gradle / native build cache
  rm_path "app/android/.gradle"
  rm_path "app/android/.cxx"
  rm_path "app/android/app/.cxx"
  rm_path "app/android/build"
  rm_path "app/android/app/build"
  rm_path "app/android/local.properties"

  # wasm-smoke harness output under cargo target (if target already gone, skip)
  TARGET_DIR=$(cargo metadata --format-version 1 --no-deps 2>/dev/null \
    | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p' || true)
  if [[ -n "$TARGET_DIR" && "$TARGET_DIR" != "target" ]]; then
    rm_path "$TARGET_DIR/wasm-smoke"
  fi

  rm_glob "**/mutants.out*"
fi

if [[ "$removed" -eq 0 ]]; then
  echo "nothing to clean"
else
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "dry run: $removed path(s) would be removed"
  else
    echo "cleaned $removed path(s)"
    if command -v du >/dev/null 2>&1 && [[ "$bytes_before" -gt 0 ]]; then
      bytes_after=$(du -sk . 2>/dev/null | awk '{print $1}')
      freed=$((bytes_before - bytes_after))
      if [[ "$freed" -gt 0 ]]; then
        echo "freed ~$((freed / 1024)) MB"
      fi
    fi
    if [[ "$ALL" -eq 0 ]]; then
      echo "tip: scripts/cleanup.sh --all removes cargo target/ and Gradle caches"
    fi
  fi
fi
