import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/doc_range.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/ui/status_bar.dart';

import 'editor_test_helpers.dart';

/// Basic caret / keyboard regression suite.
///
/// Covers Space, Tab, Enter, arrows, Backspace, and Delete through both the
/// direct controller APIs (mock engine) and the desktop Focus key path
/// ([GlyphEditorSurface]). These must not regress — they are core editor UX.
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('caret keyboard — direct (mock engine)', () {
    test('Space / letters advance caret and document text', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'hi there');

      expect(controller.documentText, 'hi there');
      expect(controller.caretOffset, 8);
      expect(controller.hasGlyphSelection, isFalse);
    });

    test('Tab inserts a tab character', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'a');
      await controller.insertGlyphCharacter('\t');
      await typeTextDirect(controller, 'b');

      expect(controller.documentText, 'a\tb');
      expect(controller.caretOffset, 3);
    });

    test('Enter after typed line does not duplicate text on new line', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.ensureGlyphCaret();

      await typeTextDirect(controller, 'dgdsdg');
      expect(controller.caretOffset, 6);
      await controller.insertGlyphParagraphBreak();
      await controller.ensureLayoutReady();

      expect(controller.documentText, 'dgdsdg\n');
      expect(controller.documentText.split('\n').where((l) => l == 'dgdsdg').length, 1);

      await controller.insertGlyphParagraphBreak();
      await controller.ensureLayoutReady();
      expect(controller.documentText, 'dgdsdg\n\n');
      expect(controller.documentText.split('\n').where((l) => l == 'dgdsdg').length, 1);
    });

    test('Enter serializes and does not drop held breaks', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.ensureGlyphCaret();

      // Fire overlapping Enters the way the Focus surface does (unawaited).
      final pending = <Future<void>>[
        for (var i = 0; i < 5; i++) controller.insertGlyphParagraphBreak(),
      ];
      await Future.wait(pending);
      await controller.ensureLayoutReady();

      expect(controller.caretRunId, isNotNull);
      expect(controller.documentText, '\n\n\n\n\n');
      expect(controller.caretOffset, 0);
      expect(controller.caretPage, 0);
    });

    test('arrows left/right move caret within typed text', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'abcd');
      expect(controller.caretOffset, 4);

      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowLeft);
      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowLeft);
      expect(controller.caretOffset, 2);

      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowRight);
      expect(controller.caretOffset, 3);
    });

    test('arrow down near page bottom advances to next page', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 2);
      controller.ensureGlyphCaret();

      // Place caret on page 0 near the content bottom so Down crosses pages.
      final bottomY = controller.pageHeight - controller.marginBottom - 4;
      controller.hitTestAt(0, controller.marginLeft + 20, bottomY);
      // Mock geometry ignores Y — force a bottom-of-page caret for the arrow path.
      controller.selectionController.setCaret(
        controller.caretRunId!,
        controller.caretOffset,
        geometry: CaretGeometry(
          x: controller.marginLeft + 20,
          y: bottomY,
          height: 14,
        ),
        page: 0,
      );

      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowDown);

      expect(controller.caretPage, 1);
    });

    test('arrow up near page top returns to previous page', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 2);
      controller.ensureGlyphCaret();

      final topY = controller.marginTop + 4;
      controller.hitTestAt(1, controller.marginLeft + 20, topY);
      controller.selectionController.setCaret(
        controller.caretRunId!,
        controller.caretOffset,
        geometry: CaretGeometry(
          x: controller.marginLeft + 20,
          y: topY,
          height: 14,
        ),
        page: 1,
      );

      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowUp);

      expect(controller.caretPage, 0);
    });

    test('statusPage follows caret when scroll visible page differs', () {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 2);
      controller.ensureGlyphCaret();
      controller.selectionController.setCaret(
        controller.caretRunId!,
        controller.caretOffset,
        page: 1,
      );
      controller.setVisiblePage(0);
      expect(controller.currentPage, 0);
      expect(controller.caretPage, 1);
      expect(controller.statusPage, 1);
    });

    test('arrow down mid-page does not falsely jump pages', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 2);

      await typeTextDirect(controller, 'abc');
      expect(controller.caretPage, 0);

      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowDown);
      expect(controller.caretPage, 0);
    });

    test('arrow down requests scroll so the caret stays visible', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 2);
      controller.ensureGlyphCaret();

      controller.selectionController.setCaret(
        controller.caretRunId!,
        controller.caretOffset,
        geometry: CaretGeometry(
          x: controller.marginLeft + 20,
          y: 640,
          height: 14,
        ),
        page: 0,
      );
      controller.ensureCaretVisible();

      final request = controller.view.caretScrollRequest;
      expect(request, isNotNull);
      expect(request!.page, 0);
      expect(request.y, 640);
    });

    test('horizontal typing does not request caret scroll', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'abc');
      expect(controller.view.caretScrollRequest, isNull);
    });

    test('same-page arrow does not call engine setCurrentPageIndex', () async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 2);
      controller.ensureGlyphCaret();
      engine.setCurrentPageIndexCount = 0;

      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowLeft);

      expect(engine.setCurrentPageIndexCount, 0);
    });

    test('Backspace and Delete edit around the caret', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'abcd');
      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowLeft);
      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowLeft);
      // caret between b|c
      await controller.deleteGlyphForward();
      expect(controller.documentText, 'abd');
      await controller.deleteGlyphBackward();
      expect(controller.documentText, 'ad');
      expect(controller.caretOffset, 1);
    });

    test('shift+arrow builds a non-empty selection', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'xyz');
      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowLeft);
      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowLeft);

      HardwareKeyboard.instance.handleKeyEvent(
        const KeyDownEvent(
          physicalKey: PhysicalKeyboardKey.shiftLeft,
          logicalKey: LogicalKeyboardKey.shiftLeft,
          timeStamp: Duration.zero,
        ),
      );
      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowRight);
      HardwareKeyboard.instance.handleKeyEvent(
        const KeyUpEvent(
          physicalKey: PhysicalKeyboardKey.shiftLeft,
          logicalKey: LogicalKeyboardKey.shiftLeft,
          timeStamp: Duration.zero,
        ),
      );

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText, 'y');
    });
  });

  group('caret keyboard — widget Focus surface', () {
    Future<EditorController> pumpEditor(WidgetTester tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await pumpTestDocumentView(tester, controller);
      // Page surface can sit below the 600px test viewport; tap a point that
      // still hits the editor chrome/focus path inside the window.
      final surface = find.byType(GlyphEditorSurface).first;
      final box = tester.renderObject(surface) as RenderBox;
      final topLeft = box.localToGlobal(Offset.zero);
      final tap = Offset(
        topLeft.dx + 40,
        topLeft.dy.clamp(0, 500) + 20,
      );
      await tester.tapAt(tap);
      await tester.pump();
      controller.ensureGlyphCaret();
      return controller;
    }

    testWidgets('Space / letters via keyboard update document', (tester) async {
      final controller = await pumpEditor(tester);

      await typeText(tester, controller, 'hi there');
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.documentText, 'hi there');
      expect(controller.caretOffset, 8);
    });

    testWidgets('Tab key inserts tab', (tester) async {
      final controller = await pumpEditor(tester);

      await typeText(tester, controller, 'a');
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await typeText(tester, controller, 'b');
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.documentText, 'a\tb');
    });

    testWidgets('Enter key does not leave caret without a run', (tester) async {
      final controller = await pumpEditor(tester);

      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.caretRunId, isNotNull);
      expect(controller.documentText, '\n\n\n');
      expect(controller.caretOffset, 0);
    });

    testWidgets('arrow keys move caret after typing', (tester) async {
      final controller = await pumpEditor(tester);

      await typeText(tester, controller, 'abcd');
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.caretOffset, 2);

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.caretOffset, 3);
    });

    testWidgets('Backspace removes previous character', (tester) async {
      final controller = await pumpEditor(tester);

      await typeText(tester, controller, 'ab');
      await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.documentText, 'a');
      expect(controller.caretOffset, 1);
    });

    testWidgets('Delete removes next character', (tester) async {
      final controller = await pumpEditor(tester);

      await typeText(tester, controller, 'ab');
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.delete);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.documentText, 'a');
      // Forward delete keeps the caret at the deletion index (end of "a").
      expect(controller.caretOffset, 1);
    });

    testWidgets('caret below viewport scrolls the page down', (tester) async {
      await tester.binding.setSurfaceSize(const Size(800, 480));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 2);
      await pumpTestDocumentView(tester, controller);

      final bottomY = controller.pageHeight - controller.marginBottom - 4;
      controller.hitTestAt(0, controller.marginLeft + 20, bottomY);
      controller.selectionController.setCaret(
        controller.caretRunId!,
        controller.caretOffset,
        geometry: CaretGeometry(
          x: controller.marginLeft + 20,
          y: bottomY,
          height: 14,
        ),
        page: 0,
      );
      controller.ensureCaretVisible();
      await tester.pump();
      await tester.pumpAndSettle();

      final pageList = find.byKey(const ValueKey('document-page-list'));
      expect(pageList, findsOneWidget);
      final listView = tester.widget<ListView>(pageList);
      expect(
        listView.controller?.offset ?? 0,
        greaterThan(0),
        reason: 'canvas should scroll down so the caret stays visible',
      );
    });

    testWidgets('arrow down onto next page scrolls canvas', (tester) async {
      await tester.binding.setSurfaceSize(const Size(800, 480));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 2);
      await pumpTestDocumentView(tester, controller);

      final bottomY = controller.pageHeight - controller.marginBottom - 4;
      controller.hitTestAt(0, controller.marginLeft + 20, bottomY);
      controller.selectionController.setCaret(
        controller.caretRunId!,
        controller.caretOffset,
        geometry: CaretGeometry(
          x: controller.marginLeft + 20,
          y: bottomY,
          height: 14,
        ),
        page: 0,
      );

      controller.moveGlyphCaretByArrow(LogicalKeyboardKey.arrowDown);
      expect(controller.caretPage, 1);
      await tester.pump();
      await tester.pumpAndSettle();

      final pageList = find.byKey(const ValueKey('document-page-list'));
      final listView = tester.widget<ListView>(pageList);
      expect(
        listView.controller?.offset ?? 0,
        greaterThan(0),
        reason: 'canvas should follow the caret onto the next page',
      );
    });

    testWidgets('status bar shows caret page when scroll still shows page 1 sliver',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 2);
      controller.ensureGlyphCaret();
      controller.selectionController.setCaret(
        controller.caretRunId!,
        controller.caretOffset,
        geometry: CaretGeometry(
          x: controller.marginLeft + 20,
          y: controller.marginTop + 20,
          height: 14,
        ),
        page: 1,
      );
      controller.setVisiblePage(0);
      expect(controller.statusPage, 1);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: WordStatusBar(controller: controller)),
        ),
      );

      expect(find.textContaining('Page 2 of 2'), findsOneWidget);
    });

    testWidgets('already-visible caret does not scroll the page', (tester) async {
      await tester.binding.setSurfaceSize(const Size(800, 480));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await pumpTestDocumentView(tester, controller);

      final pageList = find.byKey(const ValueKey('document-page-list'));
      final listView = tester.widget<ListView>(pageList);
      expect(listView.controller?.offset ?? 0, 0);

      controller.ensureGlyphCaret();
      controller.ensureCaretVisible();
      await tester.pump();
      await tester.pumpAndSettle();

      expect(
        tester.widget<ListView>(pageList).controller?.offset ?? 0,
        0,
        reason: 'caret at the top of the page is already visible',
      );
    });
  });

  group('cross-page selection rects helper', () {
    test('projectSelectionOntoPage covers focus page from top', () {
      final start = CaretGeometry(x: 90, y: 400, height: 14);
      final end = CaretGeometry(x: 120, y: 96, height: 14);
      final projected = DocumentEngineSelection.projectSelectionOntoPage(
        page: 1,
        startPage: 0,
        startGeom: start,
        endPage: 1,
        endGeom: end,
      );
      expect(projected, isNotNull);
      expect(projected!.$1, 0);
      expect(projected.$2, 0);
      expect(projected.$3, end.x);
      expect(projected.$4, end.y);
    });

    test('selectionRectsForPage returns rects on caret page after select', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'hello');
      await controller.selectAll();

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectionRectsForPage(0), isNotEmpty);
      expect(controller.selectionRectsForPage(0), controller.selectionRects);
    });

    test('DocRange cross-run still projects for painting', () {
      final range = DocRange(
        anchor: const DocPosition(runId: 'a', offset: 0),
        focus: const DocPosition(runId: 'b', offset: 1),
        page: 1,
      );
      expect(range.isCollapsed, isFalse);
      expect(range.page, 1);
    });
  });

  group('caret reveal scroll math', () {
    test('scrolls down when caret is below the viewport', () {
      final target = scrollOffsetToRevealCaret(
        caretTop: 620,
        caretBottom: 634,
        viewportTop: 0,
        viewportHeight: 480,
        minExtent: 0,
        maxExtent: 2000,
      );
      expect(target, greaterThan(0));
      expect(target, 634 + 16 - 480);
    });

    test('does not scroll when caret is already visible', () {
      expect(
        scrollOffsetToRevealCaret(
          caretTop: 120,
          caretBottom: 134,
          viewportTop: 0,
          viewportHeight: 480,
          minExtent: 0,
          maxExtent: 2000,
        ),
        isNull,
      );
    });

    test('scrolls up when caret is above the viewport', () {
      final target = scrollOffsetToRevealCaret(
        caretTop: 40,
        caretBottom: 54,
        viewportTop: 200,
        viewportHeight: 480,
        minExtent: 0,
        maxExtent: 2000,
      );
      expect(target, 40 - 16);
    });
  });
}
