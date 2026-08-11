import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('DocumentView Stage 2 integration', () {
    testWidgets('glyph mode mounts GlyphEditorSurface instead of Start typing placeholder',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);

      expect(find.byType(GlyphEditorSurface), findsWidgets);
      expect(find.text('Start typing…'), findsNothing);
      expect(find.byType(TextField), findsNothing);
    });

    testWidgets('empty glyph snapshot still mounts an editable surface', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(Uint8List(0), preferTextRendering: false);

      await pumpTestDocumentView(tester, controller);

      expect(controller.preferTextRendering, isFalse);
      expect(find.byType(GlyphEditorSurface), findsWidgets);
      expect(find.text('Start typing…'), findsNothing);
    });
  });

  group('Home tab formatting integration', () {
    testWidgets('bold italic underline toggles update controller state', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 120,
              child: HomeTab(controller: controller),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Bold'));
      await tester.pump();
      expect(controller.bold, isTrue);

      await tester.tap(find.byTooltip('Italic'));
      await tester.pump();
      expect(controller.italic, isTrue);

      await tester.tap(find.byTooltip('Underline'));
      await tester.pump();
      expect(controller.underline, isTrue);
    });

    testWidgets('alignment buttons update controller alignment', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 120,
              child: HomeTab(controller: controller),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Center'));
      await tester.pump();
      expect(controller.alignment, TextAlign.center);

      await tester.tap(find.byTooltip('Align Right'));
      await tester.pump();
      expect(controller.alignment, TextAlign.right);
    });
  });

  group('Engine-backed editing integration', () {
    testWidgets('font size applies after typing at run end', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'A');

      controller.setFontSize(24);
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();

      expect(controller.fontSize, 24);
    });

    testWidgets('font family applies after typing at run end', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'A');

      controller.setFontFamily('Georgia');
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();

      expect(controller.fontFamily, 'Georgia');
    });

    testWidgets('glyph surface accepts key events when engine is connected', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      expect(controller.preferTextRendering, isFalse);

      await pumpTestDocumentView(tester, controller);

      expect(find.byType(GlyphEditorSurface), findsWidgets);
      expect(find.text('Start typing…'), findsNothing);

      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();
      await typeText(tester, controller, 'A');
    });

    testWidgets('tab key inserts a tab and advances caret', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'A');

      final afterAOffset = controller.caretOffset;
      final afterACaretX = controller.caretGeometry?.x;
      expect(afterAOffset, greaterThan(0));
      expect(afterACaretX, isNotNull);

      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.caretOffset, greaterThan(afterAOffset));
      final afterTabCaretX = controller.caretGeometry?.x;
      expect(afterTabCaretX, isNotNull);
      expect(controller.documentText.length, greaterThan(1));
    });

    testWidgets('space key inserts and advances caret', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'A');

      final afterAOffset = controller.caretOffset;
      final afterACaretX = controller.caretGeometry?.x;
      expect(afterAOffset, greaterThan(0));
      expect(afterACaretX, isNotNull);

      await tester.sendKeyEvent(LogicalKeyboardKey.space);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.caretOffset, greaterThan(afterAOffset));
      final afterSpaceCaretX = controller.caretGeometry?.x;
      expect(afterSpaceCaretX, isNotNull);
      expect(controller.documentText, contains(' '));
    });

    testWidgets('arrow keys move caret in glyph mode', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'A');

      final afterInsert = controller.caretOffset;
      expect(afterInsert, greaterThan(0));

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.caretOffset, lessThan(afterInsert));

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.caretOffset, afterInsert);
    });

    testWidgets('delete key removes character forward (I-F02-S2-delete-key)', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'hi');

      expect(controller.documentText.toLowerCase(), contains('hi'));

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.caretOffset, 0);

      await tester.sendKeyEvent(LogicalKeyboardKey.delete);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.documentText, 'i');
    });

    testWidgets('shift+arrow extends selection', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'abc');

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.pump(const Duration(milliseconds: 50));

      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText, 'b');
    });

    test('selectGlyphWordAt selects typed word on double-click path', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'hello');

      controller.selectGlyphWordAt(0, 72 + 20, 72 + 20);

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText.toLowerCase(), 'hello');
    });

    testWidgets('enter key creates a new paragraph without tofu', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'A');

      final textBefore = controller.documentText;
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.documentText.contains('\n'), isTrue);
      expect(controller.documentText.length, greaterThan(textBefore.length));
      expect(controller.caretOffset, 0);
    });

    test('engine session can bold after ensuring caret', () {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      controller.ensureGlyphCaret();
      controller.toggleBold();
      expect(controller.bold, isTrue);
      controller.toggleBold();
      expect(controller.bold, isFalse);
    });

    testWidgets('glyph copy and paste use engine text range', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await typeTextDirect(controller, 'ab');

      await controller.selectAll();
      expect(controller.selectedText, isNotEmpty);
      final copied = controller.selectedText;

      await controller.deleteSelection();
      await controller.ensureLayoutReady();
      expect(controller.selectedText, isEmpty);

      await controller.pastePayload(
        EditorClipboardPayload(plainText: copied),
        plainText: true,
      );
      await controller.ensureLayoutReady();
      await controller.selectAll();
      expect(controller.selectedText, isNotEmpty);
    });

    testWidgets('Normal style button updates active paragraph style', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 120, child: HomeTab(controller: controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(controller.activeParagraphStyle, 'Normal');

      await tester.tap(find.text('Heading 1'));
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(controller.activeParagraphStyle, 'Heading 1');

      await tester.tap(find.text('Normal'));
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(controller.activeParagraphStyle, 'Normal');
    });

    testWidgets('ribbon syncs bold state after undo', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpTestDocumentView(tester, controller);

      controller.ensureGlyphCaret();
      expect(controller.bold, isFalse);

      controller.toggleBold();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(controller.bold, isTrue);

      await controller.undo();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(controller.bold, isFalse);
    });

    testWidgets('indent and clear formatting update controller state', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpTestDocumentView(tester, controller);

      controller.ensureGlyphCaret();
      expect(controller.indentLeft, 0);

      controller.increaseIndent();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(controller.indentLeft, greaterThan(0));

      controller.toggleBold();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(controller.bold, isTrue);

      controller.clearFormatting();
      await controller.ensureLayoutReady();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(controller.bold, isFalse);
    });

    testWidgets('page break command completes without error', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpTestDocumentView(tester, controller);

      final before = controller.pageCount;
      controller.insertPageBreak();
      await controller.ensureLayoutReady();
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();
      expect(controller.pageCount, greaterThanOrEqualTo(before));
    });
  });

  group('DisplayListSnapshot empty helper', () {
    test('empty snapshot is not paintable but has page size', () {
      final snapshot = DisplayListSnapshot.empty();
      expect(snapshot.hasPaintableContent, isFalse);
      expect(snapshot.hasPaintableGlyphs, isFalse);
      expect(snapshot.pageWidth, greaterThan(0));
      expect(snapshot.pageHeight, greaterThan(0));
    });
  });
}
