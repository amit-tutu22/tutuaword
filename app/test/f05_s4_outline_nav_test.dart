import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/navigation_pane.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F05.S4 Outline numbering', () {
    /// I-F05-S4-outline-nav: outline entries appear for numbered lists and headings.
    testWidgets('I-F05-S4-outline-nav shows numbered list in outline', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await typeTextDirect(controller, 'First section');
      controller.applyNumberedList();
      await controller.ensureLayoutReady();

      final outline = mockEngineOutline(engine);
      expect(outline, hasLength(1));
      expect(outline.first['text'], 'First section');
      expect(outline.first['level'], 0);
      expect(outline.first['page'], 0);
    });

    testWidgets('I-F05-S4-outline-nav heading appears in outline', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Chapter');
      controller.applyHeading1();
      await controller.ensureLayoutReady();

      final outline = controller.documentOutline;
      expect(outline, hasLength(1));
      expect(outline.first.text, 'Chapter');
      expect(outline.first.level, 0);
    });

    testWidgets('bullet list does not appear in outline', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Bullet only');
      controller.applyBulletList();
      await controller.ensureLayoutReady();

      expect(mockEngineOutline(engine), isEmpty);
      expect(controller.documentOutline, isEmpty);
    });

    testWidgets('outline tab lists entries and tap jumps caret page', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      controller.toggleNavigationPane();

      await typeTextDirect(controller, 'Jump target');
      controller.applyHeading1();
      await controller.ensureLayoutReady();

      await pumpTestDocumentView(tester, controller);
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Outline'));
      await tester.pumpAndSettle();

      expect(find.text('Jump target'), findsOneWidget);
      await tester.tap(find.text('Jump target'));
      await tester.pumpAndSettle();

      expect(controller.currentPage, 0);
      expect(controller.caretRunId, engine.defaultRunId);
    });

    testWidgets('empty navigation pane shows placeholder on outline tab', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      controller.toggleNavigationPane();

      await pumpTestDocumentView(tester, controller);
      await tester.pumpAndSettle();

      expect(find.byType(NavigationPane), findsOneWidget);
      await tester.tap(find.byTooltip('Outline'));
      await tester.pumpAndSettle();
      expect(find.text('No headings or outline-numbered paragraphs.'), findsOneWidget);
    });
  });
}
