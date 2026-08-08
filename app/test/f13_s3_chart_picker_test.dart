import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/illustration_pickers.dart';

void main() {
  testWidgets('I-F13-S3-chart-picker selects clustered column', (tester) async {
    int? selected;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ChartPicker(onSelected: (value) => selected = value),
        ),
      ),
    );

    await tester.tap(find.byKey(Key('chart_picker_${EditorController.chartColumn}')));
    await tester.pump();

    expect(selected, EditorController.chartColumn);
  });
}
