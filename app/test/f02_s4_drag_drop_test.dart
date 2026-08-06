import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F02.S4 drag-drop text', () {
    Future<void> typeText(WidgetTester tester, String text) async {
      for (final ch in text.split('')) {
        if (ch == ' ') {
          await tester.sendKeyEvent(LogicalKeyboardKey.space);
        } else {
          await tester.sendKeyEvent(LogicalKeyboardKey(ch.codeUnitAt(0)));
        }
        await tester.pump(const Duration(milliseconds: 20));
      }
      await tester.pumpAndSettle();
    }

    Future<void> selectWordWithShiftArrows(WidgetTester tester, int charCount) async {
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      for (var i = 0; i < charCount; i++) {
        await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
        await tester.pump(const Duration(milliseconds: 20));
      }
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await tester.pumpAndSettle();
    }

    testWidgets('I-F02-S4-drag-reorder moves selection to drop position', (tester) async {
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

      await typeText(tester, 'alpha beta');
      expect(controller.documentText.toLowerCase(), contains('alpha beta'));

      for (var i = 0; i < 4; i++) {
        await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
        await tester.pump(const Duration(milliseconds: 20));
      }
      await selectWordWithShiftArrows(tester, 4);
      expect(controller.selectedText.toLowerCase(), 'beta');
      expect(controller.selectionRects, isNotEmpty);

      final surface = find.byType(GlyphEditorSurface).first;
      final topLeft = tester.getTopLeft(surface);
      final rect = controller.selectionRects.first;
      final start = topLeft +
          Offset(rect.x + rect.width / 2, rect.y + rect.height / 2);
      final gesture = await tester.startGesture(start);
      await gesture.moveBy(const Offset(-260, 0));
      await gesture.up();
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();

      final text = controller.documentText.toLowerCase();
      expect(text.indexOf('beta'), lessThan(text.indexOf('alpha')));
      expect(text, contains('beta'));
      expect(text, contains('alpha'));
    });

    test('moveGlyphSelectionTo reorders without drag gesture', () {
      final controller = EditorController(enableAutosave: false);
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      controller.ensureGlyphCaret();
      for (final ch in 'alpha beta'.split('')) {
        controller.insertGlyphCharacter(ch == ' ' ? ' ' : ch);
      }

      controller.beginGlyphSelection(0, controller.pageWidth - 72, 100);
      controller.updateGlyphSelection(0, controller.pageWidth - 72, 100);
      controller.endGlyphSelection(0, controller.pageWidth - 72, 100);
      expect(controller.selectedText.toLowerCase(), isNotEmpty);

      controller.moveGlyphSelectionTo(0, 72, 100);
      final text = controller.documentText.toLowerCase();
      expect(text.indexOf('beta'), lessThan(text.indexOf('alpha')));
    });
  });
}
