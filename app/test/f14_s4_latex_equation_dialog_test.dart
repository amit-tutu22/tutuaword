import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  testWidgets('I-F14-S4-latex-equation-insert stores OMML from LaTeX tab', (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    await pumpRibbonTab(tester, InsertTab(controller: controller));
    await tester.scrollUntilVisible(
      find.byKey(const Key('insert_equation')),
      120,
      scrollable: find.byType(Scrollable),
    );
    await tester.tap(find.byKey(const Key('insert_equation')));
    await tester.pumpAndSettle();

    await tester.tap(find.text('LaTeX'));
    await tester.pumpAndSettle();

    await tester.enterText(
      find.byKey(const Key('equation_latex')),
      r'\frac{\pi}{2}',
    );
    await tester.tap(find.byKey(const Key('equation_ok')));
    await tester.pumpAndSettle();
    await settleEngineStyle(tester);

    final xml = engine.lastOfficeMathXml;
    expect(xml, isNotNull);
    expect(xml, contains('<m:f>'));
    expect(xml, contains('π'));
    expect(controller.sessionController.statusText, contains('Equation inserted'));
  });
}
