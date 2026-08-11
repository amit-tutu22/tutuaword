import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F05.S1 Basic lists', () {
    /// I-F05-S1-bullet-toggle: Home Bullets sets numbering id 1 on the engine.
    testWidgets('I-F05-S1-bullet-toggle applies bullet numbering', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'List item');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 120, child: HomeTab(controller: controller)),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Bullets'));
      await tester.pumpAndSettle();
      await controller.ensureLayoutReady();

      final numbering = mockEngineNumbering(engine);
      expect(numbering, isNotNull);
      expect(numbering!['numbering_id'], 1);
      expect(numbering['level'], 0);
      expect(mockEngineParaFormat(engine)['outline_level'], isNull);
    });

    testWidgets('I-F05-S1-numbered-toggle sets outline level', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Step one');
      controller.applyNumberedList();
      await controller.ensureLayoutReady();

      final numbering = mockEngineNumbering(engine);
      expect(numbering?['numbering_id'], 2);
      expect(mockEngineParaFormat(engine)['outline_level'], 0);
    });
  });
}
