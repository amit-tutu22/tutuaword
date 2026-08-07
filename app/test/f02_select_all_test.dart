import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Select All', () {
    testWidgets('Cmd+A selects entire document in glyph mode', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'hello');

      expect(controller.hasGlyphSelection, isFalse);

      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText.toLowerCase(), 'hello');
      expect(controller.selectionRects, isNotEmpty);
    });

    test('selectAll selects full document text via engine tail hit', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'alpha beta');

      await controller.selectAll();
      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText.toLowerCase(), 'alpha beta');
    });
  });
}
