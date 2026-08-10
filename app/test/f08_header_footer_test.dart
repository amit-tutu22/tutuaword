import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/document_edit_zone.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F08 Header & footer', () {
    testWidgets('I-F08-S1-edit-header opens band and typing survives save', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.scrollUntilVisible(
        find.text('Header'),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.text('Header'));
      await settleEngineStyle(tester);

      expect(controller.editZone, DocumentEditZone.header);
      expect(controller.sessionController.statusText, contains('Editing header'));

      final seed = engine.fetchHeaderFooterSeedRun(isHeader: true);
      expect(seed, isNotNull);
      expect(controller.selectionController.caretRunId, seed);

      await controller.insertGlyphCharacter('C');
      await settleEngineStyle(tester);
      await controller.insertGlyphCharacter('o');
      await settleEngineStyle(tester);

      expect(engine.fetchTextRange(seed!, 0, seed, 2), 'Co');

      final saved = engine.saveDocumentBytes();
      expect(saved, isNotNull);
      engine.newDocument();
      engine.openDocumentBytes(saved!);
      expect(engine.headerText, 'Co');
    });

    testWidgets('I-F08-S2-page-number inserts field in header edit zone', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.openHeaderEdit();
      await settleEngineStyle(tester);

      await controller.insertPageNumberField();
      await settleEngineStyle(tester);

      expect(controller.sessionController.statusText, contains('Page number inserted'));
      expect(controller.editZone, DocumentEditZone.header);
      final seed = engine.fetchHeaderFooterSeedRun(isHeader: true);
      expect(seed, isNotNull);
      expect(engine.fetchTextRange(seed!, 0, seed, 1), '1');
    });

    testWidgets('I-F08-S2-page-number from body does not corrupt body text', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Amit kumar');
      await settleEngineStyle(tester);
      expect(controller.documentText.toLowerCase(), contains('amit kumar'));

      await controller.insertPageNumberField();
      await settleEngineStyle(tester);

      expect(controller.editZone, DocumentEditZone.footer);
      expect(controller.documentText.toLowerCase(), contains('amit kumar'));
      expect(controller.documentText.startsWith('1Amit'), isFalse);
      final footerSeed = engine.fetchHeaderFooterSeedRun(isHeader: false);
      expect(footerSeed, isNotNull);
      expect(engine.fetchTextRange(footerSeed!, 0, footerSeed, 1), '1');
    });

    testWidgets('I-F08-S3-first-page and odd-even toggles update state', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      expect(controller.differentFirstPage, isFalse);
      expect(controller.evenAndOddHeaders, isFalse);

      await tester.binding.setSurfaceSize(const Size(1400, 120));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await pumpRibbonTab(
        tester,
        InsertTab(controller: controller),
        size: const Size(1400, 120),
      );

      await tester.scrollUntilVisible(
        find.byIcon(Icons.looks_one_outlined),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byIcon(Icons.looks_one_outlined));
      await settleEngineStyle(tester);
      expect(controller.differentFirstPage, isTrue);
      expect(controller.sessionController.statusText, contains('Different first page on'));

      await tester.scrollUntilVisible(
        find.byIcon(Icons.view_week_outlined),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byIcon(Icons.view_week_outlined));
      await settleEngineStyle(tester);
      expect(controller.evenAndOddHeaders, isTrue);
      expect(engine.fetchEvenAndOddHeadersEnabled(), isTrue);
      expect(controller.sessionController.statusText, contains('Odd & even headers on'));
    });

    testWidgets('I-F08-S4-link-to-previous visible and toggles in header zone', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.binding.setSurfaceSize(const Size(1400, 120));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await pumpRibbonTab(
        tester,
        InsertTab(controller: controller),
        size: const Size(1400, 120),
      );
      expect(find.byIcon(Icons.link), findsNothing);

      controller.insertSectionBreak();
      await settleEngineStyle(tester);
      controller.selectionController.setCaret(engine.defaultRunId, 0, page: 1);
      await controller.openHeaderEdit();
      await settleEngineStyle(tester);

      await pumpRibbonTab(
        tester,
        InsertTab(controller: controller),
        size: const Size(1400, 120),
      );
      expect(find.byIcon(Icons.link), findsOneWidget);
      expect(controller.headerFooterLinked, isTrue);

      await controller.setHeaderFooterLinked(false);
      await settleEngineStyle(tester);

      expect(controller.headerFooterLinked, isFalse);
      expect(engine.fetchHeaderFooterLinked(isHeader: true, pageIndex: 1), isFalse);
      expect(controller.sessionController.statusText, contains('Unlinked from previous'));
    });
  });
}
