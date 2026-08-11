import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mail_merge_csv.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/mailings_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F26.S2 Mail merge', () {
    test('U-F26-S2-parse-csv', () {
      final data = parseMailMergeCsv('Name,City\nAda,Paris\nGrace,London\n');
      expect(data.headers, ['Name', 'City']);
      expect(data.rows, hasLength(2));
      expect(data.rows.first['Name'], 'Ada');
      expect(mergeFieldPlaceholder('Name'), '«Name»');
    });

    test('U-F26-S2-mock-merge-field-replace', () async {
      final engine = MockDocumentEngine(initialText: '');
      await engine.insertMergeFieldAsync(
        runId: 'run-0',
        offset: 0,
        name: 'Name',
      );
      expect(engine.text, contains('«Name»'));
      final ok = await engine.applyMailMergeRowAsync(values: {'Name': 'Ada'});
      expect(ok, isTrue);
      expect(engine.text, contains('Ada'));
      expect(engine.text, isNot(contains('«Name»')));
    });

    testWidgets('I-F26-S2-start-mail-merge-from-ribbon', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Hello «Name»');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, MailingsTab(controller: controller));
      await tester.tap(find.byKey(const Key('start_mail_merge')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('start_mail_merge_dialog')), findsOneWidget);
      await tester.enterText(
        find.byKey(const Key('mail_merge_csv_field')),
        'Name,City\nAda,Paris\n',
      );
      await tester.tap(find.byKey(const Key('mail_merge_start_button')));
      await tester.pumpAndSettle();

      expect(controller.hasMailMergeData, isTrue);
      expect(
        controller.sessionController.statusText,
        contains('Mail merge'),
      );
    });

    testWidgets('I-F26-S2-insert-and-finish-merge', (tester) async {
      final engine = MockDocumentEngine(initialText: '');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      // Load data via controller API (same as dialog success path).
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: Column(
                children: [
                  TextButton(
                    key: const Key('open_start'),
                    onPressed: () => controller.startMailMerge(context),
                    child: const Text('Start'),
                  ),
                  TextButton(
                    key: const Key('open_insert'),
                    onPressed: () => controller.insertMergeField(context),
                    child: const Text('Insert'),
                  ),
                  TextButton(
                    key: const Key('do_finish'),
                    onPressed: () => controller.finishMailMerge(),
                    child: const Text('Finish'),
                  ),
                ],
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('open_start')));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('mail_merge_csv_field')),
        'Name\nAda\n',
      );
      await tester.tap(find.byKey(const Key('mail_merge_start_button')));
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('open_insert')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('insert_merge_field_dialog')), findsOneWidget);
      await tester.tap(find.byKey(const Key('merge_field_insert_button')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);
      expect(engine.text, contains('«Name»'));

      await tester.tap(find.byKey(const Key('do_finish')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);
      expect(engine.text, contains('Ada'));
      expect(
        controller.sessionController.statusText,
        contains('Mail merge applied'),
      );
    });

    testWidgets('MailingsTab wires merge buttons', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: MailingsTab(controller: controller),
          ),
        ),
      );

      expect(find.byKey(const Key('start_mail_merge')), findsOneWidget);
      expect(find.byKey(const Key('insert_merge_field')), findsOneWidget);
      expect(find.byKey(const Key('finish_mail_merge')), findsOneWidget);
    });

    test('S-F26-S2-mail-merge-churn', () async {
      final buffer = StringBuffer('Name,City\n');
      for (var i = 0; i < 200; i++) {
        buffer.writeln('User$i,City$i');
      }
      final data = parseMailMergeCsv(buffer.toString());
      expect(data.rows, hasLength(200));

      for (var i = 0; i < 200; i++) {
        final engine = MockDocumentEngine(initialText: 'Hello «Name» from «City»');
        final ok = await engine.applyMailMergeRowAsync(values: data.rows[i]);
        expect(ok, isTrue);
        expect(engine.text, 'Hello User$i from City$i');
      }
    }, timeout: const Timeout(Duration(seconds: 30)));
  });
}
