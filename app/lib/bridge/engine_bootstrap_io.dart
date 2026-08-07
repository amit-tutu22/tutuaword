import 'dart:io';

import 'package:tutuaword/bridge/mobile_font_bootstrap.dart';
import 'package:tutuaword/bridge/native_engine.dart';

/// Loads the native FFI engine and bundled fonts on VM targets.
Future<void> warmDocumentEngine() async {
  if (Platform.isAndroid || Platform.isIOS) {
    await MobileFontBootstrap.registerBundledFonts();
  } else {
    NativeEngine.load();
    await NativeEngine.ensureStartupReady();
  }
}
