import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_print.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/bridge/print_layout_settings.dart';
import 'package:tutuaword/ui/print_settings_dialog.dart';
import 'package:tutuaword/ui/title_bar.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F25.S2 print scale and margins', () {
    test('U-F25-S2-resolve-actual-size', () {
      final t = PrintLayoutSettings.defaults.resolve(612, 792);
      expect(t.scale, closeTo(1.0, 1e-5));
      expect(t.tx, closeTo(0.0, 1e-5));
      expect(t.ty, closeTo(0.0, 1e-5));
    });

    test('U-F25-S2-resolve-custom-scale', () {
      final t = PrintLayoutSettings.customScale(50).resolve(612, 792);
      expect(t.scale, closeTo(0.5, 1e-5));
      expect(t.tx, closeTo(612 * 0.25, 1e-3));
      expect(t.ty, closeTo(792 * 0.25, 1e-3));
    });

    test('U-F25-S2-resolve-fit-to-margins', () {
      final t = PrintLayoutSettings.fitToMargins(72).resolve(612, 792);
      final expected = ((612 - 144) / 612) < ((792 - 144) / 792)
          ? (612 - 144) / 612
          : (792 - 144) / 792;
      expect(t.scale, closeTo(expected, 1e-4));
      expect(t.scale, lessThan(1.0));
    });

    test('U-F25-S2-scale-mode-codes', () {
      expect(PrintLayoutSettings.defaults.scaleModeCode, 0);
      expect(PrintLayoutSettings.fitToMargins(36).scaleModeCode, 1);
      expect(PrintLayoutSettings.customScale(125).scaleModeCode, 2);
    });

    test('U-F25-S2-mock-engine-records-layout', () {
      final engine = MockDocumentEngine(initialText: 'layout');
      final layout = PrintLayoutSettings.customScale(75).copyWith(
        marginLeft: 18,
        marginRight: 18,
        marginTop: 18,
        marginBottom: 18,
      );
      final bytes = engine.exportPdfBytesForPrint(layout);
      expect(isPdfHeader(bytes!), isTrue);
      expect(engine.lastPrintLayout?.scalePercent, 75);
      expect(engine.lastPrintLayout?.marginLeft, 18);
    });

    test('I-F25-S2-print-passes-layout', () async {
      final host = RecordingPrintHost();
      final engine = MockDocumentEngine(initialText: 'Scaled');
      final controller = createTestEditorController(
        engine: engine,
        printHost: host,
      );
      addTearDown(controller.dispose);

      final layout = PrintLayoutSettings.fitToMargins(36);
      final result = await controller.printDocument(layout: layout);
      expect(result.outcome, PrintDialogOutcome.presented);
      expect(engine.lastPrintLayout?.scaleMode, PrintScaleMode.fitToMargins);
      expect(engine.lastPrintLayout?.marginLeft, 36);
      expect(host.calls, hasLength(1));
    });

    testWidgets('I-F25-S2-print-settings-dialog', (tester) async {
      PrintLayoutSettings? chosen;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return Scaffold(
                body: TextButton(
                  key: const Key('open_print_settings'),
                  onPressed: () async {
                    chosen = await PrintSettingsDialog.show(context);
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
      expect(find.byKey(const Key('print_settings_dialog')), findsOneWidget);

      await tester.tap(
        find.descendant(
          of: find.byKey(const Key('print_scale_mode')),
          matching: find.text('Custom'),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('print_scale_slider')), findsOneWidget);

      await tester.tap(find.byKey(const Key('print_settings_print')));
      await tester.pumpAndSettle();
      expect(chosen, isNotNull);
      expect(chosen!.scaleMode, PrintScaleMode.customPercent);
    });

    testWidgets('I-F25-S2-print-settings-cancel', (tester) async {
      final host = RecordingPrintHost();
      final engine = MockDocumentEngine(initialText: 'cancel');
      final controller = createTestEditorController(
        engine: engine,
        printHost: host,
      );
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return Scaffold(
                body: TextButton(
                  key: const Key('start_print'),
                  onPressed: () => controller.printDocument(context: context),
                  child: const Text('Print'),
                ),
              );
            },
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('start_print')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('print_settings_cancel')));
      await tester.pumpAndSettle();

      expect(host.calls, isEmpty);
      expect(engine.lastPrintLayout, isNull);
      expect(
        controller.sessionController.statusText,
        contains('Print cancelled'),
      );
    });

    testWidgets('I-F25-S2-title-bar-shows-settings', (tester) async {
      final host = RecordingPrintHost();
      final engine = MockDocumentEngine(initialText: 'Title');
      final controller = createTestEditorController(
        engine: engine,
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
      expect(find.byKey(const Key('print_settings_dialog')), findsOneWidget);

      await tester.tap(find.byKey(const Key('print_settings_print')));
      await tester.pumpAndSettle();
      expect(host.calls, hasLength(1));
      expect(engine.lastPrintLayout, isNotNull);
    });

    test('S-F25-S2-print-layout-churn', () async {
      final host = RecordingPrintHost();
      final engine = MockDocumentEngine(initialText: 'stress');
      final controller = createTestEditorController(
        engine: engine,
        printHost: host,
      );
      addTearDown(controller.dispose);

      final modes = [
        PrintLayoutSettings.defaults,
        PrintLayoutSettings.customScale(50),
        PrintLayoutSettings.uniformMargins(36),
        PrintLayoutSettings.fitToMargins(54),
      ];
      for (var i = 0; i < 100; i++) {
        final layout = modes[i % modes.length];
        final result = await controller.printDocument(layout: layout);
        expect(result.ok, isTrue);
        expect(engine.lastPrintLayout?.scaleMode, layout.scaleMode);
      }
      expect(host.calls, hasLength(100));
    }, timeout: const Timeout(Duration(seconds: 30)));
  });
}
