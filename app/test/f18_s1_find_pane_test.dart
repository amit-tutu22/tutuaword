import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_screen.dart';

import 'editor_test_helpers.dart';

Future<void> sendFindShortcut(WidgetTester tester) async {
  await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
  await tester.sendKeyEvent(LogicalKeyboardKey.keyF);
  await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F18.S1 Find UI', () {
    testWidgets('I-F18-S1-find-pane', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Find find FIND');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await tester.pumpWidget(MaterialApp(home: EditorScreen(controller: controller)));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('find_pane')), findsNothing);

      await sendFindShortcut(tester);
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('find_pane')), findsOneWidget);
      expect(find.byKey(const Key('find_query')), findsOneWidget);
    });

    testWidgets('I-F18-S1-find-highlights-and-navigates', (tester) async {
      final engine = MockDocumentEngine(initialText: 'alpha beta alpha');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await tester.pumpWidget(MaterialApp(home: EditorScreen(controller: controller)));
      await tester.pumpAndSettle();
      await sendFindShortcut(tester);
      await tester.pumpAndSettle();

      await tester.enterText(find.byKey(const Key('find_query')), 'alpha');
      await tester.pumpAndSettle();

      expect(controller.findStatusText, contains('of 2'));
      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText, 'alpha');

      await tester.scrollUntilVisible(
        find.byKey(const Key('find_next')),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byKey(const Key('find_next')));
      await tester.pumpAndSettle();
      expect(controller.findStatusText, contains('2 of 2'));

      await tester.scrollUntilVisible(
        find.byKey(const Key('find_previous')),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byKey(const Key('find_previous')));
      await tester.pumpAndSettle();
      expect(controller.findStatusText, contains('1 of 2'));
    });

    testWidgets('I-F18-S1-find-match-case', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Foo foo FOO');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.openFindPane();
      controller.setFindQuery('foo');
      expect(controller.findStatusText, contains('of 3'));

      controller.toggleFindMatchCase();
      expect(controller.findStatusText, contains('of 1'));
    });

    testWidgets('I-F18-S1-find-close', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await tester.pumpWidget(MaterialApp(home: EditorScreen(controller: controller)));
      await tester.pump();
      controller.openFindPane();
      await tester.pump();

      await tester.scrollUntilVisible(
        find.byKey(const Key('find_close')),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byKey(const Key('find_close')));
      await tester.pump();

      expect(controller.findPaneVisible, isFalse);
      expect(find.byKey(const Key('find_pane')), findsNothing);
    });
  });

  testWidgets('Find pane focuses query field', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);
    controller.setDisplayListForTest(fakeGlyphDisplayList());

    await tester.pumpWidget(MaterialApp(home: EditorScreen(controller: controller)));
    controller.openFindPane();
    await tester.pump();

    expect(find.byKey(const Key('find_query')), findsOneWidget);
  });
}
