import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/wasm_engine_bootstrap_web.dart';
import 'package:tutuaword/bridge/wasm_engine_web.dart';

/// Registers bundled font bytes for the browser WASM engine.
class MobileFontBootstrap {
  MobileFontBootstrap._();

  static const _assetPath = 'assets/fonts/NotoSans-Regular.ttf';

  static const _aliases = [
    'Arial',
    'Calibri',
    'Helvetica',
    'Times New Roman',
    'Noto Sans',
    'Liberation Sans',
  ];

  static Future<void> registerBundledFonts() async {
    final engine = WasmEngineBootstrap.cachedEngine;
    if (engine == null) {
      return;
    }
    try {
      final data = await rootBundle.load(_assetPath);
      final bytes = data.buffer.asUint8List();
      for (final family in _aliases) {
        engine.registerFont(family, bytes);
      }
    } catch (_) {}
  }
}
