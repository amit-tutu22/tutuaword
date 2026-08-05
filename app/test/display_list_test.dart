import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/display_list.dart';

void main() {
  group('DisplayListSnapshot', () {
    test('empty bytes returns default snapshot', () {
      final snap = DisplayListSnapshot.fromBytes(Uint8List(0));
      expect(snap.version, 0);
      expect(snap.pageWidth, 612);
      expect(snap.pageHeight, 792);
      expect(snap.hasPaintableGlyphs, isFalse);
    });

    test('parses v2 display list with glyphs, rects, paths, and images', () {
      final bytes = _buildV2DisplayList(
        version: 7,
        pageWidth: 612,
        pageHeight: 792,
        glyphCount: 2,
        rectCount: 1,
        pathCount: 1,
        imageCount: 1,
        imageAssetId: 'placeholder.png',
      );

      final snap = DisplayListSnapshot.fromBytes(bytes);
      expect(snap.version, 7);
      expect(snap.pageWidth, 612);
      expect(snap.pageHeight, 792);
      expect(snap.glyphOffsets.length, 4);
      expect(snap.glyphSrcRects.length, 8);
      expect(snap.glyphColors.length, 2);
      expect(snap.rectBatch.length, 4);
      expect(snap.rectColors.length, 1);
      expect(snap.pathPoints.length, 4);
      expect(snap.pathColors.length, 1);
      expect(snap.imageTransforms.length, 2);
      expect(snap.imageSizes.length, 2);
      expect(snap.imageAssetIds, ['placeholder.png']);
      expect(snap.hasPaintableGlyphs, isTrue);
    });

    test('v1 bytes omit path and image batches gracefully', () {
      final bytes = _buildV1DisplayList(glyphCount: 1);
      final snap = DisplayListSnapshot.fromBytes(bytes);
      expect(snap.pathPoints, isEmpty);
      expect(snap.imageAssetIds, isEmpty);
    });

    test('v2 bytes leave image payloads empty', () {
      final bytes = _buildV2DisplayList(
        version: 1,
        pageWidth: 612,
        pageHeight: 792,
        glyphCount: 0,
        rectCount: 0,
        pathCount: 0,
        imageCount: 1,
        imageAssetId: 'placeholder.png',
      );
      final snap = DisplayListSnapshot.fromBytes(bytes);
      expect(snap.imageAssetIds, ['placeholder.png']);
      expect(snap.imagePayloads.single, isEmpty);
    });

    test('v3 carries encoded image payloads alongside asset ids', () {
      final payload = Uint8List.fromList([137, 80, 78, 71, 13, 10, 26, 10]);
      final bytes = _buildV3DisplayList(
        assetIds: const ['word/media/image1.png', 'word/media/image2.png'],
        payloads: [payload, Uint8List(0)],
      );

      final snap = DisplayListSnapshot.fromBytes(bytes);
      expect(snap.imageAssetIds, [
        'word/media/image1.png',
        'word/media/image2.png',
      ]);
      expect(snap.imagePayloads.first, payload);
      expect(snap.imagePayloads.last, isEmpty);
      expect(snap.hasPaintableContent, isTrue,
          reason: 'a page holding only an image still needs painting');
    });

    test('a truncated payload does not throw', () {
      final bytes = _buildV3DisplayList(
        assetIds: const ['a.png'],
        payloads: [Uint8List.fromList([1, 2, 3, 4])],
      );
      final truncated = Uint8List.sublistView(bytes, 0, bytes.length - 2);

      final snap = DisplayListSnapshot.fromBytes(truncated);
      expect(snap.imagePayloads.single, isEmpty);
    });
  });
}

Uint8List _buildV3DisplayList({
  required List<String> assetIds,
  required List<Uint8List> payloads,
}) {
  final writer = _ByteWriter();
  writer.writeU32(3);
  writer.writeU64(1);
  writer.writeF32(612);
  writer.writeF32(792);
  writer.writeU32(1);
  writer.writeU32(1);
  writer.writeU32(4);
  writer.writeBytes([0, 0, 0, 255]);
  _writeGlyphBatch(writer, 0);
  _writeRectBatch(writer, 0);
  _writePathBatch(writer, 0);

  writer.writeU32(assetIds.length);
  for (var i = 0; i < assetIds.length * 2; i++) {
    writer.writeF32(72 + i * 10.0);
  }
  for (var i = 0; i < assetIds.length * 2; i++) {
    writer.writeF32(50 + i * 10.0);
  }
  for (final id in assetIds) {
    writer.writeU32(id.codeUnits.length);
    writer.writeBytes(id.codeUnits);
  }
  for (final payload in payloads) {
    writer.writeU32(payload.length);
    writer.writeBytes(payload);
  }
  return writer.toBytes();
}

Uint8List _buildV1DisplayList({required int glyphCount}) {
  final writer = _ByteWriter();
  writer.writeU32(1);
  writer.writeU64(1);
  writer.writeF32(612);
  writer.writeF32(792);
  writer.writeU32(1);
  writer.writeU32(1);
  writer.writeU32(4);
  writer.writeBytes([0, 0, 0, 255]);
  _writeGlyphBatch(writer, glyphCount);
  _writeRectBatch(writer, 0);
  return writer.toBytes();
}

Uint8List _buildV2DisplayList({
  required int version,
  required double pageWidth,
  required double pageHeight,
  required int glyphCount,
  required int rectCount,
  required int pathCount,
  required int imageCount,
  required String imageAssetId,
}) {
  final writer = _ByteWriter();
  writer.writeU32(2);
  writer.writeU64(version);
  writer.writeF32(pageWidth);
  writer.writeF32(pageHeight);
  writer.writeU32(2);
  writer.writeU32(2);
  writer.writeU32(16);
  writer.writeBytes(List.filled(16, 128));
  _writeGlyphBatch(writer, glyphCount);
  _writeRectBatch(writer, rectCount);
  _writePathBatch(writer, pathCount);
  _writeImageBatch(writer, imageCount, imageAssetId);
  return writer.toBytes();
}

void _writeGlyphBatch(_ByteWriter writer, int glyphCount) {
  writer.writeU32(glyphCount);
  for (var i = 0; i < glyphCount * 2; i++) {
    writer.writeF32(i * 10.0);
  }
  for (var i = 0; i < glyphCount * 4; i++) {
    writer.writeF32(i.toDouble());
  }
  for (var i = 0; i < glyphCount; i++) {
    writer.writeU32(0xFF000000);
  }
}

void _writeRectBatch(_ByteWriter writer, int rectCount) {
  writer.writeU32(rectCount);
  for (var i = 0; i < rectCount * 4; i++) {
    writer.writeF32(i * 5.0);
  }
  for (var i = 0; i < rectCount; i++) {
    writer.writeU32(0xFFE0E0E0);
  }
}

void _writePathBatch(_ByteWriter writer, int lineCount) {
  writer.writeU32(lineCount);
  for (var i = 0; i < lineCount * 4; i++) {
    writer.writeF32(i.toDouble());
  }
  for (var i = 0; i < lineCount; i++) {
    writer.writeU32(0xFF000000);
  }
}

void _writeImageBatch(_ByteWriter writer, int imageCount, String assetId) {
  writer.writeU32(imageCount);
  for (var i = 0; i < imageCount * 2; i++) {
    writer.writeF32(72 + i * 10.0);
  }
  for (var i = 0; i < imageCount * 2; i++) {
    writer.writeF32(100 + i * 20.0);
  }
  for (var i = 0; i < imageCount; i++) {
    final idBytes = assetId.codeUnits;
    writer.writeU32(idBytes.length);
    writer.writeBytes(idBytes);
  }
}

class _ByteWriter {
  final _bytes = BytesBuilder();

  void writeU32(int value) {
    final data = ByteData(4)..setUint32(0, value, Endian.little);
    _bytes.add(data.buffer.asUint8List());
  }

  void writeU64(int value) {
    final data = ByteData(8)..setUint64(0, value, Endian.little);
    _bytes.add(data.buffer.asUint8List());
  }

  void writeF32(double value) {
    final data = ByteData(4)..setFloat32(0, value, Endian.little);
    _bytes.add(data.buffer.asUint8List());
  }

  void writeBytes(List<int> bytes) {
    _bytes.add(bytes);
  }

  Uint8List toBytes() => _bytes.toBytes();
}
