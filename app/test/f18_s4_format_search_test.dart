import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_screen.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F18.S4 Format search', () {
    testWidgets('I-F18-S4-find-bold-text-without-query', (tester) async {
      final engine = MockDocumentEngine(initialText: 'plain BOLD plain');
      engine.setFormatSpanForTest(start: 6, end: 10, bold: true);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.openFindPane();
      controller.toggleFindBold();

      expect(controller.findStatusText, contains('of 1'));
    });

    testWidgets('I-F18-S4-find-text-only-in-bold', (tester) async {
      final engine = MockDocumentEngine(initialText: 'plain BOLD plain');
      engine.setFormatSpanForTest(start: 6, end: 10, bold: true);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.openFindPane();
      controller.toggleFindBold();
      controller.setFindQuery('plain');

      expect(controller.findStatusText, 'No matches');

      controller.setFindQuery('BOLD');
      expect(controller.findStatusText, contains('of 1'));
    });

    testWidgets('I-F18-S4-find-by-style-name', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Heading text');
      engine.setFormatSpanForTest(start: 0, end: 12, styleName: 'Heading 1');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.openFindPane();
      controller.setFindStyleName('Heading 1');

      expect(controller.findStatusText, contains('of 1'));
    });

    testWidgets('I-F18-S4-bold-toggle-in-find-pane', (tester) async {
      final engine = MockDocumentEngine(initialText: 'word WORD');
      engine.setFormatSpanForTest(start: 5, end: 9, bold: true);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await tester.pumpWidget(MaterialApp(home: EditorScreen(controller: controller)));
      controller.openFindPane();
      await tester.pump();

      await tester.scrollUntilVisible(
        find.byKey(const Key('find_format_bold')),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byKey(const Key('find_format_bold')));
      await tester.pump();

      expect(controller.findBold, isTrue);
      controller.setFindQuery('WORD');
      expect(controller.findStatusText, contains('of 1'));
    });
  });
}
