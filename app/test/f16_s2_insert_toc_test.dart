import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/references_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F16.S2 Insert table of contents', () {
    testWidgets('I-F16-S2-insert-toc-from-references-ribbon', (tester) async {
      final engine = MockDocumentEngine();
      engine.setMockOutlineHeadingsForTest([
        (text: 'Introduction', page: 1),
        (text: 'Methods', page: 2),
      ]);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReferencesTab(controller: controller));
      await tester.tap(find.byKey(const Key('insert_table_of_contents')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('Table of Contents'));
      expect(engine.text, contains('Introduction'));
      expect(engine.text, contains('Methods'));
      expect(controller.sessionController.statusText, contains('Table of contents inserted'));
    });

    testWidgets('I-F16-S2-insert-toc-direct-api', (tester) async {
      final engine = MockDocumentEngine();
      engine.setMockOutlineHeadingsForTest([
        (text: 'Chapter 1', page: 1),
      ]);
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return TextButton(
                onPressed: () => controller.insertTableOfContents(context),
                child: const Text('TOC'),
              );
            },
          ),
        ),
      );

      await tester.tap(find.text('TOC'));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.text, contains('Table of Contents'));
      expect(engine.text, contains('Chapter 1'));
    });
  });

  testWidgets('ReferencesTab wires insert table of contents button', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ReferencesTab(controller: controller),
        ),
      ),
    );

    expect(find.byKey(const Key('insert_table_of_contents')), findsOneWidget);
  });
}
