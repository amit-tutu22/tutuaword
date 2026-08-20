import 'dart:typed_data';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/shape_hit_test.dart';

import 'editor_test_helpers.dart';

DisplayListSnapshot _shapeSnapshot(ShapeBounds bounds) {
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
    shapeIds: [bounds.shapeId],
    shapeRects: Float32List.fromList([
      bounds.rect.left,
      bounds.rect.top,
      bounds.rect.width,
      bounds.rect.height,
    ]),
  );
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('I-F11-type-into-selected-shape enters body text edit',
      (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    controller.selectDiagram(
      0,
      const ShapeBounds(
        shapeId: 'shape-rect-1',
        index: 0,
        rect: Rect.fromLTWH(72, 72, 180, 90),
      ),
    );
    expect(controller.hasSelectedDiagram, isTrue);

    await controller.insertGlyphCharacter('H');
    await settleEngineStyle(tester);

    expect(controller.hasSelectedDiagram, isFalse);
    expect(engine.text, contains('H'));
  });

  testWidgets('I-F11-insert-word-art-returns-keyboard-focus', (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    final before = controller.editorFocusEpoch;
    await controller.insertWordArt('WordArt');
    await settleEngineStyle(tester);

    expect(controller.editorFocusEpoch, greaterThan(before));
  });

  testWidgets('I-F11-selected-wordart-types-into-seed-run-not-body',
      (tester) async {
    final engine = MockDocumentEngine();
    const seedRun = '00000000-0000-0000-0000-00000000aa01';
    engine.lastSplitCaret = HitTestResult(runId: seedRun, charOffset: 0);
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    controller.selectDiagram(
      0,
      const ShapeBounds(
        shapeId: 'wordart-1',
        index: 0,
        rect: Rect.fromLTWH(72, 400, 240, 72),
      ),
    );

    await controller.insertGlyphCharacter('Z');
    await settleEngineStyle(tester);

    expect(controller.hasSelectedDiagram, isFalse);
    expect(controller.caretRunId, seedRun);
    expect(engine.text, contains('Z'));
  });

  testWidgets('I-F11-second-click-inside-shape-enters-text-edit', (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    const bounds = ShapeBounds(
      shapeId: 'shape-rect-2',
      index: 0,
      rect: Rect.fromLTWH(72, 72, 180, 90),
    );
    controller.selectDiagram(0, bounds);

    // Interior second click enters text edit (consumes the pointer).
    final consumed = controller.trySelectDiagramAt(
      0,
      const Offset(100, 100),
      _shapeSnapshot(bounds),
    );
    expect(consumed, isTrue);
    await settleEngineStyle(tester);
    expect(controller.hasSelectedDiagram, isFalse);
  });

  test('I-F12-click-another-smartart-node-while-editing-keeps-text-caret', () async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    const bounds = ShapeBounds(
      shapeId: 'smartart-1',
      index: 0,
      rect: Rect.fromLTWH(72, 100, 432, 216),
    );
    controller.selectDiagram(0, bounds);
    final entered = await controller.enterSelectedShapeTextEdit(
      at: const Offset(120, 180),
    );
    expect(entered, isTrue);
    expect(controller.hasSelectedDiagram, isFalse);

    final consumed = controller.trySelectDiagramAt(
      0,
      const Offset(280, 180),
      _shapeSnapshot(bounds),
    );
    expect(consumed, isTrue);
    expect(controller.hasSelectedDiagram, isFalse,
        reason: 'clicking another SmartArt box must not re-select the object');
    expect(controller.caretRunId, isNotNull);
  });
}
