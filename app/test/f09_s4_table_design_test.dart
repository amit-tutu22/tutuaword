import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';
import 'package:tutuaword/ui/table_design_dialog.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F09.S4 Table design', () {
    testWidgets('I-F09-S4-table-design-ui applies border and shading', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.insertTable();
      await settleEngineStyle(tester);

      await tester.binding.setSurfaceSize(const Size(1600, 900));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await pumpRibbonTab(
        tester,
        LayoutTab(controller: controller),
        size: const Size(1600, 120),
      );

      await tester.tap(find.byKey(const Key('table_design_button')));
      await tester.pumpAndSettle();

      expect(find.text('Table Design'), findsOneWidget);

      await tester.tap(find.byKey(const Key('table_border_enabled')));
      await tester.pumpAndSettle();

      await tester.enterText(find.byKey(const Key('table_border_width')), '2');
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('table_design_ok')));
      await settleEngineStyle(tester);

      expect(engine.tableBorderWidth, 2);
      expect(controller.sessionController.statusText, contains('Table design applied'));
    });

    testWidgets('autofit to window from dialog', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.insertTable();
      await settleEngineStyle(tester);

      await controller.applyTableDesign(const TableDesignValues(autofitToWindow: true));
      await settleEngineStyle(tester);

      expect(engine.tableAutofitApplied, isTrue);
      expect(engine.tableColumnWidths, isNotNull);
      expect(engine.tableColumnWidths!.length, 3);
    });

    testWidgets('column width resize via dialog values', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.insertTable();
      await settleEngineStyle(tester);

      await controller.applyTableDesign(
        const TableDesignValues(columnWidth: 140),
      );
      await settleEngineStyle(tester);

      expect(engine.tableColumnWidths, isNotNull);
      expect(engine.tableColumnWidths!.every((w) => (w - 140).abs() < 0.01), isTrue);
    });
  });
}
