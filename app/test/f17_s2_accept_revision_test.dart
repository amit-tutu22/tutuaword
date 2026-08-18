import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F17.S2 Track changes at caret', () {
    testWidgets('I-F17-S2-accept-button', (tester) async {
      final engine = MockDocumentEngine(initialText: '');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.toggleTrackChanges();
      await engine.tryInsertTextAsync(engine.defaultRunId, 0, 'Tracked');
      await settleEngineStyle(tester);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('accept_revision')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('accept_revision')));
      await tester.pumpAndSettle();

      expect(engine.text, 'Tracked');
      expect(
        controller.sessionController.statusText,
        contains('Accepted revision at caret'),
      );
      expect(
        engine.acceptRevisionAtCaret(caretRunId: engine.defaultRunId),
        isFalse,
      );
    });

    testWidgets('I-F17-S2-reject-button', (tester) async {
      final engine = MockDocumentEngine(initialText: '');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.toggleTrackChanges();
      await engine.tryInsertTextAsync(engine.defaultRunId, 0, 'Remove');
      await settleEngineStyle(tester);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('reject_revision')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('reject_revision')));
      await tester.pumpAndSettle();

      expect(engine.text, isEmpty);
      expect(
        controller.sessionController.statusText,
        contains('Rejected revision at caret'),
      );
    });

    testWidgets('I-F17-S2-next-previous-change', (tester) async {
      final engine = MockDocumentEngine(initialText: '');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.toggleTrackChanges();
      await engine.tryInsertTextAsync(engine.defaultRunId, 0, 'One');
      await engine.tryInsertTextAsync(engine.defaultRunId, 1, 'Two');
      await settleEngineStyle(tester);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('next_revision')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('next_revision')));
      await tester.pumpAndSettle();
      expect(controller.sessionController.statusText, contains('Next change'));

      await tester.scrollUntilVisible(
        find.byKey(const Key('previous_revision')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('previous_revision')));
      await tester.pumpAndSettle();
      expect(controller.sessionController.statusText, contains('Previous change'));
    });

    testWidgets('I-F17-S2-accept-direct-api', (tester) async {
      final engine = MockDocumentEngine(initialText: '');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.toggleTrackChanges();
      await engine.tryInsertTextAsync(engine.defaultRunId, 0, 'Direct');
      await settleEngineStyle(tester);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return TextButton(
                onPressed: controller.acceptRevisionAtCaret,
                child: const Text('AcceptCaret'),
              );
            },
          ),
        ),
      );

      await tester.tap(find.text('AcceptCaret'));
      await tester.pumpAndSettle();

      expect(engine.text, 'Direct');
      expect(
        controller.sessionController.statusText,
        contains('Accepted revision at caret'),
      );
    });
  });

  testWidgets('ReviewTab wires accept, reject, and navigation buttons', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ReviewTab(controller: controller),
        ),
      ),
    );

    expect(find.byKey(const Key('accept_revision')), findsOneWidget);
    expect(find.byKey(const Key('reject_revision')), findsOneWidget);
    expect(find.byKey(const Key('next_revision')), findsOneWidget);
    expect(find.byKey(const Key('previous_revision')), findsOneWidget);
  });
}
