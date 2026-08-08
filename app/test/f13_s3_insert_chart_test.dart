import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';

import 'editor_test_helpers.dart';

void main() {
  testWidgets('I-F13-S3-insert-chart commits via engine', (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    await controller.insertChart(chartType: EditorController.chartColumn);
    await settleEngineStyle(tester);

    expect(controller.sessionController.statusText, contains('Column chart'));
  });
}
