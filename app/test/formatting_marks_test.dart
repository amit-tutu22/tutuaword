import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/formatting_marks.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Show formatting marks (¶)', () {
    test('U-formatting-marks-collect-space-tab-para', () {
      final engine = MockDocumentEngine(initialText: 'A B\tC');
      final marks = collectFormattingMarks(
        engine: engine,
        pageIndex: 0,
        runId: engine.defaultRunId,
        text: 'A B\tC',
      );
      expect(
        marks.where((m) => m.kind == FormattingMarkKind.space),
        hasLength(1),
      );
      expect(
        marks.where((m) => m.kind == FormattingMarkKind.tab),
        hasLength(1),
      );
      expect(
        marks.where((m) => m.kind == FormattingMarkKind.paragraph),
        hasLength(1),
      );
    });

    test('U-formatting-marks-toggle', () {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      expect(controller.showFormattingMarks, isFalse);
      controller.toggleFormattingMarks();
      expect(controller.showFormattingMarks, isTrue);
      expect(
        controller.sessionController.statusText,
        contains('Formatting marks shown'),
      );
      controller.toggleFormattingMarks();
      expect(controller.showFormattingMarks, isFalse);
    });

    testWidgets('I-formatting-marks-from-home-eye', (tester) async {
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: 'Hi there'),
      );
      addTearDown(controller.dispose);

      await pumpWideRibbon(
        tester,
        SizedBox(height: 140, child: HomeTab(controller: controller)),
      );

      await tester.tap(find.byKey(const Key('show_formatting_marks')));
      await tester.pumpAndSettle();
      expect(controller.showFormattingMarks, isTrue);
      expect(controller.formattingMarksForPage(0), isNotEmpty);

      await tester.tap(find.byKey(const Key('show_formatting_marks')));
      await tester.pumpAndSettle();
      expect(controller.showFormattingMarks, isFalse);
      expect(controller.formattingMarksForPage(0), isEmpty);
    });
  });
}
