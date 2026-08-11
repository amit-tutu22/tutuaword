import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_print.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/bridge/print_layout_settings.dart';
import 'package:tutuaword/ui/print_settings_dialog.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F25.S4 duplex N-up booklet', () {
    test('U-F25-S4-normalize-pages-per-sheet', () {
      expect(PrintLayoutSettings.normalizePagesPerSheet(0), 1);
      expect(PrintLayoutSettings.normalizePagesPerSheet(2), 2);
      expect(PrintLayoutSettings.normalizePagesPerSheet(3), 4);
      expect(PrintLayoutSettings.normalizePagesPerSheet(6), 6);
      expect(PrintLayoutSettings.normalizePagesPerSheet(12), 16);
    });

    test('U-F25-S4-nup-grid', () {
      expect(PrintLayoutSettings.nupGrid(2), (2, 1));
      expect(PrintLayoutSettings.nupGrid(4), (2, 2));
      expect(PrintLayoutSettings.nupGrid(9), (3, 3));
    });

    test('U-F25-S4-booklet-page-order', () {
      expect(
        PrintLayoutSettings.bookletPageOrder(8),
        [7, 0, 1, 6, 5, 2, 3, 4],
      );
      final padded = PrintLayoutSettings.bookletPageOrder(5);
      expect(padded.length, 8);
      expect(padded.whereType<int>().length, 5);
    });

    test('U-F25-S4-booklet-forces-duplex-long-edge', () {
      final settings = PrintLayoutSettings.defaults.copyWith(
        booklet: true,
        duplex: PrintDuplexMode.simplex,
      );
      expect(settings.effectiveDuplex, PrintDuplexMode.longEdge);
      expect(settings.effectivePagesPerSheet, 2);
      expect(settings.duplexCode, 1);
      expect(settings.toPlatformAttributes()['booklet'], isTrue);
      expect(settings.toPlatformAttributes()['duplex'], 'longEdge');
    });

    test('I-F25-S4-print-passes-sheet-attributes', () async {
      final host = RecordingPrintHost();
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: 'Sheet attrs'),
        printHost: host,
      );
      addTearDown(controller.dispose);

      final result = await controller.printDocument(
        layout: PrintLayoutSettings.defaults.copyWith(
          duplex: PrintDuplexMode.longEdge,
          pagesPerSheet: 4,
        ),
      );
      expect(result.outcome, PrintDialogOutcome.presented);
      expect(host.calls, hasLength(1));
      expect(host.calls.first.attributes['duplex'], 'longEdge');
      expect(host.calls.first.attributes['pagesPerSheet'], 4);
      expect(host.calls.first.attributes['booklet'], isFalse);
    });

    test('I-F25-S4-print-booklet-attributes', () async {
      final host = RecordingPrintHost();
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: 'Booklet'),
        printHost: host,
      );
      addTearDown(controller.dispose);

      final result = await controller.printDocument(
        layout: PrintLayoutSettings.defaults.copyWith(booklet: true),
      );
      expect(result.ok, isTrue);
      expect(host.calls.first.attributes['booklet'], isTrue);
      expect(host.calls.first.attributes['duplex'], 'longEdge');
      expect(host.calls.first.attributes['pagesPerSheet'], 2);
    });

    testWidgets('I-F25-S4-print-settings-sheet-controls', (tester) async {
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
      expect(find.byKey(const Key('print_duplex')), findsOneWidget);
      expect(find.byKey(const Key('print_pages_per_sheet')), findsOneWidget);
      expect(find.byKey(const Key('print_booklet')), findsOneWidget);

      await tester.tap(find.byKey(const Key('print_booklet')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('print_settings_print')));
      await tester.pumpAndSettle();

      expect(chosen?.booklet, isTrue);
      expect(chosen?.effectiveDuplex, PrintDuplexMode.longEdge);
      expect(chosen?.effectivePagesPerSheet, 2);
    });

    test('S-F25-S4-print-sheet-attr-churn', () async {
      final host = RecordingPrintHost();
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: 'stress'),
        printHost: host,
      );
      addTearDown(controller.dispose);

      final variants = [
        PrintLayoutSettings.defaults,
        PrintLayoutSettings.defaults.copyWith(duplex: PrintDuplexMode.longEdge),
        PrintLayoutSettings.defaults.copyWith(pagesPerSheet: 4),
        PrintLayoutSettings.defaults.copyWith(booklet: true),
        PrintLayoutSettings.defaults.copyWith(
          duplex: PrintDuplexMode.shortEdge,
          pagesPerSheet: 2,
        ),
      ];
      for (var i = 0; i < 100; i++) {
        final layout = variants[i % variants.length];
        final result = await controller.printDocument(layout: layout);
        expect(result.ok, isTrue);
        expect(
          host.calls[i].attributes['pagesPerSheet'],
          layout.effectivePagesPerSheet,
        );
        expect(
          host.calls[i].attributes['duplex'],
          layout.effectiveDuplex.name,
        );
      }
      expect(host.calls, hasLength(100));
    }, timeout: const Timeout(Duration(seconds: 30)));
  });
}
