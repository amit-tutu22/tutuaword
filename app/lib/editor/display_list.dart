import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:tutuaword/editor/formatting_marks.dart';

/// Parsed display list from Rust tw-render binary format (v3/v4).
class DisplayListSnapshot {
  DisplayListSnapshot({
    required this.version,
    required this.pageWidth,
    required this.pageHeight,
    required this.atlasPixels,
    required this.atlasWidth,
    required this.atlasHeight,
    required this.glyphOffsets,
    required this.glyphSrcRects,
    required this.glyphColors,
    required this.rectBatch,
    required this.rectColors,
    required this.pathPoints,
    required this.pathColors,
    required this.imageTransforms,
    required this.imageSizes,
    required this.imageAssetIds,
    required this.imageIds,
    required this.imagePayloads,
    required this.imageRotations,
    required this.imageOpacities,
    required this.imageCropRects,
    required this.shapeIds,
    required this.shapeRects,
    this.formattingMarks = const [],
  });

  final int version;
  final double pageWidth;
  final double pageHeight;
  final Uint8List atlasPixels;
  final int atlasWidth;
  final int atlasHeight;
  final Float32List glyphOffsets;
  final Float32List glyphSrcRects;
  final Int32List glyphColors;
  final Float32List rectBatch;
  final Int32List rectColors;
  final Float32List pathPoints;
  final Int32List pathColors;
  final Float32List imageTransforms;
  final Float32List imageSizes;
  final List<String> imageAssetIds;
  final List<String> imageIds;

  /// Encoded source bytes (PNG, JPEG, ...) per image, parallel to
  /// [imageAssetIds]. Empty for an image whose asset could not be resolved.
  final List<Uint8List> imagePayloads;
  final Float32List imageRotations;
  final Float32List imageOpacities;
  final Float32List imageCropRects;
  final List<String> shapeIds;
  final Float32List shapeRects;
  final List<FormattingMark> formattingMarks;

  /// Maximum glyphs we will parse from a single page payload (fail-soft beyond this).
  static const int _maxGlyphCount = 500000;

  factory DisplayListSnapshot.fromBytes(Uint8List bytes) {
    if (bytes.length < 20) {
      return DisplayListSnapshot.empty();
    }
    var offset = 0;
    final fileVersion = _readU32(bytes, offset);
    offset += 4;

    final version = _readU64(bytes, offset);
    offset += 8;
    final pageWidth = _readF32(bytes, offset);
    offset += 4;
    final pageHeight = _readF32(bytes, offset);
    offset += 4;

    var atlasWidth = 0;
    var atlasHeight = 0;
    var atlasPixels = Uint8List(0);
    if (fileVersion < 4) {
      atlasWidth = _readU32(bytes, offset);
      offset += 4;
      atlasHeight = _readU32(bytes, offset);
      offset += 4;
      final atlasLen = _readU32(bytes, offset);
      offset += 4;
      if (atlasLen > 256 * 1024 * 1024 || offset + atlasLen > bytes.length) {
        return DisplayListSnapshot.empty();
      }
      atlasPixels = bytes.sublist(offset, offset + atlasLen);
      offset += atlasLen;
    }

    if (offset + 4 > bytes.length) {
      return DisplayListSnapshot.empty();
    }
    final glyphCount = _readU32(bytes, offset);
    offset += 4;
    if (glyphCount > _maxGlyphCount ||
        offset + glyphCount * 24 + 4 > bytes.length) {
      return DisplayListSnapshot.empty();
    }
    final glyphOffsets = Float32List(glyphCount * 2);
    for (var i = 0; i < glyphCount * 2; i++) {
      if (offset + 4 > bytes.length) {
        return DisplayListSnapshot.empty();
      }
      glyphOffsets[i] = _readF32(bytes, offset);
      offset += 4;
    }
    final glyphSrcRects = Float32List(glyphCount * 4);
    for (var i = 0; i < glyphCount * 4; i++) {
      if (offset + 4 > bytes.length) {
        return DisplayListSnapshot.empty();
      }
      glyphSrcRects[i] = _readF32(bytes, offset);
      offset += 4;
    }
    final glyphColors = Int32List(glyphCount);
    for (var i = 0; i < glyphCount; i++) {
      if (offset + 4 > bytes.length) {
        return DisplayListSnapshot.empty();
      }
      glyphColors[i] = _readU32(bytes, offset);
      offset += 4;
    }

    final rectData = _readRectBatch(bytes, offset);
    offset = rectData.$3;

    Float32List pathPoints = Float32List(0);
    Int32List pathColors = Int32List(0);
    Float32List imageTransforms = Float32List(0);
    Float32List imageSizes = Float32List(0);
    var imageAssetIds = <String>[];
    var imageIds = <String>[];
    var imagePayloads = <Uint8List>[];
    Float32List imageRotations = Float32List(0);
    Float32List imageOpacities = Float32List(0);
    Float32List imageCropRects = Float32List(0);
    var shapeIds = <String>[];
    Float32List shapeRects = Float32List(0);
    var formattingMarks = const <FormattingMark>[];
    if (fileVersion >= 2 && offset + 4 <= bytes.length) {
      final pathData = _readPathBatch(bytes, offset);
      pathPoints = pathData.$1;
      pathColors = pathData.$2;
      offset = pathData.$3;
      if (offset + 4 <= bytes.length) {
        final imageData = _readImageBatch(bytes, offset, fileVersion);
        imageTransforms = imageData.$1;
        imageSizes = imageData.$2;
        imageAssetIds = imageData.$3;
        imageIds = imageData.$4;
        imagePayloads = imageData.$5;
        imageRotations = imageData.$6;
        imageOpacities = imageData.$7;
        imageCropRects = imageData.$8;
        offset = imageData.$9;
      }
      if (fileVersion >= 7 && offset + 4 <= bytes.length) {
        final shapeData = _readShapeSelectionBatch(bytes, offset);
        shapeIds = shapeData.$1;
        shapeRects = shapeData.$2;
        offset = shapeData.$3;
      }
      if (fileVersion >= 8 && offset + 4 <= bytes.length) {
        final markData = _readFormattingMarksBatch(bytes, offset);
        formattingMarks = markData.$1;
        offset = markData.$2;
      }
    }

    return DisplayListSnapshot(
      version: version,
      pageWidth: pageWidth,
      pageHeight: pageHeight,
      atlasPixels: Uint8List.fromList(atlasPixels),
      atlasWidth: atlasWidth,
      atlasHeight: atlasHeight,
      glyphOffsets: glyphOffsets,
      glyphSrcRects: glyphSrcRects,
      glyphColors: glyphColors,
      rectBatch: rectData.$1,
      rectColors: rectData.$2,
      pathPoints: pathPoints,
      pathColors: pathColors,
      imageTransforms: imageTransforms,
      imageSizes: imageSizes,
      imageAssetIds: imageAssetIds,
      imageIds: imageIds,
      imagePayloads: imagePayloads,
      imageRotations: imageRotations,
      imageOpacities: imageOpacities,
      imageCropRects: imageCropRects,
      shapeIds: shapeIds,
      shapeRects: shapeRects,
      formattingMarks: formattingMarks,
    );
  }

  factory DisplayListSnapshot.empty() {
    return DisplayListSnapshot(
      version: 0,
      pageWidth: 612,
      pageHeight: 792,
      atlasPixels: Uint8List(0),
      atlasWidth: 1,
      atlasHeight: 1,
      glyphOffsets: Float32List(0),
      glyphSrcRects: Float32List(0),
      glyphColors: Int32List(0),
      rectBatch: Float32List(0),
      rectColors: Int32List(0),
      pathPoints: Float32List(0),
      pathColors: Int32List(0),
      imageTransforms: Float32List(0),
      imageSizes: Float32List(0),
      imageAssetIds: [],
      imageIds: [],
      imagePayloads: [],
      imageRotations: Float32List(0),
      imageOpacities: Float32List(0),
      imageCropRects: Float32List(0),
      shapeIds: [],
      shapeRects: Float32List(0),
      formattingMarks: const [],
    );
  }

  Future<ui.Image?> buildAtlasImage() async {
    return buildAtlasImageFromPixels(atlasPixels, atlasWidth, atlasHeight);
  }

  /// Upload RGBA atlas pixels to a GPU texture.
  static Future<ui.Image?> buildAtlasImageFromPixels(
    Uint8List pixels,
    int width,
    int height,
  ) async {
    if (pixels.isEmpty || width == 0 || height == 0) {
      return null;
    }
    final buffer = await ui.ImmutableBuffer.fromUint8List(pixels);
    final descriptor = ui.ImageDescriptor.raw(
      buffer,
      width: width,
      height: height,
      pixelFormat: ui.PixelFormat.rgba8888,
    );
    final codec = await descriptor.instantiateCodec();
    final frame = await codec.getNextFrame();
    return frame.image;
  }

  /// Decodes this page's embedded images, keyed by asset id. Ids in [skip] are
  /// left alone so pages sharing an asset only decode it once.
  Future<Map<String, ui.Image>> decodeImages({
    Set<String> skip = const {},
    Future<Uint8List?> Function(String assetId)? resolveAsset,
  }) async {
    final decoded = <String, ui.Image>{};
    for (var i = 0; i < imageAssetIds.length; i++) {
      final id = imageAssetIds[i];
      if (skip.contains(id) || decoded.containsKey(id) || i >= imagePayloads.length) {
        continue;
      }
      var payload = imagePayloads[i];
      if (payload.isEmpty && resolveAsset != null) {
        payload = await resolveAsset(id) ?? Uint8List(0);
      }
      if (payload.isEmpty) {
        continue;
      }
      try {
        final buffer = await ui.ImmutableBuffer.fromUint8List(payload);
        final descriptor = await ui.ImageDescriptor.encoded(buffer);
        final codec = await descriptor.instantiateCodec();
        final frame = await codec.getNextFrame();
        decoded[id] = frame.image;
      } catch (_) {
        // An undecodable asset (EMF/WMF, corrupt data) keeps its placeholder.
      }
    }
    return decoded;
  }

  bool get hasPaintableGlyphs => glyphCount > 0;

  /// True when glyph draws need an atlas texture (v4 page lists omit pixels).
  bool get needsAtlasTexture =>
      glyphCount > 0 &&
      (atlasPixels.isNotEmpty || atlasWidth == 0);

  int get glyphCount => glyphOffsets.length ~/ 2;

  /// True when the snapshot has vector/table/image content even without glyphs.
  bool get hasPaintableContent =>
      hasPaintableGlyphs ||
      pathPoints.isNotEmpty ||
      rectBatch.isNotEmpty ||
      imageTransforms.isNotEmpty;
}

(Float32List, Int32List, int) _readRectBatch(Uint8List bytes, int offset) {
  if (offset + 4 > bytes.length) {
    return (Float32List(0), Int32List(0), offset);
  }
  final rectCount = _readU32(bytes, offset);
  offset += 4;
  final rects = Float32List(rectCount * 4);
  for (var i = 0; i < rectCount * 4; i++) {
    if (offset + 4 > bytes.length) break;
    rects[i] = _readF32(bytes, offset);
    offset += 4;
  }
  final colors = Int32List(rectCount);
  for (var i = 0; i < rectCount; i++) {
    if (offset + 4 > bytes.length) break;
    colors[i] = _readU32(bytes, offset);
    offset += 4;
  }
  return (rects, colors, offset);
}

(Float32List, Int32List, int) _readPathBatch(Uint8List bytes, int offset) {
  if (offset + 4 > bytes.length) {
    return (Float32List(0), Int32List(0), offset);
  }
  final lineCount = _readU32(bytes, offset);
  offset += 4;
  final points = Float32List(lineCount * 4);
  for (var i = 0; i < lineCount * 4; i++) {
    if (offset + 4 > bytes.length) break;
    points[i] = _readF32(bytes, offset);
    offset += 4;
  }
  final colors = Int32List(lineCount);
  for (var i = 0; i < lineCount; i++) {
    if (offset + 4 > bytes.length) break;
    colors[i] = _readU32(bytes, offset);
    offset += 4;
  }
  return (points, colors, offset);
}

(Float32List, Float32List, List<String>, List<String>, List<Uint8List>, Float32List,
    Float32List, Float32List, int)
_readImageBatch(
  Uint8List bytes,
  int offset,
  int fileVersion,
) {
  if (offset + 4 > bytes.length) {
    return (
      Float32List(0),
      Float32List(0),
      <String>[],
      <String>[],
      <Uint8List>[],
      Float32List(0),
      Float32List(0),
      Float32List(0),
      offset,
    );
  }
  final imageCount = _readU32(bytes, offset);
  offset += 4;
  final transforms = Float32List(imageCount * 2);
  for (var i = 0; i < imageCount * 2; i++) {
    if (offset + 4 > bytes.length) break;
    transforms[i] = _readF32(bytes, offset);
    offset += 4;
  }
  final sizes = Float32List(imageCount * 2);
  for (var i = 0; i < imageCount * 2; i++) {
    if (offset + 4 > bytes.length) break;
    sizes[i] = _readF32(bytes, offset);
    offset += 4;
  }
  final assetIds = <String>[];
  for (var i = 0; i < imageCount; i++) {
    if (offset + 4 > bytes.length) break;
    final len = _readU32(bytes, offset);
    offset += 4;
    if (offset + len > bytes.length) break;
    assetIds.add(String.fromCharCodes(bytes.sublist(offset, offset + len)));
    offset += len;
  }
  final imageIds = <String>[];
  if (fileVersion >= 5) {
    for (var i = 0; i < imageCount; i++) {
      if (offset + 4 > bytes.length) break;
      final len = _readU32(bytes, offset);
      offset += 4;
      if (offset + len > bytes.length) break;
      imageIds.add(String.fromCharCodes(bytes.sublist(offset, offset + len)));
      offset += len;
    }
  }
  final payloads = List<Uint8List>.filled(imageCount, Uint8List(0));
  if (fileVersion >= 3) {
    for (var i = 0; i < imageCount; i++) {
      if (offset + 4 > bytes.length) break;
      final len = _readU32(bytes, offset);
      offset += 4;
      if (offset + len > bytes.length) break;
      payloads[i] = Uint8List.sublistView(bytes, offset, offset + len);
      offset += len;
    }
  }
  var rotations = Float32List(0);
  var opacities = Float32List(0);
  var cropRects = Float32List(0);
  if (fileVersion >= 6 && imageCount > 0) {
    rotations = Float32List(imageCount);
    for (var i = 0; i < imageCount; i++) {
      if (offset + 4 > bytes.length) break;
      rotations[i] = _readF32(bytes, offset);
      offset += 4;
    }
    opacities = Float32List(imageCount);
    for (var i = 0; i < imageCount; i++) {
      if (offset + 4 > bytes.length) break;
      opacities[i] = _readF32(bytes, offset);
      offset += 4;
    }
    cropRects = Float32List(imageCount * 4);
    for (var i = 0; i < imageCount * 4; i++) {
      if (offset + 4 > bytes.length) break;
      cropRects[i] = _readF32(bytes, offset);
      offset += 4;
    }
  }
  return (
    transforms,
    sizes,
    assetIds,
    imageIds,
    payloads,
    rotations,
    opacities,
    cropRects,
    offset,
  );
}

(List<String>, Float32List, int) _readShapeSelectionBatch(Uint8List bytes, int offset) {
  if (offset + 4 > bytes.length) {
    return (<String>[], Float32List(0), offset);
  }
  final shapeCount = _readU32(bytes, offset);
  offset += 4;
  final rects = Float32List(shapeCount * 4);
  for (var i = 0; i < shapeCount * 4; i++) {
    if (offset + 4 > bytes.length) break;
    rects[i] = _readF32(bytes, offset);
    offset += 4;
  }
  final shapeIds = <String>[];
  for (var i = 0; i < shapeCount; i++) {
    if (offset + 4 > bytes.length) break;
    final len = _readU32(bytes, offset);
    offset += 4;
    if (offset + len > bytes.length) break;
    shapeIds.add(String.fromCharCodes(bytes.sublist(offset, offset + len)));
    offset += len;
  }
  return (shapeIds, rects, offset);
}

(List<FormattingMark>, int) _readFormattingMarksBatch(Uint8List bytes, int offset) {
  if (offset + 4 > bytes.length) {
    return (const [], offset);
  }
  final markCount = _readU32(bytes, offset);
  offset += 4;
  const maxMarks = 200000;
  if (markCount > maxMarks) {
    return (const [], offset);
  }
  final positionsLen = markCount * 3;
  if (offset + positionsLen * 4 + markCount > bytes.length) {
    return (const [], offset);
  }
  final marks = <FormattingMark>[];
  for (var i = 0; i < markCount; i++) {
    final x = _readF32(bytes, offset + i * 12);
    final y = _readF32(bytes, offset + i * 12 + 4);
    final height = _readF32(bytes, offset + i * 12 + 8);
    final kindByte = bytes[offset + positionsLen * 4 + i];
    final kind = switch (kindByte) {
      1 => FormattingMarkKind.tab,
      2 => FormattingMarkKind.paragraph,
      _ => FormattingMarkKind.space,
    };
    marks.add(FormattingMark(kind: kind, x: x, y: y, height: height));
  }
  offset += positionsLen * 4 + markCount;
  return (marks, offset);
}

int _readU32(Uint8List bytes, int offset) {
  return ByteData.sublistView(bytes, offset, offset + 4).getUint32(0, Endian.little);
}

int _readU64(Uint8List bytes, int offset) {
  // dart2js rejects ByteData.getUint64 — assemble LE u64 from bytes.
  final b0 = bytes[offset];
  final b1 = bytes[offset + 1];
  final b2 = bytes[offset + 2];
  final b3 = bytes[offset + 3];
  final b4 = bytes[offset + 4];
  final b5 = bytes[offset + 5];
  final b6 = bytes[offset + 6];
  final b7 = bytes[offset + 7];
  final low = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
  final high = b4 | (b5 << 8) | (b6 << 16) | (b7 << 24);
  return low + high * 0x100000000;
}

double _readF32(Uint8List bytes, int offset) {
  return ByteData.sublistView(bytes, offset, offset + 4).getFloat32(0, Endian.little);
}
