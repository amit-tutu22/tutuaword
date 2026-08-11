import 'dart:io';

import 'package:tutuaword/bridge/mobile_font_bootstrap.dart';
import 'package:tutuaword/bridge/native_engine.dart';

/// Loads the native FFI engine and bundled fonts on VM targets.
///
/// Throws [StateError] when the native library cannot be loaded so callers
/// (e.g. [AppBootstrap]) do not mount a disconnected editor/ribbon.
Future<void> warmDocumentEngine() async {
  if (Platform.isAndroid || Platform.isIOS) {
    await MobileFontBootstrap.registerBundledFonts();
  } else {
    if (NativeEngine.load() == null) {
      throw StateError(_missingEngineMessage());
    }
    final ready = await NativeEngine.ensureStartupReady();
    if (!ready) {
      throw StateError(
        'Native engine loaded but startup layout failed. '
        'Check the FFI build and relaunch.',
      );
    }
  }

  if (NativeEngine.load() == null) {
    throw StateError(_missingEngineMessage());
  }
}

String _missingEngineMessage() {
  if (Platform.isMacOS) {
    return 'Native engine failed to load (libtw_ffi.dylib). '
        'Rebuild with scripts/build_ffi.sh (or cargo build -p tw-ffi --release) '
        'and relaunch the app.';
  }
  if (Platform.isWindows) {
    return 'Native engine failed to load (tw_ffi.dll). '
        'Rebuild the FFI library and relaunch the app.';
  }
  if (Platform.isLinux) {
    return 'Native engine failed to load (libtw_ffi.so). '
        'Rebuild the FFI library and relaunch the app.';
  }
  return 'Native engine failed to load. Rebuild libtw_ffi for this platform '
      'and relaunch the app.';
}
