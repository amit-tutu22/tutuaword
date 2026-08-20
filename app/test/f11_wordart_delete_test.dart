import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/shape_hit_test.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('I-F11-backspace-deletes-char-inside-wordart-text', (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    await typeTextDirect(controller, 'lhWordArt');
    expect(engine.text, 'lhWordArt');
    expect(controller.caretOffset, 9);

    // Move caret between d and A (offset 2 after "lh").
    controller.selectionController.setCaret(
      controller.caretRunId!,
      2,
      geometry: CaretGeometry(x: 100, y: 100, height: 14),
      page: 0,
    );

    await controller.deleteGlyphBackward();
    await settleEngineStyle(tester);

    expect(engine.text, 'lWordArt');
    expect(controller.caretOffset, 1);
  });

  testWidgets('I-F11-object-select-hides-caret-then-backspace-deletes-object',
      (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    await typeTextDirect(controller, 'keep');
    controller.selectionController.setCaret(
      controller.caretRunId!,
      2,
      geometry: CaretGeometry(x: 100, y: 100, height: 14),
      page: 0,
    );

    controller.selectDiagram(
      0,
      const ShapeBounds(
        shapeId: 'wordart-1',
        index: 0,
        rect: Rect.fromLTWH(72, 72, 220, 72),
      ),
    );
    expect(controller.hasSelectedDiagram, isTrue);
    expect(controller.caretGeometry, isNull);

    await controller.deleteGlyphBackward();
    await settleEngineStyle(tester);

    expect(controller.hasSelectedDiagram, isFalse);
    expect(controller.sessionController.statusText, contains('deleted'));
  });
}
