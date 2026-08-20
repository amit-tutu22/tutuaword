import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/shape_hit_test.dart';
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

    testWidgets('I-F09-S5-sort-works-when-table-object-selected', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.insertTable();
      await settleEngineStyle(tester);

      // Object-select the table (first click) — caret visual is cleared.
      controller.selectDiagram(
        0,
        const ShapeBounds(
          shapeId: '00000000-0000-0000-0000-00000000t001',
          index: 0,
          rect: Rect.fromLTWH(72, 72, 240, 120),
        ),
      );
      expect(controller.hasSelectedDiagram, isTrue);

      await controller.sortTableDescending();
      await settleEngineStyle(tester);

      expect(engine.tableSortedAscending, isFalse);
      expect(controller.sessionController.statusText, contains('sorted'));
      expect(controller.hasSelectedDiagram, isFalse);
    });
  });
}
