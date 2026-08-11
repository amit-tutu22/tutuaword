/// Print scale / margin settings passed to the engine (F25.S2).
enum PrintScaleMode {
  actualSize,
  fitToMargins,
  customPercent,
}

/// What content to send to the print PDF (F25.S3).
enum PrintScope {
  document,
  selection,
}

/// Two-sided printing (F25.S4) — primarily a platform print attribute.
enum PrintDuplexMode {
  simplex,
  longEdge,
  shortEdge,
}

class PrintLayoutSettings {
  const PrintLayoutSettings({
    this.scaleMode = PrintScaleMode.actualSize,
    this.scalePercent = 100,
    this.marginLeft = 0,
    this.marginRight = 0,
    this.marginTop = 0,
    this.marginBottom = 0,
    this.scope = PrintScope.document,
    this.duplex = PrintDuplexMode.simplex,
    this.pagesPerSheet = 1,
    this.booklet = false,
  });

  final PrintScaleMode scaleMode;
  final double scalePercent;
  final double marginLeft;
  final double marginRight;
  final double marginTop;
  final double marginBottom;
  final PrintScope scope;
  final PrintDuplexMode duplex;
  final int pagesPerSheet;
  final bool booklet;

  static const defaults = PrintLayoutSettings();

  static PrintLayoutSettings uniformMargins(double points) => PrintLayoutSettings(
        marginLeft: points,
        marginRight: points,
        marginTop: points,
        marginBottom: points,
      );

  static PrintLayoutSettings fitToMargins(double points) => PrintLayoutSettings(
        scaleMode: PrintScaleMode.fitToMargins,
        marginLeft: points,
        marginRight: points,
        marginTop: points,
        marginBottom: points,
      );

  static PrintLayoutSettings customScale(double percent) => PrintLayoutSettings(
        scaleMode: PrintScaleMode.customPercent,
        scalePercent: percent,
      );

  /// FFI / WASM scale_mode discriminant.
  int get scaleModeCode => switch (scaleMode) {
        PrintScaleMode.actualSize => 0,
        PrintScaleMode.fitToMargins => 1,
        PrintScaleMode.customPercent => 2,
      };

  /// FFI / WASM duplex discriminant (F25.S4).
  int get duplexCode => switch (effectiveDuplex) {
        PrintDuplexMode.simplex => 0,
        PrintDuplexMode.longEdge => 1,
        PrintDuplexMode.shortEdge => 2,
      };

  /// Booklet forces long-edge duplex.
  PrintDuplexMode get effectiveDuplex =>
      booklet ? PrintDuplexMode.longEdge : duplex;

  /// Booklet forces 2-up; otherwise normalize to a supported N-up grid.
  int get effectivePagesPerSheet {
    if (booklet) return 2;
    return normalizePagesPerSheet(pagesPerSheet);
  }

  static int normalizePagesPerSheet(int n) {
    if (n <= 1) return 1;
    if (n == 2) return 2;
    if (n <= 4) return 4;
    if (n <= 6) return 6;
    if (n <= 9) return 9;
    return 16;
  }

  static (int cols, int rows) nupGrid(int pagesPerSheet) {
    switch (normalizePagesPerSheet(pagesPerSheet)) {
      case 1:
        return (1, 1);
      case 2:
        return (2, 1);
      case 4:
        return (2, 2);
      case 6:
        return (3, 2);
      case 9:
        return (3, 3);
      default:
        return (4, 4);
    }
  }

  /// Classic saddle-stitch booklet order (0-based), padded to a multiple of 4.
  static List<int?> bookletPageOrder(int pageCount) {
    if (pageCount <= 0) return const [];
    final padded = ((pageCount + 3) ~/ 4) * 4;
    final sheets = padded ~/ 4;
    final out = <int?>[];
    for (var i = 0; i < sheets; i++) {
      final a = padded - 1 - 2 * i;
      final b = 2 * i;
      final c = 2 * i + 1;
      final d = padded - 2 - 2 * i;
      for (final idx in [a, b, c, d]) {
        out.add(idx < pageCount ? idx : null);
      }
    }
    return out;
  }

  PrintLayoutSettings copyWith({
    PrintScaleMode? scaleMode,
    double? scalePercent,
    double? marginLeft,
    double? marginRight,
    double? marginTop,
    double? marginBottom,
    PrintScope? scope,
    PrintDuplexMode? duplex,
    int? pagesPerSheet,
    bool? booklet,
  }) {
    return PrintLayoutSettings(
      scaleMode: scaleMode ?? this.scaleMode,
      scalePercent: scalePercent ?? this.scalePercent,
      marginLeft: marginLeft ?? this.marginLeft,
      marginRight: marginRight ?? this.marginRight,
      marginTop: marginTop ?? this.marginTop,
      marginBottom: marginBottom ?? this.marginBottom,
      scope: scope ?? this.scope,
      duplex: duplex ?? this.duplex,
      pagesPerSheet: pagesPerSheet ?? this.pagesPerSheet,
      booklet: booklet ?? this.booklet,
    );
  }

  Map<String, dynamic> toJson() => {
        'scaleMode': scaleMode.name,
        'scalePercent': scalePercent,
        'marginLeft': marginLeft,
        'marginRight': marginRight,
        'marginTop': marginTop,
        'marginBottom': marginBottom,
        'scope': scope.name,
        'duplex': duplex.name,
        'pagesPerSheet': pagesPerSheet,
        'booklet': booklet,
      };

  /// Attributes forwarded to the OS print host (F25.S4).
  Map<String, dynamic> toPlatformAttributes() => {
        'duplex': effectiveDuplex.name,
        'pagesPerSheet': effectivePagesPerSheet,
        'booklet': booklet,
      };

  /// Pure resolve helper mirrored from Rust (for unit tests).
  ({double scale, double tx, double ty}) resolve(double pageW, double pageH) {
    final w = pageW < 1 ? 1.0 : pageW;
    final h = pageH < 1 ? 1.0 : pageH;
    final ml = marginLeft.clamp(0.0, w * 0.45);
    final mr = marginRight.clamp(0.0, w * 0.45);
    final mt = marginTop.clamp(0.0, h * 0.45);
    final mb = marginBottom.clamp(0.0, h * 0.45);
    final availW = (w - ml - mr).clamp(1.0, double.infinity);
    final availH = (h - mt - mb).clamp(1.0, double.infinity);
    final scale = switch (scaleMode) {
      PrintScaleMode.fitToMargins =>
        (availW / w) < (availH / h) ? (availW / w) : (availH / h),
      PrintScaleMode.actualSize || PrintScaleMode.customPercent =>
        (scalePercent / 100).clamp(0.1, 4.0),
    };
    final contentW = w * scale;
    final contentH = h * scale;
    final tx = ml + ((availW - contentW) < 0 ? 0.0 : (availW - contentW) / 2);
    final ty = mb + ((availH - contentH) < 0 ? 0.0 : (availH - contentH) / 2);
    return (scale: scale, tx: tx, ty: ty);
  }
}
