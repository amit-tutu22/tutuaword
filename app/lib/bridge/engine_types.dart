import 'dart:typed_data';

/// Budget for an edit's worker round-trip (shared by FFI and WASM bridges).
const Duration kEditCompletionTimeout = Duration(milliseconds: 1500);

class DisplayListData {
  DisplayListData({
    required this.bytes,
    required this.version,
    required this.pageWidth,
    required this.pageHeight,
    required this.pageCount,
  });

  final Uint8List bytes;
  final int version;
  final double pageWidth;
  final double pageHeight;
  final int pageCount;
}

class PageDisplayListData {
  PageDisplayListData({
    required this.bytes,
    required this.version,
    required this.pageWidth,
    required this.pageHeight,
  });

  final Uint8List bytes;
  final int version;
  final double pageWidth;
  final double pageHeight;
}

class AtlasData {
  AtlasData({
    required this.generation,
    required this.bytes,
    required this.width,
    required this.height,
  });

  final int generation;
  final Uint8List bytes;
  final int width;
  final int height;
}

class HitTestResult {
  HitTestResult({required this.runId, required this.charOffset});

  final String runId;
  final int charOffset;
}

class CaretGeometry {
  CaretGeometry({required this.x, required this.y, required this.height});

  final double x;
  final double y;
  final double height;
}

class GlyphSelectionRect {
  GlyphSelectionRect({
    required this.x,
    required this.y,
    required this.width,
    required this.height,
  });

  final double x;
  final double y;
  final double width;
  final double height;
}
