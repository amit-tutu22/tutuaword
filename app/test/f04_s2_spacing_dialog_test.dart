import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/paragraph_spacing_dialog.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F04.S2 Spacing UI', () {
    /// I-F04-S2-spacing-dialog: Home Paragraph Spacing dialog writes line
    /// spacing + space before/after onto the engine para format.
    testWidgets('I-F04-S2-spacing-dialog applies spacing', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Hello');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 120, child: HomeTab(controller: controller)),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Paragraph Spacing'));
      await tester.pumpAndSettle();
      expect(find.byType(ParagraphSpacingDialog), findsOneWidget);

      await tester.tap(find.byKey(const Key('line_spacing_mode')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Double').last);
      await tester.pumpAndSettle();

      await tester.enterText(find.byKey(const Key('space_before')), '12');
      await tester.enterText(find.byKey(const Key('space_after')), '6');
      await tester.tap(find.byKey(const Key('spacing_dialog_ok')));
      await tester.pumpAndSettle();
      await controller.ensureLayoutReady();

      expect(controller.lineSpacing, LineSpacingMode.double_);
      expect(controller.spaceBefore, 12);
      expect(controller.spaceAfter, 6);

      final para = mockEngineParaFormat(engine);
      expect(para['line_spacing'], 'Double');
      expect((para['space_before'] as num).toDouble(), 12);
      expect((para['space_after'] as num).toDouble(), 6);
    });

    testWidgets('I-F04-S2 exact line spacing encodes Exactly points', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Exact');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 120, child: HomeTab(controller: controller)),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Paragraph Spacing'));
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('line_spacing_mode')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Exactly').last);
      await tester.pumpAndSettle();

      await tester.enterText(find.byKey(const Key('exact_points')), '24');
      await tester.tap(find.byKey(const Key('spacing_dialog_ok')));
      await tester.pumpAndSettle();
      await controller.ensureLayoutReady();

      expect(controller.lineSpacing, LineSpacingMode.exact);
      expect(controller.exactLineSpacingPt, 24);

      final spacing = mockEngineParaFormat(engine)['line_spacing'];
      expect(spacing, isA<Map>());
      expect(((spacing as Map)['Exactly'] as num).toDouble(), 24);
    });

    testWidgets('I-F04-S2 one-and-a-half encodes Multiple 1.5', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Half');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 120, child: HomeTab(controller: controller)),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Paragraph Spacing'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('line_spacing_mode')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('1.5 lines').last);
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('spacing_dialog_ok')));
      await tester.pumpAndSettle();

      final spacing = mockEngineParaFormat(engine)['line_spacing'] as Map;
      expect((spacing['Multiple'] as num).toDouble(), 1.5);
    });
  });
}
