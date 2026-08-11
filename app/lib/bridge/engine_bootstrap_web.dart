import 'package:tutuaword/bridge/font_bootstrap.dart';
import 'package:tutuaword/bridge/wasm_engine_bootstrap_web.dart';

/// Loads the browser WASM engine and bundled fonts.
///
/// Throws [StateError] when WASM init fails so the UI does not mount a
/// disconnected editor/ribbon.
Future<void> warmDocumentEngine() async {
  await WasmEngineBootstrap.initialize();
  await MobileFontBootstrap.registerBundledFonts();
  if (WasmEngineBootstrap.cachedEngine == null) {
    throw StateError(
      'WASM document engine failed to load. '
      'Rebuild app/web/wasm (tw-wasm) and hard-refresh the browser.',
    );
  }
  // Relayout the startup document now that bundled fonts are registered.
  WasmEngineBootstrap.cachedEngine!.newDocument();
}
