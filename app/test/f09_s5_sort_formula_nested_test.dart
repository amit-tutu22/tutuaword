import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F09.S5 Sort, formula, nested', () {
    testWidgets('I-F09-S5-sort-table sorts from Layout tab', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.insertTable();
      await settleEngineStyle(tester);

      await tester.binding.setSurfaceSize(const Size(1800, 160));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await pumpRibbonTab(
        tester,
        LayoutTab(controller: controller),
        size: const Size(1800, 160),
      );

      await tester.tap(find.byKey(const Key('table_sort_asc')));
      await settleEngineStyle(tester);

      expect(engine.tableSortedAscending, isTrue);
      expect(controller.sessionController.statusText, contains('sorted'));
    });

    testWidgets('sum formula and nested table buttons', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.insertTable();
      await settleEngineStyle(tester);

      await controller.insertTableSumFormula();
      await settleEngineStyle(tester);
      expect(engine.tableSumFieldInserted, isTrue);

      await controller.insertNestedTable();
      await settleEngineStyle(tester);
      expect(engine.nestedTableCount, 1);
    });
  });
}
