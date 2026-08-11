import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';

import 'editor_test_helpers.dart';

void main() {
  testWidgets('I-F11-S3-insert-text-box commits via engine', (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    await controller.insertTextBox();
    await settleEngineStyle(tester);

    expect(controller.sessionController.statusText, contains('Text box'));
  });

  testWidgets('I-F11-S3-insert-word-art commits via engine', (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    await controller.insertWordArt('Glow');
    await settleEngineStyle(tester);

    expect(controller.sessionController.statusText, contains('WordArt'));
  });
}
