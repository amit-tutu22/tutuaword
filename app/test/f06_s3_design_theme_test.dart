import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/ui/ribbon_tabs/design_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F06.S3 Themes', () {
    testWidgets('I-F06-S3-design-tab applies Facet theme', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 120, child: DesignTab(controller: controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(controller.documentThemeName, 'Office');
      expect(find.text('Office'), findsOneWidget);

      await tester.tap(find.text('Facet'));
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.documentThemeName, 'Facet');
    });

    testWidgets('I-F06-S3-design-tab Ion theme is selectable', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 120, child: DesignTab(controller: controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Ion'));
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.documentThemeName, 'Ion');
    });
  });
}
