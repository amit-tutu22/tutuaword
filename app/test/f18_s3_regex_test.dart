import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_screen.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F18.S3 Regex and wildcards', () {
    testWidgets('I-F18-S3-regex-finds-digit-groups', (tester) async {
      final engine = MockDocumentEngine(initialText: 'order 42 and 1001');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.openFindPane();
      controller.toggleFindUseRegex();
      controller.setFindQuery(r'\d+');

      expect(controller.findStatusText, contains('of 2'));
    });

    testWidgets('I-F18-S3-wildcards-match-single-character', (tester) async {
      final engine = MockDocumentEngine(initialText: 'cat cot car');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.openFindPane();
      controller.toggleFindUseWildcards();
      controller.setFindQuery('c?t');

      expect(controller.findStatusText, contains('of 2'));
    });

    testWidgets('I-F18-S3-regex-replace-all-from-pane', (tester) async {
      final engine = MockDocumentEngine(initialText: 'item 1 item 22');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await tester.pumpWidget(MaterialApp(home: EditorScreen(controller: controller)));
      controller.openFindPane();
      controller.toggleFindUseRegex();
      await tester.pump();

      await tester.enterText(find.byKey(const Key('find_query')), r'item \d+');
      await tester.enterText(find.byKey(const Key('find_replace')), 'entry');
      await tester.pump();

      await tester.scrollUntilVisible(
        find.byKey(const Key('replace_all')),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byKey(const Key('replace_all')));
      await tester.pump();

      expect(engine.text, 'entry entry');
      expect(controller.findStatusText, contains('Replaced 2'));
    });

    testWidgets('I-F18-S3-invalid-regex-shows-status', (tester) async {
      final engine = MockDocumentEngine(initialText: 'nothing special');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.openFindPane();
      controller.toggleFindUseRegex();
      controller.setFindQuery('[unclosed');

      expect(controller.findStatusText, 'Invalid pattern');
    });
  });
}
