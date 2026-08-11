import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/doc_range.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Format Painter', () {
    test('U-format-painter-pickup-and-apply', () async {
      final engine = MockDocumentEngine(initialText: 'HelloWorld');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.selectionController.selectDocRange(
        DocRange(
          anchor: DocPosition(runId: engine.defaultRunId, offset: 0),
          focus: DocPosition(runId: engine.defaultRunId, offset: 5),
        ),
      );
      controller.toggleBold();
      await controller.ensureLayoutReady();
      expect(controller.bold, isTrue);

      controller.toggleFormatPainter();
      expect(controller.formatPainterArmed, isTrue);

      await controller.formattingController.clearFormatting();
      await controller.ensureLayoutReady();
      expect(controller.bold, isFalse);
      expect(controller.formatPainterArmed, isTrue);

      controller.selectionController.selectDocRange(
        DocRange(
          anchor: DocPosition(runId: engine.defaultRunId, offset: 5),
          focus: DocPosition(runId: engine.defaultRunId, offset: 10),
        ),
      );
      await Future<void>.delayed(Duration.zero);
      await controller.ensureLayoutReady();

      expect(controller.formatPainterArmed, isFalse);
      expect(controller.bold, isTrue);
      expect(controller.sessionController.statusText, contains('Format painted'));
    });

    test('U-format-painter-second-click-cancels', () {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      controller.toggleFormatPainter();
      expect(controller.formatPainterArmed, isTrue);
      controller.toggleFormatPainter();
      expect(controller.formatPainterArmed, isFalse);
      expect(
        controller.sessionController.statusText,
        contains('Format Painter cancelled'),
      );
    });

    testWidgets('I-format-painter-from-home-ribbon', (tester) async {
      final engine = MockDocumentEngine(initialText: 'AbcDef');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.selectionController.selectDocRange(
        DocRange(
          anchor: DocPosition(runId: engine.defaultRunId, offset: 0),
          focus: DocPosition(runId: engine.defaultRunId, offset: 3),
        ),
      );
      controller.toggleItalic();
      await settleEngineStyle(tester);

      await pumpWideRibbon(
        tester,
        SizedBox(height: 140, child: HomeTab(controller: controller)),
      );

      await tester.tap(find.byKey(const Key('format_painter')));
      await tester.pumpAndSettle();
      expect(controller.formatPainterArmed, isTrue);

      controller.selectionController.selectDocRange(
        DocRange(
          anchor: DocPosition(runId: engine.defaultRunId, offset: 3),
          focus: DocPosition(runId: engine.defaultRunId, offset: 6),
        ),
      );
      await tester.pumpAndSettle();

      expect(controller.formatPainterArmed, isFalse);
      expect(controller.italic, isTrue);
    });
  });
}
