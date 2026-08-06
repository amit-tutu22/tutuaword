import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F02.S2 forward delete and selection extend', () {
    testWidgets('I-F02-S2-delete-key removes next character and undo restores', (tester) async {
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

      controller.undo();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.documentText.toLowerCase(), contains('hi'));
    });

    testWidgets('shift+arrow extends selection', (tester) async {
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

    testWidgets('double-click selects word', (tester) async {
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

      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText.toLowerCase(), 'hello');
    });

    test('selectGlyphWordAt selects typed word', () {
      final controller = EditorController(enableAutosave: false);
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
  });
}
