import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/paragraph_borders_dialog.dart';
import 'package:tutuaword/ui/paragraph_spacing_dialog.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F04.S4 Borders and pagination', () {
    testWidgets('I-F04-S4-spacing-dialog applies pagination flags', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Page');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 200, child: HomeTab(controller: controller)),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Paragraph Spacing'));
      await tester.pumpAndSettle();

      await tester.ensureVisible(find.byKey(const Key('keep_together')));
      await tester.tap(find.byKey(const Key('keep_together')));
      await tester.tap(find.byKey(const Key('keep_with_next')));
      await tester.tap(find.byKey(const Key('widow_orphan')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('spacing_dialog_ok')));
      await tester.pumpAndSettle();
      await controller.ensureLayoutReady();

      expect(controller.keepTogether, isTrue);
      expect(controller.keepWithNext, isTrue);
      expect(controller.widowOrphanControl, isFalse);

      final para = mockEngineParaFormat(engine);
      expect(para['keep_together'], isTrue);
      expect(para['keep_with_next'], isTrue);
      expect(para['widow_orphan_control'], isFalse);
    });

    testWidgets('I-F04-S4-borders-dialog applies uniform border width', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Bordered');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 120, child: HomeTab(controller: controller)),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Borders and Shading'));
      await tester.pumpAndSettle();
      expect(find.byType(ParagraphBordersDialog), findsOneWidget);

      await tester.tap(find.byKey(const Key('para_border_enabled')));
      await tester.pumpAndSettle();
      await tester.enterText(find.byKey(const Key('para_border_width')), '2');
      await tester.tap(find.byKey(const Key('borders_dialog_ok')));
      await tester.pumpAndSettle();
      await controller.ensureLayoutReady();

      expect(controller.borderWidth, 2);
      final top = (mockEngineParaFormat(engine)['borders'] as Map)['top'] as Map;
      expect((top['width'] as num).toDouble(), 2);
      expect((top['color'] as Map)['r'], 0);
    });

    test('applyBordersAndShading encodes paragraph shading', () async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.applyBordersAndShading(shading: const Color(0xFFFFFF00));
      await controller.ensureLayoutReady();

      final shading = mockEngineParaFormat(engine)['shading'] as Map;
      expect(shading['r'], 255);
      expect(shading['g'], 255);
      expect(shading['b'], 0);
      expect(shading['a'], 255);
    });

    testWidgets('I-F04-S4-borders-dialog clear removes shading and borders', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.applyBordersAndShading(
        shading: Colors.yellow,
        borderWidth: 1,
      );
      await controller.ensureLayoutReady();

      await pumpWideRibbon(
        tester,
        SizedBox(height: 120, child: HomeTab(controller: controller)),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Borders and Shading'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('borders_dialog_clear')));
      await tester.pumpAndSettle();
      await controller.ensureLayoutReady();

      expect(controller.paraShading, isNull);
      expect(controller.borderWidth, 0);

      final para = mockEngineParaFormat(engine);
      expect((para['shading'] as Map)['a'], 0);
      final borders = para['borders'] as Map;
      expect(borders['top'], isNull);
      expect(borders['left'], isNull);
    });
  });
}
