import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/changes_pane.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F17.S2 Changes pane', () {
    testWidgets('I-F17-S2-changes-pane lists revisions and accepts', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.toggleTrackChanges();
      await typeTextDirect(controller, 'Tracked');
      await settleEngineStyle(tester);

      await controller.refreshRevisions();
      expect(controller.trackedChanges, isNotEmpty);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ChangesPane(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      final entry = controller.trackedChanges.first;
      expect(find.byKey(Key('changes_entry_${entry.runId}_0')), findsOneWidget);
      await tester.tap(find.byKey(Key('changes_accept_${entry.runId}')));
      await tester.pumpAndSettle();

      expect(controller.trackedChanges, isEmpty);
    });

    testWidgets('N-F17-S2-review-changes-button opens pane', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.toggleTrackChanges();
      await typeTextDirect(controller, 'Change');
      await settleEngineStyle(tester);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('changes_pane')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('changes_pane')));
      await tester.pumpAndSettle();

      expect(controller.showChangesPane, isTrue);
    });
  });
}
