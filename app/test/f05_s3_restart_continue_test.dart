import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F05.S3 Restart and continue numbering', () {
    testWidgets('I-F05-S3-restart sets num_restart on list paragraph', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Item');
      controller.applyNumberedList();
      await controller.ensureLayoutReady();

      controller.restartNumbering();
      await controller.ensureLayoutReady();

      expect(mockEngineParaFormat(engine)['num_restart'], isTrue);
      expect(mockEngineNumbering(engine)?['numbering_id'], 2);
    });

    testWidgets('I-F05-S3-continue clears num_restart', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Item');
      controller.applyNumberedList();
      controller.restartNumbering();
      await controller.ensureLayoutReady();
      expect(mockEngineParaFormat(engine)['num_restart'], isTrue);

      controller.continueNumbering();
      await controller.ensureLayoutReady();

      expect(mockEngineParaFormat(engine).containsKey('num_restart'), isFalse);
    });
  });
}
