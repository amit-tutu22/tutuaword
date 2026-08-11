import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/chart_data.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/chart_data_dialog.dart';

import 'editor_test_helpers.dart';

void main() {
  testWidgets('I-F13-S3-chart-data-editor applies SetChartData', (tester) async {
    final engine = MockDocumentEngine();
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    await controller.insertChart(chartType: EditorController.chartColumn);
    await settleEngineStyle(tester);

    final chartId = engine.latestChartId();
    expect(chartId, isNotNull);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Builder(
            builder: (context) {
              return TextButton(
                key: const Key('open_chart_editor'),
                onPressed: () {
                  controller.editChartData(context, shapeId: chartId);
                },
                child: const Text('Edit'),
              );
            },
          ),
        ),
      ),
    );

    await tester.tap(find.byKey(const Key('open_chart_editor')));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('chart_data_dialog')), findsOneWidget);

    await tester.enterText(
      find.byKey(const Key('chart_data_category_0')),
      'North',
    );
    await tester.enterText(
      find.byKey(const Key('chart_data_value_0_0')),
      '9.5',
    );
    await tester.tap(find.byKey(const Key('chart_data_ok')));
    await tester.pumpAndSettle();
    await settleEngineStyle(tester);

    final data = engine.lastChartData;
    expect(data, isNotNull);
    expect(data!['categories'], contains('North'));
    final series = data['series'] as List;
    final first = series.first as Map;
    expect((first['values'] as List).first, 9.5);
    expect(controller.sessionController.statusText, contains('Chart data'));
  });

  testWidgets('ChartDataDialog returns edited model', (tester) async {
    ChartDataModel? result;
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) {
            return TextButton(
              onPressed: () async {
                result = await ChartDataDialog.show(
                  context,
                  initial: ChartDataModel.sample(),
                );
              },
              child: const Text('Open'),
            );
          },
        ),
      ),
    );

    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('chart_data_add_category')));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('chart_data_category_4')), findsOneWidget);
    await tester.tap(find.byKey(const Key('chart_data_ok')));
    await tester.pumpAndSettle();

    expect(result, isNotNull);
    expect(result!.categories.length, 5);
  });
}
