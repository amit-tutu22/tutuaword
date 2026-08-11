import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F07.S1 Page setup', () {
    testWidgets('I-F07-S1-margin-preset applies Narrow margins', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      expect(controller.marginLeft, 72);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 120, child: LayoutTab(controller: controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Margins'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Narrow'));
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.marginLeft, 36);
      expect(controller.marginPresetName, 'Narrow');
    });

    testWidgets('U-F07-S1-landscape-swaps-dimensions', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      expect(controller.pageWidth, 612);
      expect(controller.pageHeight, 792);
      expect(controller.isLandscape, isFalse);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 120, child: LayoutTab(controller: controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Orientation'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Landscape'));
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.pageWidth, 792);
      expect(controller.pageHeight, 612);
      expect(controller.isLandscape, isTrue);
    });

    testWidgets('Layout tab applies Letter page size', (tester) async {
      final engine = MockDocumentEngine(pageWidth: 595, pageHeight: 842);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      expect(controller.pageSizePresetName, 'A4');

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 120, child: LayoutTab(controller: controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Size'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Letter'));
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.pageWidth, 612);
      expect(controller.pageHeight, 792);
      expect(controller.pageSizePresetName, 'Letter');
    });
  });
}
