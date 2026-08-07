import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F02.S4 drag-drop text', () {
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
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'alpha beta');
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
      await controller.ensureLayoutReady();
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();

      final text = controller.documentText.toLowerCase();
      expect(text.indexOf('beta'), lessThan(text.indexOf('alpha')));
      expect(text, contains('beta'));
      expect(text, contains('alpha'));
    });

    test('moveGlyphSelectionTo reorders without drag gesture', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'alpha beta');

      controller.selectGlyphWordAt(0, 106, 100);
      expect(controller.selectedText.toLowerCase(), 'beta');

      await controller.moveGlyphSelectionTo(0, 72, 100);
      final text = controller.documentText.toLowerCase();
      expect(text.indexOf('beta'), lessThan(text.indexOf('alpha')));
    });
  });
}
