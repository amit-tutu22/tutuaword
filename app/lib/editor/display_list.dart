import 'dart:typed_data';
import 'dart:ui' as ui;

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
    required this.imagePayloads,
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

  /// Encoded source bytes (PNG, JPEG, ...) per image, parallel to
  /// [imageAssetIds]. Empty for an image whose asset could not be resolved.
  final List<Uint8List> imagePayloads;

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
      atlasPixels = bytes.sublist(offset, offset + atlasLen);
      offset += atlasLen;
    }

    final glyphCount = _readU32(bytes, offset);
    offset += 4;
    final glyphOffsets = Float32List(glyphCount * 2);
    for (var i = 0; i < glyphCount * 2; i++) {
      glyphOffsets[i] = _readF32(bytes, offset);
      offset += 4;
    }
    final glyphSrcRects = Float32List(glyphCount * 4);
    for (var i = 0; i < glyphCount * 4; i++) {
      glyphSrcRects[i] = _readF32(bytes, offset);
      offset += 4;
    }
    final glyphColors = Int32List(glyphCount);
    for (var i = 0; i < glyphCount; i++) {
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
    var imagePayloads = <Uint8List>[];
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
        imagePayloads = imageData.$4;
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
      imagePayloads: imagePayloads,
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
      imagePayloads: [],
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
  }) async {
    final decoded = <String, ui.Image>{};
    for (var i = 0; i < imageAssetIds.length; i++) {
      final id = imageAssetIds[i];
      if (skip.contains(id) || decoded.containsKey(id) || i >= imagePayloads.length) {
        continue;
      }
      final payload = imagePayloads[i];
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

(Float32List, Float32List, List<String>, List<Uint8List>) _readImageBatch(
  Uint8List bytes,
  int offset,
  int fileVersion,
) {
  if (offset + 4 > bytes.length) {
    return (Float32List(0), Float32List(0), <String>[], <Uint8List>[]);
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
  return (transforms, sizes, assetIds, payloads);
}

int _readU32(Uint8List bytes, int offset) {
  return ByteData.sublistView(bytes, offset, offset + 4).getUint32(0, Endian.little);
}

int _readU64(Uint8List bytes, int offset) {
  return ByteData.sublistView(bytes, offset, offset + 8).getUint64(0, Endian.little);
}

double _readF32(Uint8List bytes, int offset) {
  return ByteData.sublistView(bytes, offset, offset + 4).getFloat32(0, Endian.little);
}
