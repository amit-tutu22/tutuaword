import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F09.S1 Table insert', () {
    testWidgets('I-F09-S1-insert-table picks size from Insert tab grid', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      expect(engine.hasTable, isFalse);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.tap(find.byKey(const Key('insert_table_button')));
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('table_size_cell_2_4')));
      await settleEngineStyle(tester);

      expect(controller.sessionController.statusText, contains('Table inserted (2×4)'));
      expect(engine.hasTable, isTrue);
      expect(engine.tableRows, 2);
      expect(engine.tableCols, 4);

      final saved = engine.saveDocumentBytes();
      expect(saved, isNotNull);
      engine.newDocument();
      expect(engine.hasTable, isFalse);
      engine.openDocumentBytes(saved!);
      expect(engine.tableRows, 2);
      expect(engine.tableCols, 4);
    });

    testWidgets('I-F09-S1-insert-table dismisses without inserting', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.tap(find.byKey(const Key('insert_table_button')));
      await tester.pumpAndSettle();

      await tester.tapAt(const Offset(1, 1));
      await tester.pumpAndSettle();

      expect(engine.hasTable, isFalse);
    });
  });
}
