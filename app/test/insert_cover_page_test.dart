import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('Insert Cover Page', () {
    testWidgets('insertCoverPageContent writes title and page break status', (
      tester,
    ) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.insertCoverPageContent(
        title: 'Annual Report',
        subtitle: 'FY2026',
        author: 'Acme',
      );
      await settleEngineStyle(tester);

      expect(controller.documentText, contains('Annual Report'));
      expect(controller.documentText, contains('FY2026'));
      expect(controller.documentText, contains('Acme'));
      expect(controller.sessionController.statusText, contains('Cover page'));
    });

    testWidgets('Cover Page button opens dialog and inserts', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, InsertTab(controller: controller));
      await tester.tap(find.byKey(const Key('insert_cover_page')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('cover_page_dialog')), findsOneWidget);
      await tester.enterText(
        find.byKey(const Key('cover_page_title')),
        'My Cover',
      );
      await tester.tap(find.byKey(const Key('cover_page_ok')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(controller.documentText, contains('My Cover'));
      expect(controller.sessionController.statusText, contains('Cover page'));
    });

    testWidgets('picture tools show select-first tip when disabled', (
      tester,
    ) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, InsertTab(controller: controller));

      expect(controller.hasSelectedImage, isFalse);
      await tester.longPress(find.byKey(const Key('change_picture')));
      await tester.pumpAndSettle();
      expect(
        find.text('Select a picture in the document first'),
        findsOneWidget,
      );
    });
  });
}
