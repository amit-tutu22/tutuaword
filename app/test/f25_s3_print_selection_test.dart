import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_print.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/bridge/print_layout_settings.dart';
import 'package:tutuaword/editor/doc_range.dart';
import 'package:tutuaword/ui/print_settings_dialog.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F25.S3 print selection', () {
    test('U-F25-S3-print-scope-defaults-document', () {
      expect(PrintLayoutSettings.defaults.scope, PrintScope.document);
      expect(
        PrintLayoutSettings.defaults
            .copyWith(scope: PrintScope.selection)
            .scope,
        PrintScope.selection,
      );
    });

    test('U-F25-S3-mock-engine-records-selection', () {
      final engine = MockDocumentEngine(initialText: 'Hello World');
      final range = DocRange(
        anchor: DocPosition(runId: engine.defaultRunId, offset: 0),
        focus: DocPosition(runId: engine.defaultRunId, offset: 5),
      );
      final bytes = engine.exportPdfBytesForPrint(
        PrintLayoutSettings.defaults.copyWith(scope: PrintScope.selection),
        range,
      );
      expect(isPdfHeader(bytes!), isTrue);
      expect(engine.lastPrintSelection, range);
    });

    test('I-F25-S3-print-passes-selection', () async {
      final host = RecordingPrintHost();
      final engine = MockDocumentEngine(initialText: 'Hello World');
      final controller = createTestEditorController(
        engine: engine,
        printHost: host,
      );
      addTearDown(controller.dispose);

      controller.selectionController.selectDocRange(
        DocRange(
          anchor: DocPosition(runId: engine.defaultRunId, offset: 0),
          focus: DocPosition(runId: engine.defaultRunId, offset: 5),
        ),
      );
      expect(controller.hasGlyphSelection, isTrue);

      final result = await controller.printDocument(
        layout: PrintLayoutSettings.defaults.copyWith(
          scope: PrintScope.selection,
        ),
      );
      expect(result.outcome, PrintDialogOutcome.presented);
      expect(engine.lastPrintSelection, isNotNull);
      expect(engine.lastPrintSelection!.anchor.offset, 0);
      expect(engine.lastPrintSelection!.focus.offset, 5);
      expect(
        controller.sessionController.statusText,
        contains('selection'),
      );
    });

    test('I-F25-S3-print-selection-without-range-fails', () async {
      final host = RecordingPrintHost();
      final engine = MockDocumentEngine(initialText: 'Hello');
      final controller = createTestEditorController(
        engine: engine,
        printHost: host,
      );
      addTearDown(controller.dispose);

      final result = await controller.printDocument(
        layout: PrintLayoutSettings.defaults.copyWith(
          scope: PrintScope.selection,
        ),
      );
      expect(result.outcome, PrintDialogOutcome.failed);
      expect(host.calls, isEmpty);
      expect(engine.lastPrintSelection, isNull);
    });

    testWidgets('I-F25-S3-print-settings-selection-disabled', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return Scaffold(
                body: TextButton(
                  key: const Key('open_print_settings'),
                  onPressed: () => PrintSettingsDialog.show(
                    context,
                    selectionAvailable: false,
                  ),
                  child: const Text('Open'),
                ),
              );
            },
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('open_print_settings')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('print_scope')), findsOneWidget);
      expect(find.byKey(const Key('print_scope_hint')), findsOneWidget);
    });

    testWidgets('I-F25-S3-print-settings-selection-enabled', (tester) async {
      PrintLayoutSettings? chosen;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return Scaffold(
                body: TextButton(
                  key: const Key('open_print_settings'),
                  onPressed: () async {
                    chosen = await PrintSettingsDialog.show(
                      context,
                      selectionAvailable: true,
                      initial: PrintLayoutSettings.defaults.copyWith(
                        scope: PrintScope.selection,
                      ),
                    );
                  },
                  child: const Text('Open'),
                ),
              );
            },
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('open_print_settings')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('print_scope_hint')), findsNothing);

      await tester.tap(find.byKey(const Key('print_settings_print')));
      await tester.pumpAndSettle();
      expect(chosen?.scope, PrintScope.selection);
    });

    test('S-F25-S3-print-selection-churn', () async {
      final host = RecordingPrintHost();
      final engine = MockDocumentEngine(initialText: '0123456789' * 5);
      final controller = createTestEditorController(
        engine: engine,
        printHost: host,
      );
      addTearDown(controller.dispose);

      for (var i = 0; i < 100; i++) {
        final start = i % 20;
        final end = start + 5;
        controller.selectionController.selectDocRange(
          DocRange(
            anchor: DocPosition(runId: engine.defaultRunId, offset: start),
            focus: DocPosition(runId: engine.defaultRunId, offset: end),
          ),
        );
        final result = await controller.printDocument(
          layout: PrintLayoutSettings.defaults.copyWith(
            scope: PrintScope.selection,
          ),
        );
        expect(result.ok, isTrue);
        expect(engine.lastPrintSelection?.anchor.offset, start);
        expect(engine.lastPrintSelection?.focus.offset, end);
      }
      expect(host.calls, hasLength(100));
    }, timeout: const Timeout(Duration(seconds: 30)));
  });
}
