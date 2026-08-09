import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/shape_hit_test.dart';

import 'editor_test_helpers.dart';

void main() {
  testWidgets('I-F11-delete-selected-object removes diagram selection', (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    controller.selectDiagram(
      0,
      const ShapeBounds(
        shapeId: 'shape-1',
        index: 0,
        rect: Rect.fromLTWH(10, 10, 100, 80),
      ),
    );
    expect(controller.hasSelectedDiagram, isTrue);

    final ok = await controller.deleteSelectedObject();
    await settleEngineStyle(tester);

    expect(ok, isTrue);
    expect(controller.hasSelectedDiagram, isFalse);
    expect(controller.sessionController.statusText, contains('deleted'));
  });
}
