import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/goto_dialog.dart';
import 'package:tutuaword/ui/ribbon_tabs/view_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F19.S4 Go To dialog', () {
    test('jumpToPage from Go To clamps', () {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 3);

      controller.jumpToPage(1);
      expect(controller.currentPage, 1);
      expect(controller.view.takeScrollRequest(), 1);
    });

    testWidgets('I-F19-S4-goto-page', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Hello');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 5);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                key: const Key('open_goto'),
                onPressed: () => controller.openGoToDialog(context),
                child: const Text('Go'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('open_goto')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('goto_dialog')), findsOneWidget);

      await tester.enterText(find.byKey(const Key('goto_page_field')), '3');
      await tester.tap(find.byKey(const Key('goto_go_button')));
      await tester.pumpAndSettle();

      expect(controller.currentPage, 2);
      expect(controller.view.takeScrollRequest(), 2);
      expect(controller.sessionController.statusText, contains('Go To page 3'));
    });

    testWidgets('I-F19-S4-goto-bookmark', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Hello');
      engine.setBookmarksForTest([
        {
          'name': 'SectionRef',
          'run_id': 'run-bm',
          'paragraph_id': 'p-bm',
          'page': 2,
        },
      ]);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 3);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                key: const Key('open_goto'),
                onPressed: () => controller.openGoToDialog(context),
                child: const Text('Go'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('open_goto')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('goto_target_bookmark')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('goto_bookmark_list')), findsOneWidget);
      await tester.tap(find.byKey(const Key('goto_bookmark_SectionRef')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('goto_go_button')));
      await tester.pumpAndSettle();

      expect(controller.currentPage, 2);
      expect(controller.caretRunId, 'run-bm');
      expect(controller.sessionController.statusText, contains('Bookmark: SectionRef'));
    });

    testWidgets('I-F19-S4-goto-heading', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Hello');
      engine.setOutlineEntriesForTest([
        {
          'paragraph_id': 'p-h1',
          'level': 0,
          'text': 'Chapter One',
          'run_id': 'run-h1',
          'page': 3,
        },
      ]);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 4);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                key: const Key('open_goto'),
                onPressed: () => controller.openGoToDialog(context),
                child: const Text('Go'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('open_goto')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('goto_target_heading')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('goto_heading_list')), findsOneWidget);
      await tester.tap(find.byKey(const Key('goto_heading_p-h1')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('goto_go_button')));
      await tester.pumpAndSettle();

      expect(controller.currentPage, 3);
      expect(controller.caretRunId, 'run-h1');
      expect(controller.sessionController.statusText, contains('Outline: Chapter One'));
    });

    testWidgets('ViewTab wires Go To button', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ViewTab(controller: controller));
      expect(find.byKey(const Key('goto_button')), findsOneWidget);
    });

    testWidgets('GoToDialog page target emits zero-based index', (tester) async {
      GoToResult? result;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                onPressed: () async {
                  result = await GoToDialog.show(
                    context,
                    pageCount: 10,
                    currentPage: 0,
                    bookmarks: const [],
                    headings: const [],
                  );
                },
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      await tester.enterText(find.byKey(const Key('goto_page_field')), '4');
      await tester.tap(find.byKey(const Key('goto_go_button')));
      await tester.pumpAndSettle();
      expect(result, isA<GoToPageResult>());
      expect((result as GoToPageResult).pageIndex, 3);
    });
  });
}
