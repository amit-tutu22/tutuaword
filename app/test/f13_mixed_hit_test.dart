import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/editor_controller.dart';

import 'editor_test_helpers.dart';

DisplayListSnapshot _overlappingSnapshot() {
  return DisplayListSnapshot(
    version: 7,
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
    imageTransforms: Float32List.fromList([100, 100]),
    imageSizes: Float32List.fromList([200, 150]),
    imageAssetIds: const ['preview.png'],
    imageIds: const ['image-block-1'],
    imagePayloads: [Uint8List.fromList(const [1, 2, 3])],
    imageRotations: Float32List(1),
    imageOpacities: Float32List(1),
    imageCropRects: Float32List(4),
    shapeIds: const ['chart-preview-1'],
    shapeRects: Float32List.fromList([100, 100, 200, 150]),
  );
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('I-F13-mixed-hit-test shape selection wins over image at overlap', () {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    final snapshot = _overlappingSnapshot();
    const point = Offset(150, 125);

    expect(controller.trySelectDiagramAt(0, point, snapshot), isTrue);
    expect(controller.selectedDiagramId, 'chart-preview-1');
    expect(controller.hasSelectedImage, isFalse);

    expect(controller.trySelectImageAt(0, point, snapshot), isTrue);
    expect(controller.hasSelectedImage, isTrue);
    expect(controller.hasSelectedDiagram, isFalse);
  });
}
