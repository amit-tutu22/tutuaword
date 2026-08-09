import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F15.S2 Emoji and math symbol dialog', () {
    testWidgets('I-F15-S2-insert-emoji-from-picker', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('insert_symbol')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('insert_symbol')));
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('symbol_category_dropdown')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Emoji').last);
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('symbol_grid_emoji')), findsOneWidget);

      await tester.tap(find.byKey(const Key('symbol_cell_thumbs_up')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, '👍');
      expect(controller.sessionController.statusText, contains('Symbol inserted'));
    });

    testWidgets('I-F15-S2-insert-greek-alpha-from-math-subset', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('insert_symbol')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('insert_symbol')));
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('symbol_category_dropdown')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Greek & Advanced').last);
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('symbol_cell_alpha')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, 'α');
    });

    testWidgets('I-F15-S2-emoji-and-math-via-direct-api', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.insertSymbolCharacter('😀');
      await controller.insertSymbolCharacter('α');
      await settleEngineStyle(tester);

      expect(engine.text, '😀α');
    });
  });
}
