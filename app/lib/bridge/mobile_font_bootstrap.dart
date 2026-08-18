import 'dart:io' if (dart.library.html) 'package:tutuaword/bridge/platform_stub.dart';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/native_engine.dart';

/// Registers bundled font bytes on mobile hosts that do not scan system fonts.
class MobileFontBootstrap {
  MobileFontBootstrap._();

  static const _notoPath = 'assets/fonts/NotoSans-Regular.ttf';

  /// Metric-compatible Office substitutes registered under Word family names.
  static const _officeFaces = <({String path, String family, bool bold, bool italic})>[
    (path: 'assets/fonts/Carlito-Regular.ttf', family: 'Calibri', bold: false, italic: false),
    (path: 'assets/fonts/Carlito-Bold.ttf', family: 'Calibri', bold: true, italic: false),
    (path: 'assets/fonts/LiberationSans-Regular.ttf', family: 'Arial', bold: false, italic: false),
    (path: 'assets/fonts/LiberationSans-Bold.ttf', family: 'Arial', bold: true, italic: false),
    (path: 'assets/fonts/LiberationSans-Regular.ttf', family: 'Helvetica', bold: false, italic: false),
    (path: 'assets/fonts/LiberationSerif-Regular.ttf', family: 'Times New Roman', bold: false, italic: false),
    (path: 'assets/fonts/LiberationSerif-Bold.ttf', family: 'Times New Roman', bold: true, italic: false),
  ];

  /// True after [registerBundledFonts] has finished (engine load + font bytes).
  static bool isReady = false;

  static Future<void> registerBundledFonts() async {
    if (kIsWeb) {
      return;
    }
    if (!Platform.isAndroid && !Platform.isIOS) {
      return;
    }
    await Future<void>.delayed(Duration.zero);
    final engine = NativeEngine.load();
    if (engine == null) {
      debugPrint('MobileFontBootstrap: native engine not available');
      return;
    }
    debugPrint('MobileFontBootstrap: engine loaded');
    try {
      for (final face in _officeFaces) {
        final data = await rootBundle.load(face.path);
        final bytes = data.buffer.asUint8List();
        final label = '${face.family}${face.bold ? ' Bold' : ''}${face.italic ? ' Italic' : ''}';
        if (!engine.registerFont(face.family, bytes, bold: face.bold, italic: face.italic)) {
          debugPrint('MobileFontBootstrap: failed to register $label');
        }
        await Future<void>.delayed(Duration.zero);
      }
      final noto = await rootBundle.load(_notoPath);
      if (!engine.registerFont('Noto Sans', noto.buffer.asUint8List())) {
        debugPrint('MobileFontBootstrap: failed to register Noto Sans fallback');
      }
      debugPrint('MobileFontBootstrap: registered Office substitutes + Noto fallback');
    } catch (e) {
      debugPrint('MobileFontBootstrap: skipped ($e)');
    }
    debugPrint('MobileFontBootstrap: waiting for startup document…');
    await NativeEngine.ensureStartupReady();
    debugPrint('MobileFontBootstrap: startup complete');
    isReady = true;
  }
}
