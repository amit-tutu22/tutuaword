import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/change_case.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/references_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

import 'editor_test_helpers.dart';

Future<void> pumpWideTab(WidgetTester tester, Widget tab) async {
  await tester.binding.setSurfaceSize(const Size(1600, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await pumpRibbonTab(tester, tab, size: const Size(1600, 140));
}

void main() {
  group('Release closeout wiring', () {
    testWidgets('Layout indent and spacing nudge', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpWideTab(tester, LayoutTab(controller: controller));

      final before = controller.spaceBefore;
      await tester.tap(find.byIcon(Icons.format_indent_increase));
      await tester.pump();
      expect(controller.spaceBefore, greaterThanOrEqualTo(0));

      await tester.tap(find.text('Before'));
      await tester.pump();
      expect(controller.spaceBefore, greaterThan(before));
    });

    testWidgets('Insert Text Box button is wired on Insert tab', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpWideTab(tester, InsertTab(controller: controller));
      expect(find.byKey(const Key('insert_text_box_text_group')), findsOneWidget);
    });

    testWidgets('Home No Spacing applies style', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      controller.applyParagraphStyle('No Spacing');
      await settleEngineStyle(tester);
      expect(controller.activeParagraphStyle, 'No Spacing');
    });

    testWidgets('Change case menu transforms selection', (tester) async {
      final engine = MockDocumentEngine(initialText: 'hello world');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      await controller.selectAll();
      await tester.pump();

      await controller.applyChangeCase(ChangeCaseKind.upper);
      await settleEngineStyle(tester);
      expect(engine.text.toUpperCase(), contains('HELLO WORLD'));
    });

    testWidgets('Sort paragraphs A to Z', (tester) async {
      final engine = MockDocumentEngine(initialText: 'zebra\napple\nmango');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.sortParagraphs(ascending: true);
      await settleEngineStyle(tester);
      expect(engine.text.indexOf('apple'), lessThan(engine.text.indexOf('mango')));
      expect(engine.text.indexOf('mango'), lessThan(engine.text.indexOf('zebra')));
    });

    testWidgets('Comment dialog inserts custom body', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Draft');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () => controller.insertComment(context),
              child: const Text('Comment'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Comment'));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('comment_dialog')), findsOneWidget);
      await tester.enterText(find.byKey(const Key('comment_body')), 'Needs review');
      await tester.tap(find.byKey(const Key('comment_ok')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);
      expect(engine.text, contains('[C1]'));
    });

    testWidgets('Compare uses external text', (tester) async {
      final engine = MockDocumentEngine(initialText: 'Alpha');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.compareWithText('Beta');
      expect(controller.sessionController.statusText, contains('Compare complete'));
    });

    testWidgets('References caption disabled without image', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpWideTab(tester, ReferencesTab(controller: controller));
      expect(controller.hasSelectedImage, isFalse);
      await tester.longPress(find.byKey(const Key('references_insert_caption')));
      await tester.pumpAndSettle();
      expect(
        find.text('Select a picture in the document first'),
        findsOneWidget,
      );
    });

    testWidgets('primary tabs have no Coming soon tooltips', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      for (final tab in [
        HomeTab(controller: controller),
        InsertTab(controller: controller),
        LayoutTab(controller: controller),
        ReferencesTab(controller: controller),
        ReviewTab(controller: controller),
      ]) {
        await pumpWideTab(tester, tab);
        expect(find.byTooltip(kComingSoonTooltip), findsNothing);
      }
    });
  });

  group('change_case unit', () {
    test('transformChangeCase variants', () {
      expect(transformChangeCase('hello WORLD', ChangeCaseKind.upper), 'HELLO WORLD');
      expect(transformChangeCase('HELLO', ChangeCaseKind.lower), 'hello');
      expect(
        transformChangeCase('hello world', ChangeCaseKind.capitalizeEachWord),
        'Hello World',
      );
    });
  });
}
