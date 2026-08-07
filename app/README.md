# tutuaword (Flutter app)

Shell for the Rust document engine (`tw-ffi`).

## Build the native engine

From the repository root:

```bash
scripts/build-all.sh            # workspace + host + mobile (if NDK/Xcode) + wasm
scripts/build-ffi.sh              # desktop only
scripts/build-ffi.sh android        # Android → jniLibs/ (needs Android NDK)
scripts/build-ffi.sh ios            # iOS → tw_ffi.xcframework (macOS + Xcode only)
```

`build-all.sh` options: `--skip-mobile`, `--skip-wasm`, `--with-smoke`, `--with-flutter`, `--no-verify`.

Native libraries are gitignored — run the matching script after pulling Rust changes.

| Platform | Output | Dart loading |
|----------|--------|--------------|
| macOS | `libtw_ffi.dylib` | `DynamicLibrary.open` |
| Linux | `libtw_ffi.so` | `DynamicLibrary.open` |
| Windows | `tw_ffi.dll` | `DynamicLibrary.open` |
| Android | `jniLibs/*/libtw_ffi.so` | `DynamicLibrary.open('libtw_ffi.so')` |
| iOS | `tw_ffi.xcframework` (static) | `DynamicLibrary.process()` |

## Run

```bash
cd app

flutter run -d macos       # after scripts/build-ffi.sh
flutter run -d android     # after scripts/build-ffi.sh android
flutter run -d ios         # after scripts/build-ffi.sh ios
```

**Android:** Android Studio (SDK + NDK). Use **Open…** in the app to pick `.docx` files (system picker; no broad storage permission on Android 13+).

**iOS:** Xcode + code signing. Static `libtw_ffi.a` is linked into the Runner binary at build time.

## Mobile fonts

Android and iOS builds use injected fonts (no system font scan). On startup the app registers bundled `NotoSans-Regular.ttf` under common Word family names (`Arial`, `Calibri`, `Helvetica`, etc.). Rebuild the FFI library after changing font registration in Rust.

## Cleanup

```bash
scripts/cleanup.sh          # Flutter outputs + copied libs
scripts/cleanup.sh --all      # also cargo target/ and platform caches
```

More detail: [docs/architecture/ffi-bridge.md](../docs/architecture/ffi-bridge.md)
