import 'package:tutuaword/bridge/font_bootstrap.dart';
import 'package:tutuaword/bridge/wasm_engine_bootstrap_web.dart';

/// Loads the browser WASM engine and bundled fonts.
Future<void> warmDocumentEngine() async {
  await WasmEngineBootstrap.initialize();
  await MobileFontBootstrap.registerBundledFonts();
  // Relayout the startup document now that bundled fonts are registered.
  WasmEngineBootstrap.cachedEngine?.newDocument();
}
