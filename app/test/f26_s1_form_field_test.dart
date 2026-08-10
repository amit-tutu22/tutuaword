import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/form_field_dialog.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F26.S1 Form fields', () {
    test('U-F26-S1-mock-insert-form-text', () async {
      final engine = MockDocumentEngine(initialText: '');
      final ok = await engine.insertFormFieldAsync(
        runId: 'run-0',
        offset: 0,
        kind: 'text',
        name: 'City',
        initialValue: 'Paris',
      );
      expect(ok, isTrue);
      expect(engine.text, contains('Paris'));
      expect(engine.lastFormFieldRunId, isNotNull);
    });

    test('U-F26-S1-mock-toggle-checkbox', () async {
      final engine = MockDocumentEngine(initialText: '');
      await engine.insertFormFieldAsync(
        runId: 'run-0',
        offset: 0,
        kind: 'checkbox',
        initialValue: 'false',
      );
      final id = engine.lastFormFieldRunId!;
      expect(engine.text, contains('☐'));
      final ok = await engine.setFormFieldValueAsync(runId: id, value: 'toggle');
      expect(ok, isTrue);
      expect(engine.text, contains('☑'));
    });

    testWidgets('I-F26-S1-insert-form-text-from-insert-ribbon', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Hello');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('insert_form_field')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('insert_form_field')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('form_field_dialog')), findsOneWidget);
      await tester.enterText(
        find.byKey(const Key('form_field_name_field')),
        'FullName',
      );
      await tester.enterText(
        find.byKey(const Key('form_field_text_field')),
        'Ada',
      );
      await tester.tap(find.byKey(const Key('form_field_insert_button')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('Ada'));
      expect(
        controller.sessionController.statusText,
        contains('Form field inserted'),
      );
    });

    testWidgets('I-F26-S1-insert-checkbox-from-dialog', (tester) async {
      final engine = MockDocumentEngine(initialText: 'x');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                key: const Key('open_form_field_dialog'),
                onPressed: () => controller.insertFormField(context),
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.byKey(const Key('open_form_field_dialog')));
      await tester.pumpAndSettle();

      await tester.tap(find.text('Checkbox'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('form_field_checked_toggle')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('form_field_insert_button')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('☑'));
      expect(
        controller.sessionController.statusText,
        contains('Form field inserted'),
      );
    });

    testWidgets('InsertTab wires Form Field button', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: InsertTab(controller: controller),
          ),
        ),
      );

      expect(find.byKey(const Key('insert_form_field')), findsOneWidget);
    });

    test('U-F26-S1-dialog-result-wires', () {
      const text = FormFieldDialogResult(
        kind: FormFieldDialogKind.plainText,
        name: 'n',
        initialValue: 'v',
      );
      expect(text.kindWire, 'text');
      expect(text.initialValueWire, 'v');

      const box = FormFieldDialogResult(
        kind: FormFieldDialogKind.checkbox,
        checked: true,
      );
      expect(box.kindWire, 'checkbox');
      expect(box.initialValueWire, 'true');
    });

    test('S-F26-S1-form-field-churn', () async {
      final engine = MockDocumentEngine(initialText: '');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      for (var i = 0; i < 100; i++) {
        final ok = await engine.insertFormFieldAsync(
          runId: 'run-0',
          offset: engine.text.length,
          kind: i.isEven ? 'text' : 'checkbox',
          name: 'f$i',
          initialValue: i.isEven ? 'v$i' : (i % 4 == 1 ? 'true' : 'false'),
        );
        expect(ok, isTrue);
      }
      expect(engine.text, contains('v0'));
      expect(engine.lastFormFieldRunId, isNotNull);

      final id = engine.lastFormFieldRunId!;
      for (var i = 0; i < 50; i++) {
        await controller.setFormFieldValue(runId: id, value: 'toggle');
      }
      expect(engine.text.contains('☑') || engine.text.contains('☐'), isTrue);
    }, timeout: const Timeout(Duration(seconds: 30)));
  });
}
