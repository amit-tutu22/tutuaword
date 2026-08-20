import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/text_to_speech.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F21.S5 Read aloud', () {
    testWidgets('I-F21-S5-read-aloud-speaks-selection', (tester) async {
      final tts = RecordingTextToSpeech();
      final controller = createTestEditorController(textToSpeech: tts);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Hello accessibility');
      await controller.selectAll();
      expect(controller.selectedText, 'Hello accessibility');

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('read_aloud')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('read_aloud')));
      await tester.pumpAndSettle();

      expect(tts.spoken, ['Hello accessibility']);
      expect(controller.sessionController.statusText, contains('Finished reading'));
    });

    testWidgets('I-F21-S5-read-aloud-requires-selection', (tester) async {
      final tts = RecordingTextToSpeech();
      final controller = createTestEditorController(textToSpeech: tts);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'No selection yet');
      await controller.readAloudSelection();

      // No selection → read the whole document rather than refusing.
      expect(tts.spoken, ['No selection yet']);
      expect(controller.sessionController.statusText, contains('Finished'));
    });

    testWidgets('I-F21-S5-read-aloud-empty-document', (tester) async {
      final tts = RecordingTextToSpeech();
      final controller = createTestEditorController(textToSpeech: tts);
      addTearDown(controller.dispose);

      await controller.readAloudSelection();

      expect(tts.spoken, isEmpty);
      expect(controller.sessionController.statusText, contains('Nothing to read'));
    });

    testWidgets('I-F21-S5-read-aloud-stop', (tester) async {
      final tts = RecordingTextToSpeech(completeImmediately: false);
      final controller = createTestEditorController(textToSpeech: tts);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Long passage');
      await controller.selectAll();

      final speakFuture = controller.readAloudSelection();
      await tester.pump();
      expect(controller.isReadingAloud, isTrue);

      await controller.stopReadAloud();
      await speakFuture;
      expect(controller.isReadingAloud, isFalse);
      expect(controller.sessionController.statusText, contains('stopped'));
    });

    testWidgets('Review tab wires Read Aloud button', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ReviewTab(controller: controller),
          ),
        ),
      );

      expect(find.byKey(const Key('read_aloud')), findsOneWidget);
      expect(find.textContaining('Read'), findsWidgets);
    });
  });
}
