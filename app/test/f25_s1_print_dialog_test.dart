import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_print.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/title_bar.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F25.S1 OS print dialog', () {
    test('U-F25-S1-pdf-header-gate', () {
      expect(isPdfHeader(MockDocumentEngine.mockPrintPdf), isTrue);
      expect(isPdfHeader(Uint8List.fromList([0, 1, 2])), isFalse);
      expect(isPdfHeader(Uint8List(0)), isFalse);
    });

    test('U-F25-S1-recording-print-host', () async {
      final host = RecordingPrintHost();
      final bytes = MockDocumentEngine.mockPrintPdf;
      final result = await host.presentPrintDialog(
        pdfBytes: bytes,
        jobName: 'Report',
      );
      expect(result.outcome, PrintDialogOutcome.presented);
      expect(host.calls, hasLength(1));
      expect(host.calls.first.jobName, 'Report');
      expect(isPdfHeader(host.calls.first.bytes), isTrue);
    });

    test('U-F25-S1-mock-engine-print-pdf', () {
      final engine = MockDocumentEngine(initialText: 'Hello');
      final bytes = engine.exportPdfBytesForPrint();
      expect(bytes, isNotNull);
      expect(isPdfHeader(bytes!), isTrue);
      // Structural export remains empty in the mock (save-as PDF path).
      expect(engine.exportPdfBytes(), isEmpty);
    });

    test('I-F25-S1-print-dialog-opens', () async {
      final host = RecordingPrintHost();
      final engine = MockDocumentEngine(initialText: 'Print me');
      final controller = createTestEditorController(
        engine: engine,
        printHost: host,
      );
      addTearDown(controller.dispose);

      final result = await controller.printDocument();
      expect(result.outcome, PrintDialogOutcome.presented);
      expect(host.calls, hasLength(1));
      expect(isPdfHeader(host.calls.first.bytes), isTrue);
      expect(
        controller.sessionController.statusText,
        contains('Print dialog opened'),
      );
    });

    test('I-F25-S1-print-cancel-status', () async {
      final host = RecordingPrintHost()
        ..nextResult = const PrintDialogResult(
          outcome: PrintDialogOutcome.cancelled,
        );
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: 'x'),
        printHost: host,
      );
      addTearDown(controller.dispose);

      final result = await controller.printDocument();
      expect(result.outcome, PrintDialogOutcome.cancelled);
      expect(
        controller.sessionController.statusText,
        contains('Print cancelled'),
      );
    });

    testWidgets('I-F25-S1-title-bar-print', (tester) async {
      final host = RecordingPrintHost();
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: 'Title bar print'),
        printHost: host,
      );
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WordTitleBar(controller: controller),
          ),
        ),
      );
      await tester.pump();

      await tester.tap(find.byTooltip('Print'));
      await tester.pumpAndSettle();
      // F25.S2: settings dialog precedes the OS print host.
      expect(find.byKey(const Key('print_settings_print')), findsOneWidget);
      await tester.tap(find.byKey(const Key('print_settings_print')));
      await tester.pumpAndSettle();

      expect(host.calls, hasLength(1));
      expect(
        controller.sessionController.statusText,
        contains('Print dialog opened'),
      );
    });

    testWidgets('I-F25-S1-shortcut-print', (tester) async {
      final host = RecordingPrintHost();
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: 'Shortcut'),
        printHost: host,
      );
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return Shortcuts(
                shortcuts: const <ShortcutActivator, Intent>{
                  SingleActivator(LogicalKeyboardKey.keyP, meta: true):
                      _TestPrintIntent(),
                },
                child: Actions(
                  actions: <Type, Action<Intent>>{
                    _TestPrintIntent: CallbackAction<_TestPrintIntent>(
                      onInvoke: (_) {
                        controller.printDocument();
                        return null;
                      },
                    ),
                  },
                  child: const Focus(
                    autofocus: true,
                    child: SizedBox(width: 100, height: 100),
                  ),
                ),
              );
            },
          ),
        ),
      );
      await tester.pump();

      await tester.sendKeyDownEvent(LogicalKeyboardKey.meta);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyP);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.meta);
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));

      expect(host.calls, hasLength(1));
    });

    test('S-F25-S1-print-churn', () async {
      final host = RecordingPrintHost();
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: 'stress'),
        printHost: host,
      );
      addTearDown(controller.dispose);

      for (var i = 0; i < 100; i++) {
        final result = await controller.printDocument();
        expect(result.ok, isTrue);
      }
      expect(host.calls, hasLength(100));
      for (final call in host.calls) {
        expect(isPdfHeader(call.bytes), isTrue);
      }
    }, timeout: const Timeout(Duration(seconds: 30)));
  });
}

class _TestPrintIntent extends Intent {
  const _TestPrintIntent();
}
