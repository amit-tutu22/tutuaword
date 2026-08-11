import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/equation_omml.dart';
import 'package:tutuaword/ui/equation_dialog.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  testWidgets('I-F14-S3-insert-equation ribbon opens dialog and stores OMML', (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    expect(engine.lastOfficeMathXml, isNull);

    await pumpRibbonTab(tester, InsertTab(controller: controller));
    await tester.scrollUntilVisible(
      find.byKey(const Key('insert_equation')),
      120,
      scrollable: find.byType(Scrollable),
    );
    await tester.tap(find.byKey(const Key('insert_equation')));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('equation_dialog')), findsOneWidget);

    await tester.enterText(find.byKey(const Key('equation_text')), 'πr');
    await tester.tap(find.byKey(const Key('equation_ok')));
    await tester.pumpAndSettle();
    await settleEngineStyle(tester);

    final xml = engine.lastOfficeMathXml;
    expect(xml, isNotNull);
    expect(xml, contains('<m:oMath'));
    expect(xml, contains('π'));
    expect(controller.sessionController.statusText, contains('Equation inserted'));
  });

  testWidgets('I-F14-S3-equation-fraction builds OMML from palette', (tester) async {
    EquationModel? result;
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) {
            return TextButton(
              key: const Key('open_equation_dialog'),
              onPressed: () async {
                result = await EquationDialog.show(
                  context,
                  initial: EquationModel.fraction(),
                );
              },
              child: const Text('Open'),
            );
          },
        ),
      ),
    );

    await tester.tap(find.byKey(const Key('open_equation_dialog')));
    await tester.pumpAndSettle();

    await tester.enterText(find.byKey(const Key('equation_numerator')), '1');
    await tester.enterText(find.byKey(const Key('equation_denominator')), '2');
    await tester.tap(find.byKey(const Key('equation_ok')));
    await tester.pumpAndSettle();

    expect(result, isNotNull);
    final omml = result!.toOmml();
    expect(omml, contains('<m:f>'));
    expect(omml, contains('>1<'));
    expect(omml, contains('>2<'));
  });
}
