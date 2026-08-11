import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/ui/ribbon_color_picker.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F06.S3 Theme color slots', () {
    test('themeColorSelectionForPicker maps accent column', () {
      final selection = themeColorSelectionForPicker(
        column: 4,
        row: 3,
        color: const Color(0xFF4472C4),
      );
      expect(selection, isNotNull);
      expect(selection!.themeSlot, 'Accent1');
      expect(selection.themeVariant, 3);
      expect(selection.isThemeColor, isTrue);
    });

    testWidgets('I-F06-S3-theme-color theme accent recolors when theme changes', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Accent text');
      controller.setFontColor(
        const Color(0xFF4472C4),
        themeSlot: 'Accent1',
        themeVariant: 3,
      );
      await controller.ensureLayoutReady();

      expect(controller.fontColor, const Color(0xFF4472C4));

      controller.applyDocumentTheme('Ion');
      await controller.ensureLayoutReady();

      expect(controller.fontColor, const Color(0xFFFF7200));
    });
  });
}
