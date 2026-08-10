import 'dart:io' if (dart.library.html) 'package:tutuaword/bridge/platform_stub.dart';

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
  /// True after [registerBundledFonts] has finished (engine load + font bytes).
  static bool isReady = false;

  static Future<void> registerBundledFonts() async {
    if (kIsWeb) {
      return;
    }
    if (!Platform.isAndroid && !Platform.isIOS) {
      return;
    }
    // Yield so AppBootstrap can paint before tw_init / font registration.
    await Future<void>.delayed(Duration.zero);
    final engine = NativeEngine.load();
    if (engine == null) {
      // Leave isReady false so warmDocumentEngine can fail loudly instead of
      // mounting a disconnected editor/ribbon.
      debugPrint('MobileFontBootstrap: native engine not available');
      return;
    }
    debugPrint('MobileFontBootstrap: engine loaded');
    try {
      final data = await rootBundle.load(_assetPath);
      final bytes = data.buffer.asUint8List();
      for (final family in _aliases) {
        if (!engine.registerFont(family, bytes)) {
          debugPrint('MobileFontBootstrap: failed to register $family');
        }
        // Each face blocks on the worker's reply; yield so frames keep painting.
        await Future<void>.delayed(Duration.zero);
      }
      debugPrint('MobileFontBootstrap: registered ${_aliases.length} font aliases');
    } catch (e) {
      debugPrint('MobileFontBootstrap: skipped ($e)');
    }
    debugPrint('MobileFontBootstrap: waiting for startup document…');
    await NativeEngine.ensureStartupReady();
    debugPrint('MobileFontBootstrap: startup complete');
    isReady = true;
  }
}
