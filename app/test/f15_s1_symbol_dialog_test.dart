import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/recent_symbols.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';
import 'package:tutuaword/ui/symbol_dialog.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F15.S1 Symbol dialog', () {
    testWidgets('I-F15-S1-insert-copyright inserts © at caret', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      expect(engine.text, isEmpty);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('insert_symbol')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('insert_symbol')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('symbol_dialog')), findsOneWidget);

      await tester.tap(find.byKey(const Key('symbol_cell_copyright')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, '©');
      expect(controller.sessionController.statusText, contains('Symbol inserted'));
    });

    testWidgets('I-F15-S1-insert-euro-from-currency-category', (tester) async {
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
      await tester.tap(find.text('Currency').last);
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('symbol_cell_euro')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, '€');
    });

    testWidgets('I-F15-S1-symbol-dialog-dismisses-without-insert', (tester) async {
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

      await tester.tap(find.byKey(const Key('symbol_cancel')));
      await tester.pumpAndSettle();

      expect(engine.text, isEmpty);
    });

    testWidgets('I-F15-S1-symbol-dialog-direct-insert-api', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.insertSymbolCharacter('±');
      await settleEngineStyle(tester);

      expect(engine.text, '±');
    });
  });

  testWidgets('SymbolDialog returns selected character', (tester) async {
    String? result;
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) {
            return TextButton(
              onPressed: () async {
                result = await SymbolDialog.show(context, recentStore: RecentSymbolsStore());
              },
              child: const Text('Open'),
            );
          },
        ),
      ),
    );

    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('symbol_cell_registered')));
    await tester.pumpAndSettle();

    expect(result, '®');
  });
}
