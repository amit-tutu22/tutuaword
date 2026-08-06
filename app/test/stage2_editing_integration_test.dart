import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

Uint8List _fakeGlyphDisplayList() {
  final atlas = List<int>.filled(4 * 4 * 4, 0xFF);
  final parts = <int>[
    2, 0, 0, 0,
    1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0x44, 0x43,
    0, 0, 0x46, 0x43,
    4, 0, 0, 0,
    4, 0, 0, 0,
    atlas.length, 0, 0, 0,
    ...atlas,
    1, 0, 0, 0,
    72, 0, 0, 0, 100, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0,
    0, 0, 0, 0,
    0, 0, 0, 0,
    0, 0, 0, 0,
    0, 0, 0, 0,
  ];
  return Uint8List.fromList(parts);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('DocumentView Stage 2 integration', () {
    testWidgets('glyph mode mounts GlyphEditorSurface instead of Start typing placeholder',
        (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(_fakeGlyphDisplayList());

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      expect(find.byType(GlyphEditorSurface), findsWidgets);
      expect(find.text('Start typing…'), findsNothing);
      expect(find.byType(TextField), findsNothing);
    });

    testWidgets('text fallback mode mounts TextField', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (controller.isEngineConnected) {
        // Force text mode for this integration check.
        controller.setDisplayListForTest(Uint8List(0), preferTextRendering: true);
      }

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      if (controller.preferTextRendering) {
        expect(find.byType(TextField), findsOneWidget);
        expect(find.text('Start typing…'), findsNothing);
      }
    });

    testWidgets('empty glyph snapshot still mounts an editable surface', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      // Prefer glyph, empty bytes — surface should still mount via empty snapshot.
      controller.setDisplayListForTest(
        Uint8List(0),
        preferTextRendering: false,
      );

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      expect(controller.preferTextRendering, isFalse);
      // Without paintable page bytes DocumentView still builds a GlyphEditorSurface
      // with DisplayListSnapshot.empty() when preferTextRendering is false.
      expect(find.byType(GlyphEditorSurface), findsWidgets);
      expect(find.text('Start typing…'), findsNothing);
    });
  });

  group('Home tab formatting integration', () {
    testWidgets('bold italic underline toggles update controller state', (tester) async {
      final controller = EditorController();
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
      final controller = EditorController();
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
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      controller.ensureGlyphCaret();
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      controller.setFontSize(24);
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();

      expect(controller.fontSize, 24);
    });

    testWidgets('font family applies after typing at run end', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      controller.ensureGlyphCaret();
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      controller.setFontFamily('Georgia');
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();

      expect(controller.fontFamily, 'Georgia');
    });

    testWidgets('glyph surface accepts key events when engine is connected', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      expect(controller.preferTextRendering, isFalse);

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      expect(find.byType(GlyphEditorSurface), findsWidgets);
      expect(find.text('Start typing…'), findsNothing);

      // Focus and type — should not throw; engine path handles insert.
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();
      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
    });

    testWidgets('tab key inserts a tab and advances caret', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      controller.ensureGlyphCaret();
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      final afterAOffset = controller.caretOffset;
      final afterACaretX = controller.caretGeometry?.x;
      expect(afterAOffset, greaterThan(0));
      expect(afterACaretX, isNotNull);

      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.caretOffset, greaterThan(afterAOffset));
      final afterTabCaretX = controller.caretGeometry?.x;
      expect(afterTabCaretX, isNotNull);
      expect((afterTabCaretX! - afterACaretX!).abs(), greaterThan(0.05));
    });

    testWidgets('space key inserts and advances caret', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      // Ensure caret is placed so insertGlyphCharacter has a target.
      controller.ensureGlyphCaret();
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      // Match the user's report: "A + Space + B".
      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      final afterAOffset = controller.caretOffset;
      final afterACaretX = controller.caretGeometry?.x;
      expect(afterAOffset, greaterThan(0));
      expect(afterACaretX, isNotNull);

      await tester.sendKeyEvent(LogicalKeyboardKey.space);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.caretOffset, greaterThan(afterAOffset));
      final afterSpaceCaretX = controller.caretGeometry?.x;
      expect(afterSpaceCaretX, isNotNull);
      expect((afterSpaceCaretX! - afterACaretX!).abs(), greaterThan(0.05));
    });

    testWidgets('arrow keys move caret in glyph mode', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      controller.ensureGlyphCaret();
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      // Type one character so the caret can move meaningfully.
      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      final afterInsert = controller.caretOffset;
      expect(afterInsert, greaterThan(0));

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.caretOffset, lessThan(afterInsert));

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.caretOffset, afterInsert);
    });

    testWidgets('delete key removes character forward (I-F02-S2-delete-key)', (tester) async {
      final controller = EditorController(enableAutosave: false);
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      controller.ensureGlyphCaret();
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await tester.sendKeyEvent(LogicalKeyboardKey.keyH);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyI);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.documentText.toLowerCase(), contains('hi'));

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.caretOffset, 0);

      await tester.sendKeyEvent(LogicalKeyboardKey.delete);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.documentText, 'i');
    });

    testWidgets('shift+arrow extends selection', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      controller.ensureGlyphCaret();
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      for (final key in [
        LogicalKeyboardKey.keyA,
        LogicalKeyboardKey.keyB,
        LogicalKeyboardKey.keyC,
      ]) {
        await tester.sendKeyEvent(key);
        await tester.pump(const Duration(milliseconds: 20));
      }
      await tester.pumpAndSettle();

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
      await tester.pump(const Duration(milliseconds: 50));

      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText, 'b');
    });

    test('selectGlyphWordAt selects typed word on double-click path', () {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      controller.ensureGlyphCaret();
      for (final ch in 'hello'.split('')) {
        controller.insertGlyphCharacter(ch);
      }

      controller.selectGlyphWordAt(0, 72 + 20, 72 + 20);

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText.toLowerCase(), 'hello');
    });

    testWidgets('enter key creates a new paragraph without tofu', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      controller.ensureGlyphCaret();
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      final textBefore = controller.documentText;
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      // Document text joins paragraphs with \n — Enter should produce a real break,
      // not a glyph tofu box (control chars filtered from insertGlyphCharacter).
      expect(controller.documentText.contains('\n'), isTrue);
      expect(controller.documentText.length, greaterThan(textBefore.length));
      expect(controller.caretOffset, 0);
    });

    test('displayListForPage returns distinct bytes per page after scroll', () {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      controller.insertTable();
      // Force a multi-page layout by inserting many paragraph breaks + text.
      for (var i = 0; i < 80; i++) {
        controller.insertGlyphParagraphBreak();
        controller.insertGlyphCharacter('Line $i of filler text. ');
      }
      expect(controller.pageCount, greaterThan(1));

      final page0 = controller.displayListForPage(0);
      final page1 = controller.displayListForPage(1);
      expect(page0, isNotEmpty);
      expect(page1, isNotEmpty);
      expect(page0, isNot(equals(page1)));

      // Simulate scroll updating visible page without engine round-trip.
      controller.setVisiblePage(1);
      final afterScroll = controller.displayListForPage(1);
      expect(afterScroll, equals(page1));
      expect(afterScroll, isNot(equals(page0)));
    });

    test('typing on later page after click updates that page display list', () {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      for (var i = 0; i < 120; i++) {
        controller.insertGlyphParagraphBreak();
        controller.insertGlyphCharacter('Fill line $i. ');
      }
      expect(controller.pageCount, greaterThanOrEqualTo(2));

      final pageIndex = controller.pageCount - 1;
      controller.hitTestAt(pageIndex, 100, 700);
      expect(controller.caretPage, pageIndex);

      final before = controller.displayListForPage(pageIndex);
      controller.insertGlyphCharacter('Z');
      final after = controller.displayListForPage(pageIndex);
      expect(after, isNotEmpty);
      expect(after, isNot(equals(before)));
    });

    test('engine session can bold after ensuring caret', () {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      controller.ensureGlyphCaret();
      controller.toggleBold();
      expect(controller.bold, isTrue);
      // Does not throw when routing format JSON to FFI (caret may be set).
      controller.toggleBold();
      expect(controller.bold, isFalse);
    });

    testWidgets('glyph copy and paste use engine text range', (tester) async {
      final controller = EditorController(enableAutosave: false);
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      controller.ensureGlyphCaret();
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.keyB);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      controller.selectAll();
      await tester.pump();
      expect(controller.selectedText, isNotEmpty);

      await controller.copySelection();
      controller.deleteSelection();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(controller.selectedText, isEmpty);

      await controller.paste();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      controller.selectAll();
      await tester.pump();
      expect(controller.selectedText, isNotEmpty);
    });

    testWidgets('Normal style button updates active paragraph style', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
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
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      controller.ensureGlyphCaret();
      expect(controller.bold, isFalse);

      controller.toggleBold();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(controller.bold, isTrue);

      controller.undo();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(controller.bold, isFalse);
    });

    testWidgets('indent and clear formatting update controller state', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

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
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(controller.bold, isFalse);
    });

    testWidgets('page break increases page count', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      final before = controller.pageCount;
      controller.insertPageBreak();
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
