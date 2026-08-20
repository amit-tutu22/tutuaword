import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/compare_diff.dart';
import 'package:tutuaword/bridge/document_picker.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F17.S4 Grammar, compare, restrict', () {
    testWidgets('I-F17-S4-proofing-runs-spell-and-grammar', (tester) async {
      final engine = MockDocumentEngine(
        initialText: 'Teh team could of done alot  better.',
      );
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.tap(find.byKey(const Key('spell_check')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(controller.spellMisspellings, contains('Teh'));
      expect(controller.grammarIssues, isNotEmpty);
      expect(
        controller.grammarIssues.any((m) => m.contains('could have')),
        isTrue,
      );
    });

    testWidgets('I-F17-S4-grammar-check-direct-api', (tester) async {
      final engine = MockDocumentEngine(
        initialText: 'I could of finished sooner.',
      );
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.grammarCheckDocument();

      expect(controller.grammarIssues, isNotEmpty);
      expect(controller.sessionController.statusText, contains('issue'));
    });

    test('U-compare-line-diff-counts-insertions-and-deletions', () {
      final diff = compareDocumentLines('Alpha\nBeta', 'Alpha\nGamma');
      expect(diff.insertionCount, 1);
      expect(diff.deletionCount, 1);
      expect(diff.summary, 'insertions:1 deletions:1');
      expect(
        diff.noteworthy.map((c) => '${c.kind.name}:${c.text}').toList(),
        ['delete:Beta', 'insert:Gamma'],
      );
    });

    testWidgets('I-F17-S4-compare-identical-text', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Same text');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      debugDocumentPickerOverride = ({
        required dialogTitle,
        required allowedExtensions,
        required type,
      }) async {
        return PickedDocumentFile(
          bytes: Uint8List.fromList(utf8.encode('Same text')),
          path: 'other.txt',
          name: 'other.txt',
        );
      };
      addTearDown(() => debugDocumentPickerOverride = null);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('compare_documents')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('compare_documents')));
      await tester.pumpAndSettle();

      expect(controller.compareSummary, 'insertions:0 deletions:0');
      expect(controller.sessionController.statusText, contains('Compare complete'));
      expect(find.byKey(const Key('compare_results_dialog')), findsOneWidget);
      expect(find.byKey(const Key('compare_results_identical')), findsOneWidget);
      await tester.tap(find.byKey(const Key('compare_results_close')));
      await tester.pumpAndSettle();
    });

    testWidgets('I-F17-S4-compare-different-text-shows-dialog', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Alpha\nBeta');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      debugDocumentPickerOverride = ({
        required dialogTitle,
        required allowedExtensions,
        required type,
      }) async {
        return PickedDocumentFile(
          bytes: Uint8List.fromList(utf8.encode('Alpha\nGamma')),
          path: 'revised.txt',
          name: 'revised.txt',
        );
      };
      addTearDown(() => debugDocumentPickerOverride = null);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('compare_documents')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('compare_documents')));
      await tester.pumpAndSettle();

      expect(controller.compareSummary, 'insertions:1 deletions:1');
      expect(find.byKey(const Key('compare_results_dialog')), findsOneWidget);
      expect(find.byKey(const Key('compare_results_list')), findsOneWidget);
      expect(find.textContaining('− Beta'), findsOneWidget);
      expect(find.textContaining('+ Gamma'), findsOneWidget);
    });

    testWidgets('I-F17-S4-compare-different-text', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Alpha\nBeta');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.compareWithText('Alpha\nGamma');

      expect(controller.compareSummary, 'insertions:1 deletions:1');
    });

    testWidgets('I-F17-S4-restrict-editing-toggle', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Protected doc');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      expect(controller.documentReadOnly, isFalse);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('restrict_editing')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('restrict_editing')));
      await tester.pumpAndSettle();

      expect(controller.documentReadOnly, isTrue);
      expect(controller.sessionController.statusText, contains('restricted'));

      await tester.tap(find.byKey(const Key('restrict_editing')));
      await tester.pumpAndSettle();

      expect(controller.documentReadOnly, isFalse);
      expect(controller.sessionController.statusText, contains('allowed'));
    });
  });

  testWidgets('ReviewTab wires compare and restrict buttons', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ReviewTab(controller: controller),
        ),
      ),
    );

    expect(find.byKey(const Key('compare_documents')), findsOneWidget);
    expect(find.byKey(const Key('restrict_editing')), findsOneWidget);
  });
}
