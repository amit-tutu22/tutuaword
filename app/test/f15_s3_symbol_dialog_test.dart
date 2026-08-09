import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/recent_symbols.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';
import 'package:tutuaword/ui/symbol_dialog.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F15.S3 Recent symbols dialog', () {
    testWidgets('I-F15-S3-recent-row-after-insert', (tester) async {
      final recents = RecentSymbolsStore();
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(
        engine: engine,
        recentSymbols: recents,
      );
      addTearDown(controller.dispose);

      await controller.insertSymbolCharacter('©');
      await settleEngineStyle(tester);
      expect(recents.entries().map((e) => e.character), ['©']);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('insert_symbol')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('insert_symbol')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('symbol_recent_row')), findsOneWidget);
      expect(find.byKey(const Key('symbol_recent_copyright')), findsOneWidget);
      expect(find.text('Recently Used'), findsOneWidget);
    });

    testWidgets('I-F15-S3-insert-from-recent-row', (tester) async {
      final recents = RecentSymbolsStore();
      recents.recordId('copyright');
      recents.recordId('euro');

      final engine = MockDocumentEngine();
      final controller = createTestEditorController(
        engine: engine,
        recentSymbols: recents,
      );
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('insert_symbol')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('insert_symbol')));
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('symbol_recent_euro')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, '€');
      expect(recents.ids.first, 'euro');
    });

    testWidgets('I-F15-S3-reinsert-bumps-mru-order', (tester) async {
      final recents = RecentSymbolsStore();
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(
        engine: engine,
        recentSymbols: recents,
      );
      addTearDown(controller.dispose);

      await controller.insertSymbolCharacter('©');
      await controller.insertSymbolCharacter('€');
      await controller.insertSymbolCharacter('©');
      await settleEngineStyle(tester);

      expect(recents.ids, ['copyright', 'euro']);
      expect(engine.text, '©€©');
    });
  });

  testWidgets('SymbolDialog defaults to Recent subset when populated', (tester) async {
    final recents = RecentSymbolsStore()..recordId('registered');

    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) {
            return TextButton(
              onPressed: () => SymbolDialog.show(context, recentStore: recents),
              child: const Text('Open'),
            );
          },
        ),
      ),
    );

    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('symbol_grid_recent')), findsOneWidget);
    expect(find.byKey(const Key('symbol_cell_registered')), findsOneWidget);
  });
}
