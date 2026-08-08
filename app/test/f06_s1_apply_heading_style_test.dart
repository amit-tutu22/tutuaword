import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F06.S1 Built-in paragraph styles', () {
    testWidgets('I-F06-S1-apply-heading-style gallery applies resolved size', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Section title');
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 120, child: HomeTab(controller: controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(controller.activeParagraphStyle, 'Normal');

      await tester.tap(find.text('Heading 1'));
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.activeParagraphStyle, 'Heading 1');
      expect(controller.fontSize, 16);
      expect(controller.bold, isTrue);
    });

    testWidgets('I-F06-S1-apply-heading-style deeper headings resolve size', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Subsection');
      controller.applyParagraphStyle('Heading 2');
      await controller.ensureLayoutReady();

      expect(controller.activeParagraphStyle, 'Heading 2');
      expect(controller.fontSize, 14);
      expect(controller.bold, isTrue);
    });

    testWidgets('I-F06-S1-apply-heading-style Quote applies italic indent', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Quoted text');
      controller.applyParagraphStyle('Quote');
      await controller.ensureLayoutReady();

      expect(controller.activeParagraphStyle, 'Quote');
      expect(controller.italic, isTrue);
      expect(controller.indentLeft, 36);
    });
  });
}
