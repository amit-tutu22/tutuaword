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
| Web (Chrome) | `app/web/wasm/tw_wasm_bg.wasm` | `tw-wasm` via JS (`TwEngine`) |

## Run

```bash
cd app

flutter run -d macos       # after scripts/build-ffi.sh
flutter run -d android     # after scripts/build-ffi.sh android
flutter run -d ios         # after scripts/build-ffi.sh ios
scripts/build-web.sh            # WASM → app/web/wasm/ (see script --help)
flutter run -d chrome      # WASM inline engine (no tw-ffi)
```

**Web / Chrome:** Build WASM bindings with `scripts/build-web.sh` from the repo root (`scripts/build-web.sh --help` for options; requires `wasm-bindgen` CLI). The app loads `tw-wasm` through `app/web/tw_wasm_loader.js`. Use **Open…** / **Save** via the browser file picker; documents download instead of writing to a local path.

**Android:** Android Studio (SDK + NDK). Use **Open…** in the app to pick `.docx` files (system picker; no broad storage permission on Android 13+).

**iOS:** Xcode + code signing. Static `libtw_ffi.a` is linked into the Runner binary at build time. Minimum iOS **14.0** (`file_picker` and Podfile).

Plugins use **CocoaPods** (SPM disabled in `pubspec.yaml`) so Xcode is not broken by Flutter regenerating `FlutterGeneratedPluginSwiftPackage` at iOS 13.0. After `pub get` / clone:

```bash
scripts/ios-xcode-sync.sh
# or: cd app && flutter pub get && cd ios && pod install
```

Open **`app/ios/Runner.xcworkspace`** (not the bare `.xcodeproj`).

## AI (local vs cloud)

**Review → AI Settings**:

| Mode | Behavior |
|------|----------|
| **Always Local** | App probes `http://127.0.0.1:11434` and runs `ollama serve` if Ollama is installed but not running |
| **Always Cloud** | Uses OpenAI / Gemini API keys (no local server) |
| **Automatic** | Short tasks prefer local (same auto-start); heavy summarize prefers cloud |

Install Ollama once (`brew install ollama` or https://ollama.com), then pick **Always Local**. Pull a model if needed: `ollama pull llama3.2`.

On macOS, Debug/Profile builds turn off App Sandbox so the app can start `ollama serve`. Release builds stay sandboxed and prefer opening **Ollama.app**; if auto-start is blocked, open Ollama from Applications (or run `ollama serve` in Terminal) and retry.

## Mobile fonts

Android, iOS, and web builds use injected fonts (no system font scan). On startup the app registers bundled `NotoSans-Regular.ttf` under common Word family names (`Arial`, `Calibri`, `Helvetica`, etc.). Rebuild the native/WASM engine after changing font registration in Rust.

## Cleanup

```bash
scripts/cleanup.sh            # Flutter outputs + copied libs + scratch files
scripts/cleanup.sh --all      # also cargo target/ (incl. CARGO_TARGET_DIR) and platform caches
scripts/cleanup.sh --deep     # --all plus Xcode / simulator / Cursor sandbox caches
scripts/cleanup.sh --dry-run  # print paths without deleting
```

More detail: [docs/architecture/ffi-bridge.md](../docs/architecture/ffi-bridge.md)
