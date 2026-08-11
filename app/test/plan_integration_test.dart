import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Plan integration — Home tab lists', () {
    testWidgets('numbered list button is wired and updates status', (tester) async {
      final controller = EditorController.forTest();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 120,
              child: HomeTab(controller: controller),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Numbering'));
      await tester.pump();

      if (controller.isEngineConnected) {
        expect(controller.statusText, contains('Numbered list'));
      } else {
        expect(controller.statusText, contains('Numbered list applied'));
      }
    });

    testWidgets('bullet and numbered list tooltips are present', (tester) async {
      final controller = EditorController.forTest();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 120,
              child: HomeTab(controller: controller),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byTooltip('Bullets'), findsOneWidget);
      expect(find.byTooltip('Numbering'), findsOneWidget);
    });
  });

  group('Plan integration — EditorController lists', () {
    test('applyNumberedList updates status in mock mode', () async {
      final controller = EditorController.forTest();
      addTearDown(controller.dispose);

      controller.applyNumberedList();
      await Future<void>.delayed(Duration.zero);
      expect(controller.statusText, contains('Numbered list applied'));
    });

    test('applyBulletList and applyNumberedList are distinct', () async {
      final controller = EditorController.forTest();
      addTearDown(controller.dispose);

      controller.applyBulletList();
      await Future<void>.delayed(Duration.zero);
      final bulletStatus = controller.statusText;

      controller.applyNumberedList();
      await Future<void>.delayed(Duration.zero);
      final numberedStatus = controller.statusText;

      expect(bulletStatus, isNot(equals(numberedStatus)));
      expect(numberedStatus, contains('Numbered'));
    });
  });
}
