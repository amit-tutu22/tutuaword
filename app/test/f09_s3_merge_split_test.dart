import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F09.S3 Table merge and split', () {
    testWidgets('I-F09-S3-merge-cells merges from Layout tab', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.insertTable();
      await settleEngineStyle(tester);
      expect(engine.tableLeadColspan, 1);

      await tester.binding.setSurfaceSize(const Size(1400, 120));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await pumpRibbonTab(
        tester,
        LayoutTab(controller: controller),
        size: const Size(1400, 120),
      );

      await tester.tap(find.byIcon(Icons.merge_type));
      await settleEngineStyle(tester);

      expect(engine.tableLeadColspan, 2);
      expect(controller.sessionController.statusText, contains('Cells merged'));
    });

    testWidgets('split cell restores colspan', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.insertTable();
      await controller.mergeTableCells();
      await settleEngineStyle(tester);
      expect(engine.tableLeadColspan, 2);

      await controller.splitTableCell();
      await settleEngineStyle(tester);

      expect(engine.tableLeadColspan, 1);
      expect(controller.sessionController.statusText, contains('Cell split'));
    });
  });
}
