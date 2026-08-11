import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F02.S2 forward delete and selection extend', () {
    testWidgets('I-F02-S2-delete-key removes next character and undo restores', (tester) async {
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

      await controller.undo();
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.documentText.toLowerCase(), contains('hi'));
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

    testWidgets('double-click selects word', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'hello');

      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText.toLowerCase(), 'hello');
    });

    test('selectGlyphWordAt selects typed word', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'hello');

      controller.selectGlyphWordAt(0, 72 + 20, 72 + 20);

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText.toLowerCase(), 'hello');
    });
  });
}
