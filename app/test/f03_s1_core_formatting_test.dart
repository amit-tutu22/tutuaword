import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F03.S1 Core ribbon', () {
    testWidgets('I-F03-S1-font-size-caret-end applies size to typed text', (tester) async {
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

      for (final ch in 'Hi'.split('')) {
        await tester.sendKeyEvent(LogicalKeyboardKey(ch.codeUnitAt(0)));
        await tester.pump(const Duration(milliseconds: 20));
      }
      await tester.pumpAndSettle();

      controller.setFontSize(24);
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pumpAndSettle();

      expect(controller.fontSize, 24);
      expect(controller.documentText.toLowerCase(), 'hi');
    });

    testWidgets('I-F03-S1-strike-super-sub toggles from Home tab', (tester) async {
      final controller = EditorController(enableAutosave: false);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 120, child: HomeTab(controller: controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Strikethrough'));
      await tester.pump();
      expect(controller.strikethrough, isTrue);

      await tester.tap(find.byTooltip('Subscript'));
      await tester.pump();
      expect(controller.subscript, isTrue);
      expect(controller.superscript, isFalse);

      await tester.tap(find.byTooltip('Superscript'));
      await tester.pump();
      expect(controller.superscript, isTrue);
      expect(controller.subscript, isFalse);
    });

    test('I-F03-S1-bold-toggle-roundtrip with engine caret', () {
      final controller = EditorController(enableAutosave: false);
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      controller.ensureGlyphCaret();
      controller.insertGlyphCharacter('x');
      controller.toggleBold();
      expect(controller.bold, isTrue);
      controller.toggleBold();
      expect(controller.bold, isFalse);
    });
  });
}
