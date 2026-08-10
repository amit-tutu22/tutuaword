import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/bridge/outline_entry.dart';
import 'package:tutuaword/editor/navigation_pane.dart';
import 'package:tutuaword/editor/outline_navigator.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F19.S2 Outline view', () {
    test('buildOutlineTree nests by outline level', () {
      const entries = [
        DocumentOutlineEntry(
          paragraphId: 'p1',
          level: 0,
          text: 'Chapter',
          runId: 'r1',
          page: 0,
        ),
        DocumentOutlineEntry(
          paragraphId: 'p2',
          level: 1,
          text: 'Section A',
          runId: 'r2',
          page: 0,
        ),
        DocumentOutlineEntry(
          paragraphId: 'p3',
          level: 1,
          text: 'Section B',
          runId: 'r3',
          page: 1,
        ),
        DocumentOutlineEntry(
          paragraphId: 'p4',
          level: 2,
          text: 'Detail',
          runId: 'r4',
          page: 1,
        ),
        DocumentOutlineEntry(
          paragraphId: 'p5',
          level: 0,
          text: 'Appendix',
          runId: 'r5',
          page: 2,
        ),
      ];

      final roots = buildOutlineTree(entries);
      expect(roots, hasLength(2));
      expect(roots[0].entry.text, 'Chapter');
      expect(roots[0].children, hasLength(2));
      expect(roots[0].children[0].entry.text, 'Section A');
      expect(roots[0].children[1].entry.text, 'Section B');
      expect(roots[0].children[1].children, hasLength(1));
      expect(roots[0].children[1].children.first.entry.text, 'Detail');
      expect(roots[1].entry.text, 'Appendix');
      expect(roots[1].children, isEmpty);
    });

    testWidgets('I-F19-S2-outline-tree shows nested headings and collapses', (tester) async {
      final engine = MockDocumentEngine();
      engine.setOutlineEntriesForTest([
        {
          'paragraph_id': 'p-h1',
          'level': 0,
          'text': 'Chapter One',
          'run_id': 'run-h1',
          'page': 0,
        },
        {
          'paragraph_id': 'p-h2',
          'level': 1,
          'text': 'Section Nested',
          'run_id': 'run-h2',
          'page': 1,
        },
      ]);

      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 2);
      controller.showNavigationOutline();

      await tester.binding.setSurfaceSize(const Size(900, 700));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await pumpTestDocumentView(tester, controller);
      await tester.pumpAndSettle();

      expect(find.byType(NavigationPane), findsOneWidget);
      expect(find.byType(OutlineNavigator), findsOneWidget);
      expect(find.text('Chapter One'), findsOneWidget);
      expect(find.text('Section Nested'), findsOneWidget);

      await tester.tap(find.byTooltip('Collapse'));
      await tester.pumpAndSettle();
      expect(find.text('Section Nested'), findsNothing);

      await tester.tap(find.byTooltip('Expand'));
      await tester.pumpAndSettle();
      expect(find.text('Section Nested'), findsOneWidget);
    });

    testWidgets('I-F19-S2-outline-jump click heading scrolls to page', (tester) async {
      final engine = MockDocumentEngine();
      engine.setOutlineEntriesForTest([
        {
          'paragraph_id': 'p-a',
          'level': 0,
          'text': 'Intro',
          'run_id': engine.defaultRunId,
          'page': 0,
        },
        {
          'paragraph_id': 'p-b',
          'level': 0,
          'text': 'Later Chapter',
          'run_id': 'run-later',
          'page': 2,
        },
      ]);

      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 3);
      controller.showNavigationOutline();

      await tester.binding.setSurfaceSize(const Size(900, 700));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await pumpTestDocumentView(tester, controller);
      await tester.pumpAndSettle();

      expect(controller.currentPage, 0);
      await tester.tap(find.text('Later Chapter'));
      await tester.pump();
      await tester.pumpAndSettle();

      expect(controller.currentPage, 2);
      expect(controller.caretRunId, 'run-later');
      expect(controller.sessionController.statusText, contains('Later Chapter'));

      final pageList = find.byKey(const ValueKey('document-page-list'));
      final listView = tester.widget<ListView>(pageList);
      expect(listView.controller?.offset ?? 0, greaterThan(0));
    });

    testWidgets('showNavigationOutline opens pane on Outline tab', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      expect(controller.showNavigationPane, isFalse);
      controller.showNavigationOutline();
      expect(controller.showNavigationPane, isTrue);

      await pumpTestDocumentView(tester, controller);
      await tester.pumpAndSettle();

      expect(find.byType(OutlineNavigator), findsOneWidget);
      expect(
        find.text('No headings or outline-numbered paragraphs.'),
        findsOneWidget,
      );
    });
  });
}
