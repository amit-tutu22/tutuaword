import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Select All', () {
    testWidgets('Cmd+A selects entire document in glyph mode', (tester) async {
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

      for (final ch in 'hello'.split('')) {
        await tester.sendKeyEvent(LogicalKeyboardKey(ch.codeUnitAt(0)));
        await tester.pump(const Duration(milliseconds: 20));
      }
      await tester.pumpAndSettle();

      expect(controller.hasGlyphSelection, isFalse);

      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await tester.pumpAndSettle();

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText.toLowerCase(), 'hello');
      expect(controller.selectionRects, isNotEmpty);
    });

    test('selectAll selects full document text via engine tail hit', () {
      final controller = EditorController(enableAutosave: false);
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      controller.ensureGlyphCaret();
      for (final ch in 'alpha beta'.split('')) {
        controller.insertGlyphCharacter(ch == ' ' ? ' ' : ch);
      }

      controller.selectAll();
      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText.toLowerCase(), 'alpha beta');
    });
  });
}
