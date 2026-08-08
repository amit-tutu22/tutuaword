import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F09.S1 Table insert', () {
    testWidgets('I-F09-S1-insert-3x3 inserts table from Insert tab', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      expect(engine.hasTable, isFalse);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.tap(find.text('Table'));
      await settleEngineStyle(tester);

      expect(controller.sessionController.statusText, contains('Table inserted (3×3)'));
      expect(engine.hasTable, isTrue);
      expect(engine.tableRows, 3);
      expect(engine.tableCols, 3);

      final saved = engine.saveDocumentBytes();
      expect(saved, isNotNull);
      engine.newDocument();
      expect(engine.hasTable, isFalse);
      engine.openDocumentBytes(saved!);
      expect(engine.tableRows, 3);
      expect(engine.tableCols, 3);
    });
  });
}
