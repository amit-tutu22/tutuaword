import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/bridge/spell_issue.dart';
import 'package:tutuaword/editor/editor_controller.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Next release word gaps', () {
    testWidgets('I-references-endnote-and-table-of-figures', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Body');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => Column(
                children: [
                  ElevatedButton(
                    key: const Key('btn_endnote'),
                    onPressed: () => controller.insertEndnote(context),
                    child: const Text('Endnote'),
                  ),
                  ElevatedButton(
                    key: const Key('btn_tof'),
                    onPressed: () => controller.insertTableOfFigures(context),
                    child: const Text('ToF'),
                  ),
                ],
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('btn_endnote')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('btn_tof')));
      await tester.pumpAndSettle();
      expect(engine.text, contains('Table of Figures'));
    });

    test('U-spell-issues-json-parse', () {
      const json = '[{"word":"Teh","suggestions":["the"]}]';
      final issues = SpellIssue.parseJsonList(json);
      expect(issues, hasLength(1));
      expect(issues.first.word, 'Teh');
      expect(issues.first.suggestions, contains('the'));
    });

    testWidgets('I-mailings-envelope-and-rules', (tester) async {
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: ''),
      );
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => Column(
                children: [
                  TextButton(
                    key: const Key('btn_start_merge'),
                    onPressed: () => controller.startMailMerge(context),
                    child: const Text('Start'),
                  ),
                  ElevatedButton(
                    key: const Key('btn_envelope'),
                    onPressed: () => controller.createEnvelope(context),
                    child: const Text('Envelope'),
                  ),
                  ElevatedButton(
                    key: const Key('btn_next'),
                    onPressed: () => controller.insertNextRecordField(context),
                    child: const Text('Next'),
                  ),
                ],
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.byKey(const Key('btn_start_merge')));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('mail_merge_csv_field')),
        'Name\nAda\n',
      );
      await tester.tap(find.byKey(const Key('mail_merge_start_button')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('btn_envelope')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('create_envelope')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('btn_next')));
      await tester.pumpAndSettle();
      expect(controller.statusText, contains('Next Record'));
    });
  });
}
