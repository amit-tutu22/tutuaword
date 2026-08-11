import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F07.S2 Section breaks', () {
    testWidgets('Layout tab inserts section break', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 120, child: LayoutTab(controller: controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Section'));
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.sessionController.statusText, contains('Section break inserted'));
    });
  });
}
