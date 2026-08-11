import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F07.S3 Columns', () {
    testWidgets('I-F07-S3-columns applies Two columns', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      expect(controller.columnCountLabel, 'One');

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 120, child: LayoutTab(controller: controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Columns'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Two'));
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.columnCountLabel, 'Two');
      expect(controller.sessionController.statusText, contains('Two columns applied'));
    });
  });
}
