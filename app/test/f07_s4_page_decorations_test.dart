import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/ui/page_setup.dart';
import 'package:tutuaword/ui/ribbon_tabs/design_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F07.S4 Page decorations', () {
    testWidgets('I-F07-S4-watermark applies and removes DRAFT preset', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      expect(controller.watermarkText, isNull);

      await pumpRibbonTab(tester, DesignTab(controller: controller));
      await tester.tap(find.text('Watermark'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('DRAFT'));
      await settleEngineStyle(tester);

      expect(controller.watermarkText, 'DRAFT');
      expect(controller.sessionController.statusText, contains('Watermark applied'));

      await tester.tap(find.text('Watermark'));
      await tester.pumpAndSettle();
      await tester.tap(find.text(PageSetupPresets.removeWatermarkLabel));
      await settleEngineStyle(tester);

      expect(controller.watermarkText, isNull);
      expect(controller.sessionController.statusText, contains('Watermark removed'));
    });

    testWidgets('I-F07-S4-page-color applies and clears', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      expect(controller.pageColor, isNull);

      const tint = Color(0xFFF0F0F0);
      controller.applyPageColor(tint);
      await settleEngineStyle(tester);

      expect(controller.pageColor, tint);

      controller.applyPageColor(null);
      await settleEngineStyle(tester);

      expect(controller.pageColor, isNull);
      expect(controller.sessionController.statusText, contains('Page color cleared'));
    });

    testWidgets('I-F07-S4-line-numbers toggles on and off', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      expect(controller.lineNumbersEnabled, isFalse);

      await tester.binding.setSurfaceSize(const Size(1400, 120));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await pumpRibbonTab(
        tester,
        LayoutTab(controller: controller),
        size: const Size(1400, 120),
      );

      await tester.scrollUntilVisible(
        find.byIcon(Icons.format_list_numbered),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byIcon(Icons.format_list_numbered));
      await settleEngineStyle(tester);
      expect(controller.lineNumbersEnabled, isTrue);
      expect(controller.sessionController.statusText, contains('Line numbers enabled'));

      await tester.tap(find.byIcon(Icons.format_list_numbered));
      await settleEngineStyle(tester);
      expect(controller.lineNumbersEnabled, isFalse);
      expect(controller.sessionController.statusText, contains('Line numbers disabled'));
    });
  });
}
