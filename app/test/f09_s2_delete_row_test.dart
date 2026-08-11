import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F09.S2 Table row/column delete', () {
    testWidgets('I-F09-S2-delete-row-ui removes row from Layout tab', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.insertTable();
      await settleEngineStyle(tester);
      expect(engine.tableRows, 3);

      await tester.binding.setSurfaceSize(const Size(1200, 120));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await pumpRibbonTab(
        tester,
        LayoutTab(controller: controller),
        size: const Size(1200, 120),
      );

      await tester.tap(find.byIcon(Icons.table_rows));
      await settleEngineStyle(tester);

      expect(engine.tableRows, 2);
      expect(controller.sessionController.statusText, contains('Table row deleted'));
    });

    testWidgets('delete column via controller', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.insertTable();
      await settleEngineStyle(tester);
      expect(engine.tableCols, 3);

      await controller.deleteTableColumn();
      await settleEngineStyle(tester);

      expect(engine.tableCols, 2);
      expect(controller.sessionController.statusText, contains('Table column deleted'));
    });
  });
}
