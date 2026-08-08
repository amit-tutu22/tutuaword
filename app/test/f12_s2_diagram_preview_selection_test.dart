import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/image_hit_test.dart';

import 'editor_test_helpers.dart';

DisplayListSnapshot _snapshotWithShapeSelection({
  required String shapeId,
  required Rect shapeRect,
}) {
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
    imageTransforms: Float32List(0),
    imageSizes: Float32List(0),
    imageAssetIds: const [],
    imageIds: const [],
    imagePayloads: const [],
    imageRotations: Float32List(0),
    imageOpacities: Float32List(0),
    imageCropRects: Float32List(0),
    shapeIds: [shapeId],
    shapeRects: Float32List.fromList([
      shapeRect.left,
      shapeRect.top,
      shapeRect.width,
      shapeRect.height,
    ]),
  );
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('I-F12-S2-diagram-preview-selection selects read-only shape', () {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    controller.selectImage(
      0,
      ImageBounds(
        imageId: 'img-1',
        index: 0,
        rect: const Rect.fromLTWH(10, 10, 50, 50),
      ),
    );

    final snapshot = _snapshotWithShapeSelection(
      shapeId: 'diagram-preview-1',
      shapeRect: const Rect.fromLTWH(100, 100, 432, 216),
    );

    final hit = controller.trySelectDiagramAt(
      0,
      const Offset(200, 150),
      snapshot,
    );

    expect(hit, isTrue);
    expect(controller.selectedDiagramId, 'diagram-preview-1');
    expect(controller.selectedDiagramRect, isNotNull);
    expect(controller.hasSelectedImage, isFalse);
  });
}
