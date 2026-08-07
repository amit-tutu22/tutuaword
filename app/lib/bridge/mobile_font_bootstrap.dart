import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/native_engine.dart';

/// Registers bundled font bytes on mobile hosts that do not scan system fonts.
class MobileFontBootstrap {
  MobileFontBootstrap._();

  static const _assetPath = 'assets/fonts/NotoSans-Regular.ttf';

  /// Families commonly referenced by Word documents and OOXML defaults.
  static const _aliases = [
    'Arial',
    'Calibri',
    'Helvetica',
    'Times New Roman',
    'Noto Sans',
    'Liberation Sans',
  ];

  /// Load bundled fonts and register them with the Rust engine when running on
  /// Android or iOS. Safe to call on desktop (no-op).
  static Future<void> registerBundledFonts() async {
    if (kIsWeb || (!Platform.isAndroid && !Platform.isIOS)) {
      return;
    }
    final engine = NativeEngine.load();
    if (engine == null) {
      return;
    }
    try {
      final data = await rootBundle.load(_assetPath);
      final bytes = data.buffer.asUint8List();
      for (final family in _aliases) {
        if (!engine.registerFont(family, bytes)) {
          debugPrint('MobileFontBootstrap: failed to register $family');
        }
      }
      debugPrint('MobileFontBootstrap: registered ${_aliases.length} font aliases');
    } catch (e) {
      debugPrint('MobileFontBootstrap: skipped ($e)');
    }
  }
}
