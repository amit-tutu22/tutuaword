import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F19.S3 Bookmarks and hyperlinks', () {
    testWidgets('I-F19-S3-insert-hyperlink-from-insert-ribbon', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Hello');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('insert_hyperlink')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('insert_hyperlink')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('hyperlink_dialog')), findsOneWidget);
      await tester.enterText(
        find.byKey(const Key('hyperlink_text_field')),
        'Docs',
      );
      await tester.enterText(
        find.byKey(const Key('hyperlink_url_field')),
        'https://example.com',
      );
      await tester.tap(find.byKey(const Key('hyperlink_insert_button')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('Docs'));
      expect(controller.sessionController.statusText, contains('Hyperlink inserted'));
    });

    testWidgets('I-F19-S3-insert-bookmark-from-insert-ribbon', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Introduction');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('insert_bookmark_insert_tab')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('insert_bookmark_insert_tab')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('bookmark_name_dialog')), findsOneWidget);
      await tester.enterText(
        find.byKey(const Key('bookmark_name_field')),
        'MyBookmark',
      );
      await tester.tap(find.byKey(const Key('bookmark_insert_button')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(controller.sessionController.statusText, contains('Bookmark inserted'));
    });

    testWidgets('InsertTab wires Link and Bookmark buttons', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: InsertTab(controller: controller),
          ),
        ),
      );

      expect(find.byKey(const Key('insert_hyperlink')), findsOneWidget);
      expect(find.byKey(const Key('insert_bookmark_insert_tab')), findsOneWidget);
    });
  });
}
