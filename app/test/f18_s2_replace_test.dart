import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_screen.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F18.S2 Replace', () {
    testWidgets('I-F18-S2-replace-all-updates-document', (tester) async {
      final engine = MockDocumentEngine(initialText: 'cat dog cat');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await tester.pumpWidget(MaterialApp(home: EditorScreen(controller: controller)));
      controller.openFindPane();
      controller.setFindQuery('cat');
      controller.setFindReplaceText('fish');
      await tester.pump();

      final count = await controller.replaceAll();
      await tester.pump();

      expect(count, 2);
      expect(engine.text, 'fish dog fish');
      expect(controller.findStatusText, contains('Replaced 2'));
    });

    testWidgets('I-F18-S2-replace-all-zero-when-no-match', (tester) async {
      final engine = MockDocumentEngine(initialText: 'nothing here');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.openFindPane();
      controller.setFindQuery('missing');
      controller.setFindReplaceText('nope');

      final count = await controller.replaceAll();
      expect(count, 0);
      expect(engine.text, 'nothing here');
    });

    testWidgets('I-F18-S2-replace-all-from-pane-button', (tester) async {
      final engine = MockDocumentEngine(initialText: 'one two one');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await tester.pumpWidget(MaterialApp(home: EditorScreen(controller: controller)));
      controller.openFindPane();
      await tester.pump();

      await tester.enterText(find.byKey(const Key('find_query')), 'one');
      await tester.enterText(find.byKey(const Key('find_replace')), '1');
      await tester.pump();

      await tester.scrollUntilVisible(
        find.byKey(const Key('replace_all')),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byKey(const Key('replace_all')));
      await tester.pump();

      expect(engine.text, '1 two 1');
      expect(controller.findStatusText, contains('Replaced 2'));
    });
  });
}
