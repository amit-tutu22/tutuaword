import 'dart:typed_data';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/accessibility_checker_pane.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/image_hit_test.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';
import 'package:tutuaword/ui/status_bar.dart';

import 'editor_test_helpers.dart';

final _png1x1 = Uint8List.fromList([
  0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
  0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
  0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
  0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
  0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
]);

void main() {
  group('F21.S4 Accessibility checker', () {
    testWidgets('I-F21-S4-checker-review-button runs check and updates status',
        (tester) async {
      final engine = MockDocumentEngine();
      engine.setAccessibilityIssuesForTest([
        {
          'rule': 'empty_heading',
          'severity': 'error',
          'message': 'Heading is empty',
          'node_id': 'para-1',
          'run_id': 'run-1',
        },
      ]);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('check_accessibility')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('check_accessibility')));
      await tester.pumpAndSettle();

      expect(controller.showAccessibilityChecker, isTrue);
      expect(controller.accessibilityIssues, hasLength(1));
      expect(controller.accessibilityStatusLabel, contains('1 error'));
      expect(controller.sessionController.statusText, contains('Accessibility'));
    });

    testWidgets('I-F21-S4-checker-pane-jump focuses issue target', (tester) async {
      final engine = MockDocumentEngine();
      engine.setAccessibilityIssuesForTest([
        {
          'rule': 'empty_heading',
          'severity': 'error',
          'message': 'Heading is empty',
          'node_id': 'para-1',
          'run_id': 'run-heading',
        },
        {
          'rule': 'missing_alt',
          'severity': 'error',
          'message': 'Picture is missing alternative text',
          'node_id': 'image-1',
        },
      ]);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.checkAccessibility();
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: AccessibilityCheckerPane(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('accessibility_checker_pane')), findsOneWidget);
      await tester.tap(find.byKey(const Key('accessibility_issue_empty_heading_para-1')));
      await tester.pumpAndSettle();
      expect(controller.caretRunId, 'run-heading');
      expect(controller.sessionController.statusText, 'Heading is empty');

      await tester.tap(find.byKey(const Key('accessibility_issue_missing_alt_image-1')));
      await tester.pumpAndSettle();
      expect(controller.hasSelectedImage, isTrue);
      expect(controller.selectedImageId, 'image-1');
    });

    testWidgets('I-F21-S4-status-bar reflects check result', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ListenableBuilder(
              listenable: controller,
              builder: (context, _) => WordStatusBar(controller: controller),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Accessibility: Not checked'), findsOneWidget);

      controller.checkAccessibility();
      await tester.pumpAndSettle();
      expect(find.text('Accessibility: Good to go'), findsOneWidget);
    });

    testWidgets('U-F21-S4-missing-alt from mock image without alt', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.insertImageBytes(_png1x1, 'image/png');
      await settleEngineStyle(tester);
      controller.selectImage(
        0,
        ImageBounds(
          imageId: engine.mockImageId!,
          index: 0,
          rect: const Rect.fromLTWH(72, 72, 72, 72),
        ),
      );

      controller.checkAccessibility();
      expect(
        controller.accessibilityIssues.any((i) => i.rule == 'missing_alt'),
        isTrue,
      );

      await controller.setSelectedImageAltText('Logo');
      await settleEngineStyle(tester);
      controller.checkAccessibility();
      expect(
        controller.accessibilityIssues.any((i) => i.rule == 'missing_alt'),
        isFalse,
      );
    });
  });
}
