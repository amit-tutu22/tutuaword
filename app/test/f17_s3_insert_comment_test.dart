import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

Future<void> pumpWideTab(WidgetTester tester, Widget tab) async {
  await tester.binding.setSurfaceSize(const Size(1600, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await pumpRibbonTab(tester, tab, size: const Size(1600, 140));
}

void main() {
  group('F17.S3 Comments', () {
    testWidgets('I-F17-S3-insert-comment-from-review-ribbon', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Draft text');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpWideTab(tester, ReviewTab(controller: controller));
      await tester.tap(find.byKey(const Key('insert_comment')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('comment_dialog')), findsOneWidget);
      await tester.enterText(find.byKey(const Key('comment_body')), 'Review note');
      await tester.tap(find.byKey(const Key('comment_ok')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('[C1]'));
      expect(controller.sessionController.statusText, contains('Comment inserted'));
    });

    testWidgets('I-F17-S3-insert-comment-direct-api', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Draft text');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return TextButton(
                onPressed: () => controller.insertComment(context),
                child: const Text('Comment'),
              );
            },
          ),
        ),
      );

      await tester.tap(find.text('Comment'));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('comment_dialog')), findsOneWidget);
      await tester.enterText(find.byKey(const Key('comment_body')), 'API note');
      await tester.tap(find.byKey(const Key('comment_ok')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('[C1]'));
    });
  });

  testWidgets('ReviewTab wires insert comment button', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ReviewTab(controller: controller),
        ),
      ),
    );

    expect(find.byKey(const Key('insert_comment')), findsOneWidget);
  });
}
