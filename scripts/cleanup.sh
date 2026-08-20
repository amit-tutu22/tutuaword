#!/usr/bin/env bash
# Remove local build artifacts, copied FFI libraries, and optional machine caches.
#
# Usage:
#   scripts/cleanup.sh              # Flutter outputs + copied native libs + scratch (fast)
#   scripts/cleanup.sh --all        # also cargo target/, Gradle, Pods, ephemeral Flutter glue
#   scripts/cleanup.sh --deep       # --all plus Xcode / simulator / Cursor sandbox caches
#   scripts/cleanup.sh --dry-run    # print what would be removed
#   scripts/cleanup.sh --help
#
# Does not delete source, git history, or tracked files (including app/web/wasm).
set -euo pipefail
cd "$(dirname "$0")/.."

DRY_RUN=0
ALL=0
DEEP=0

usage() {
  cat <<'EOF'
Remove local build artifacts, copied FFI libraries, and optional machine caches.

Usage:
  scripts/cleanup.sh              # Flutter outputs + copied native libs + scratch (fast)
  scripts/cleanup.sh --all        # also cargo target/, Gradle, Pods, ephemeral Flutter glue
  scripts/cleanup.sh --deep       # --all plus Xcode / simulator / Cursor sandbox caches
  scripts/cleanup.sh --dry-run    # print what would be removed
  scripts/cleanup.sh --help

Does not delete source, git history, or tracked files (including app/web/wasm).
EOF
}

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --all) ALL=1 ;;
    --deep)
      ALL=1
      DEEP=1
      ;;
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

removed=0

size_of() {
  local path="$1"
  local sz=""
  if command -v du >/dev/null 2>&1 && [[ -e "$path" ]]; then
    sz=$(du -sh "$path" 2>/dev/null | awk '{print $1}' || true)
  fi
  echo "${sz:-?}"
}

rm_path() {
  local path="$1"
  if [[ -e "$path" || -L "$path" ]]; then
    local sz
    sz=$(size_of "$path")
    if [[ "$DRY_RUN" -eq 1 ]]; then
      echo "would remove ($sz): $path"
    else
      rm -rf "$path"
      echo "removed ($sz): $path"
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

resolve_cargo_target_dirs() {
  local dirs=()
  dirs+=("target")
  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    dirs+=("$CARGO_TARGET_DIR")
  fi
  if command -v cargo >/dev/null 2>&1; then
    local meta
    meta=$(cargo metadata --format-version 1 --no-deps 2>/dev/null \
      | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p' || true)
    if [[ -n "$meta" ]]; then
      dirs+=("$meta")
    fi
  fi

  local seen="" dir
  for dir in "${dirs[@]}"; do
    [[ -z "$dir" ]] && continue
    case " $seen " in
      *" $dir "*) continue ;;
    esac
    seen+=" $dir"
    printf '%s\n' "$dir"
  done
}

# --- default: copied FFI + Flutter outputs + scratch -------------------------

rm_glob "app/libtw_ffi.*"
rm_glob "app/macos/Runner/libtw_ffi.*"
rm_glob "app/macos/Runner/Frameworks/libtw_ffi.*"
rm_glob "app/linux/libtw_ffi.*"
rm_glob "app/windows/libtw_ffi.*"
rm_glob "app/android/app/src/main/jniLibs/*/libtw_ffi.so"
rm_path "app/ios/Frameworks/tw_ffi.xcframework"
rm_path "app/ios/Flutter/tw_ffi_generated.xcconfig"

rm_path "app/.dart_tool"
rm_path "app/build"
rm_path "app/coverage"
rm_path "app/.flutter-plugins"
rm_path "app/.flutter-plugins-dependencies"

# Agent / local scratch at the repo root (do not walk target/)
rm_glob ".tmp-*"
if command -v find >/dev/null 2>&1; then
  while IFS= read -r -d '' bk; do
    rm_path "$bk"
  done < <(find crates app \
    \( -path 'app/build' -o -path 'app/.dart_tool' \) -prune \
    -o -name '*.rs.bk' -print0 2>/dev/null || true)
fi

if [[ "$ALL" -eq 1 ]]; then
  # Rust (largest footprint — host target plus any CARGO_TARGET_DIR override)
  while IFS= read -r dir; do
    rm_path "$dir"
  done < <(resolve_cargo_target_dirs)

  # Flutter platform glue regenerated on next build
  rm_path "app/macos/Flutter/ephemeral"
  rm_path "app/linux/flutter/ephemeral"
  rm_path "app/windows/flutter/ephemeral"
  rm_path "app/ios/Flutter/ephemeral"

  # CocoaPods (pod install on next iOS/macOS build)
  rm_path "app/ios/Pods"
  rm_path "app/macos/Pods"

  # Android Gradle / native build cache
  rm_path "app/android/.gradle"
  rm_path "app/android/.cxx"
  rm_path "app/android/app/.cxx"
  rm_path "app/android/build"
  rm_path "app/android/app/build"
  rm_path "app/android/local.properties"

  if command -v find >/dev/null 2>&1; then
    while IFS= read -r -d '' mutant; do
      rm_path "$mutant"
    done < <(find . -path ./target -prune -o -name 'mutants.out*' -print0 2>/dev/null || true)
  fi
fi

if [[ "$DEEP" -eq 1 ]]; then
  # Cursor agent cargo/sandbox cache (this is what filled the disk during gated tests)
  if [[ -n "${TMPDIR:-}" ]]; then
    rm_path "${TMPDIR%/}/cursor-sandbox-cache"
  fi
  rm_path /tmp/cursor-sandbox-cache

  if [[ "$(uname -s)" == "Darwin" ]]; then
    local_derived="$HOME/Library/Developer/Xcode/DerivedData"
    if [[ -d "$local_derived" ]]; then
      shopt -s nullglob
      for derived in "$local_derived"/*; do
        [[ -d "$derived" ]] || continue
        if grep -Fq "/Hypersdk/tutuaword" "$derived/info.plist" 2>/dev/null; then
          rm_path "$derived"
        fi
      done
      shopt -u nullglob
    fi
    rm_path "$HOME/Library/Developer/CoreSimulator/Caches"
  fi
fi

if [[ "$removed" -eq 0 ]]; then
  echo "nothing to clean"
else
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "dry run: $removed path(s) would be removed"
  else
    echo "cleaned $removed path(s)"
  fi
  if [[ "$ALL" -eq 0 ]]; then
    echo "tip: scripts/cleanup.sh --all also removes cargo target/ and Gradle caches"
  elif [[ "$DEEP" -eq 0 ]]; then
    echo "tip: scripts/cleanup.sh --deep also removes Xcode / simulator / sandbox caches"
  fi
fi
