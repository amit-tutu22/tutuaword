import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/references_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F16.S4 Index and cross-references', () {
    testWidgets('I-F16-S4-insert-cross-ref-from-references-ribbon', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Introduction');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReferencesTab(controller: controller));
      await tester.tap(find.byKey(const Key('insert_bookmark')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('bookmark_insert_button')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);
      await tester.scrollUntilVisible(
        find.byKey(const Key('insert_cross_reference')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('insert_cross_reference')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('Introduction'));
      expect(controller.sessionController.statusText, contains('Cross-reference inserted'));
    });

    testWidgets('I-F16-S4-insert-index-from-references-ribbon', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Introduction');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReferencesTab(controller: controller));
      await tester.tap(find.byKey(const Key('insert_bookmark')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('bookmark_insert_button')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);
      await tester.scrollUntilVisible(
        find.byKey(const Key('insert_index')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('insert_index')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('Index'));
      expect(engine.text, contains('Introduction'));
      expect(controller.sessionController.statusText, contains('Index inserted'));
    });

    testWidgets('I-F16-S4-insert-cross-ref-direct-api', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Introduction');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return Column(
                children: [
                  TextButton(
                    onPressed: () => controller.insertBookmark(context),
                    child: const Text('Bookmark'),
                  ),
                  TextButton(
                    onPressed: () => controller.insertCrossReference(context),
                    child: const Text('CrossRef'),
                  ),
                ],
              );
            },
          ),
        ),
      );

      await tester.tap(find.text('Bookmark'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('bookmark_insert_button')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);
      await tester.tap(find.text('CrossRef'));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('Introduction'));
    });
  });

  testWidgets('ReferencesTab wires bookmark, cross-ref, and index buttons', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ReferencesTab(controller: controller),
        ),
      ),
    );

    expect(find.byKey(const Key('insert_bookmark')), findsOneWidget);
    expect(find.byKey(const Key('insert_cross_reference')), findsOneWidget);
    expect(find.byKey(const Key('insert_index')), findsOneWidget);
  });
}
