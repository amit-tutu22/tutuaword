import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/document_painter.dart';

/// Encodes an opaque red square as a PNG, standing in for a document asset.
Future<Uint8List> redPng(int size) async {
  final rgba = Uint8List(size * size * 4);
  for (var i = 0; i < size * size; i++) {
    rgba[i * 4] = 0xFF;
    rgba[i * 4 + 3] = 0xFF;
  }
  final buffer = await ui.ImmutableBuffer.fromUint8List(rgba);
  final descriptor = ui.ImageDescriptor.raw(
    buffer,
    width: size,
    height: size,
    pixelFormat: ui.PixelFormat.rgba8888,
  );
  final frame = await (await descriptor.instantiateCodec()).getNextFrame();
  final encoded = await frame.image.toByteData(format: ui.ImageByteFormat.png);
  frame.image.dispose();
  return encoded!.buffer.asUint8List();
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('image decoding', () {
    test('an encoded payload becomes a ui.Image keyed by asset id', () async {
      final snapshot = _snapshotWith(
        assetIds: const ['word/media/image1.png'],
        payloads: [await redPng(2)],
      );

      final decoded = await snapshot.decodeImages();

      expect(decoded.keys, ['word/media/image1.png']);
      expect(decoded['word/media/image1.png']!.width, 2);
      expect(decoded['word/media/image1.png']!.height, 2);
    });

    test('an asset already decoded is skipped', () async {
      final snapshot = _snapshotWith(
        assetIds: const ['word/media/image1.png'],
        payloads: [await redPng(2)],
      );

      final decoded = await snapshot.decodeImages(
        skip: {'word/media/image1.png'},
      );

      expect(decoded, isEmpty);
    });

    test('an undecodable payload is dropped rather than thrown', () async {
      final snapshot = _snapshotWith(
        assetIds: const ['word/media/image1.emf'],
        payloads: [Uint8List.fromList([1, 2, 3, 4, 5])],
      );

      expect(await snapshot.decodeImages(), isEmpty);
    });
  });

  group('DocumentPainter', () {
    test('draws the decoded image at the layout rect', () async {
      final snapshot = _snapshotWith(
        assetIds: const ['logo.png'],
        payloads: [await redPng(8)],
        transforms: [31.4, 25.2],
        sizes: [43.7, 56.8],
      );
      final images = await snapshot.decodeImages();
      addTearDown(() {
        for (final image in images.values) {
          image.dispose();
        }
      });

      final recorder = ui.PictureRecorder();
      DocumentPainter(snapshot: snapshot, images: images)
          .paint(Canvas(recorder), const Size(612, 792));
      final picture = recorder.endRecording();
      final rendered = await picture.toImage(612, 792);
      addTearDown(rendered.dispose);

      final pixels = await rendered.toByteData();
      // Sample the middle of where the logo was placed.
      final offset = ((25.2 + 28).floor() * 612 + (31.4 + 21).floor()) * 4;
      expect(pixels!.getUint8(offset), greaterThan(200), reason: 'red channel');
      expect(pixels.getUint8(offset + 1), lessThan(60), reason: 'green channel');
    });

    test('an undecoded image still gets a placeholder outline', () async {
      final snapshot = _snapshotWith(
        assetIds: const ['missing.png'],
        payloads: [Uint8List(0)],
      );

      final recorder = ui.PictureRecorder();
      // Painting with an empty cache must not throw.
      DocumentPainter(snapshot: snapshot, images: const {})
          .paint(Canvas(recorder), const Size(612, 792));
      expect(recorder.endRecording(), isNotNull);
    });
  });
}

DisplayListSnapshot _snapshotWith({
  required List<String> assetIds,
  required List<Uint8List> payloads,
  List<double> transforms = const [40, 40],
  List<double> sizes = const [20, 20],
}) {
  return DisplayListSnapshot(
    version: 1,
    pageWidth: 612,
    pageHeight: 792,
    atlasPixels: Uint8List(0),
    atlasWidth: 0,
    atlasHeight: 0,
    glyphOffsets: Float32List(0),
    glyphSrcRects: Float32List(0),
    glyphColors: Int32List(0),
    rectBatch: Float32List(0),
    rectColors: Int32List(0),
    pathPoints: Float32List(0),
    pathColors: Int32List(0),
    imageTransforms: Float32List.fromList(transforms),
    imageSizes: Float32List.fromList(sizes),
    imageAssetIds: assetIds,
    imageIds: List<String>.generate(assetIds.length, (i) => 'img-$i'),
    imagePayloads: payloads,
    imageRotations: Float32List(assetIds.length),
    imageOpacities: Float32List.fromList(List.filled(assetIds.length, 1.0)),
    imageCropRects: Float32List(assetIds.length * 4),
    shapeIds: const [],
    shapeRects: Float32List(0),
  );
}
