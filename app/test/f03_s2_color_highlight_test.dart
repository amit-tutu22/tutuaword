import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/ui/ribbon_color_picker.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F03.S2 Color and highlight', () {
    testWidgets('I-F03-S2-highlight-visible emits yellow rect in display list', (tester) async {
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

      controller.setHighlight(kRibbonHighlightColors.first);
      await tester.pump(const Duration(milliseconds: 150));
      await tester.pumpAndSettle();

      expect(controller.highlightColor, kRibbonHighlightColors.first);

      final snapshot = DisplayListSnapshot.fromBytes(controller.displayListBytes);
      const yellowArgb = 0xFFFFFF00;
      expect(
        snapshot.rectColors.any((c) => c == yellowArgb),
        isTrue,
        reason: 'highlight decoration should appear in rect batch',
      );
    });

    testWidgets('Home tab exposes font and highlight color pickers', (tester) async {
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

      expect(find.byTooltip('Font Color'), findsOneWidget);
      expect(find.byTooltip('Text Highlight Color'), findsOneWidget);

      controller.setFontColor(const Color(0xFFFF0000));
      controller.setHighlight(kRibbonHighlightColors.first);
      expect(controller.fontColor, const Color(0xFFFF0000));
      expect(controller.highlightColor, kRibbonHighlightColors.first);
    });

    testWidgets('font color picker shows Word-style theme grid', (tester) async {
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

      await tester.tap(find.byTooltip('Font Color'));
      await tester.pumpAndSettle();

      expect(find.text('Automatic'), findsOneWidget);
      expect(find.text('Theme Colors'), findsOneWidget);
      expect(find.text('Standard Colors'), findsOneWidget);
      expect(find.byType(WordColorPalettePanel), findsOneWidget);
    });

    test('setFontColor applies red via engine when connected', () {
      final controller = EditorController(enableAutosave: false);
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      controller.ensureGlyphCaret();
      controller.insertGlyphCharacter('x');
      controller.setFontColor(const Color(0xFFFF0000));
      expect(controller.fontColor, const Color(0xFFFF0000));
    });
  });
}
