import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_share.dart';
import 'package:tutuaword/editor/editor_screen.dart';
import 'package:tutuaword/ui/find_pane.dart';
import 'package:tutuaword/ui/ribbon.dart';
import 'package:tutuaword/ui/title_bar.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('title-bar Search opens Find/replace pane', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: WordTitleBar(controller: controller),
        ),
      ),
    );
    expect(controller.findPaneVisible, isFalse);

    await tester.tap(find.byKey(const Key('title_bar_search')));
    await tester.pump();

    expect(controller.findPaneVisible, isTrue);
  });

  testWidgets('ribbon Share opens OS share host with document content',
      (tester) async {
    final share = RecordingShareHost();
    final controller = createTestEditorController(shareHost: share);
    addTearDown(controller.dispose);
    await typeTextDirect(controller, 'Hello share world');

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: WordRibbon(controller: controller),
        ),
      ),
    );

    await tester.tap(find.byKey(const Key('ribbon_share')));
    await tester.pumpAndSettle();

    expect(share.calls, isNotEmpty);
    expect(share.calls.last.text, contains('Hello share world'));
    expect(share.calls.last.subject, isNotNull);
    expect(share.calls.last.filePath, isNotNull);
  });

  test('shareWithApps records file share via host', () async {
    final share = RecordingShareHost();
    final controller = createTestEditorController(shareHost: share);
    addTearDown(controller.dispose);
    await typeTextDirect(controller, 'Share me');

    final result = await controller.shareWithApps();
    expect(result.outcome, DocumentShareOutcome.presented);
    expect(share.calls.single.filePath, isNotNull);
    expect(share.calls.single.mimeType, isNotNull);
  });

  testWidgets('EditorScreen Search key opens FindPane widget', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: EditorScreen(controller: controller),
      ),
    );
    await tester.pump();

    await tester.tap(find.byKey(const Key('title_bar_search')));
    await tester.pumpAndSettle();

    expect(find.byType(FindPane), findsOneWidget);
  });
}
