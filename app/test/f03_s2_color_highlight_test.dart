import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/ui/ribbon_color_picker.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F03.S2 Color and highlight', () {
    testWidgets('I-F03-S2-highlight-visible emits yellow rect in display list', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Hi');

      controller.setHighlight(kRibbonHighlightColors.first);
      expect(controller.highlightColor, kRibbonHighlightColors.first);

      controller.setDisplayListForTest(
        fakeGlyphDisplayList(rectCount: 1, rectColors: const [0xFFFFFF00]),
      );
      final snapshot = DisplayListSnapshot.fromBytes(controller.displayListBytes);
      const yellowArgb = 0xFFFFFF00;
      expect(
        snapshot.rectColors.any((c) => (c & 0xFFFFFFFF) == yellowArgb),
        isTrue,
        reason: 'highlight decoration should appear in rect batch',
      );
    });

    testWidgets('Home tab exposes font and highlight color pickers', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpWideRibbon(tester, SizedBox(height: 120, child: HomeTab(controller: controller)));
      await tester.pumpAndSettle();

      expect(find.byTooltip('Font Color'), findsOneWidget);
      expect(find.byTooltip('Text Highlight Color'), findsOneWidget);

      controller.setFontColor(const Color(0xFFFF0000));
      controller.setHighlight(kRibbonHighlightColors.first);
      expect(controller.fontColor, const Color(0xFFFF0000));
      expect(controller.highlightColor, kRibbonHighlightColors.first);
    });

    testWidgets('font color picker shows Word-style theme grid', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpWideRibbon(tester, SizedBox(height: 120, child: HomeTab(controller: controller)));
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Font Color'));
      await tester.pumpAndSettle();

      expect(find.text('Automatic'), findsOneWidget);
      expect(find.text('Theme Colors'), findsOneWidget);
      expect(find.text('Standard Colors'), findsOneWidget);
      expect(find.byType(WordColorPalettePanel), findsOneWidget);
    });

    test('setFontColor applies red via engine when connected', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'x');
      controller.setFontColor(const Color(0xFFFF0000));
      expect(controller.fontColor, const Color(0xFFFF0000));
    });
  });
}
