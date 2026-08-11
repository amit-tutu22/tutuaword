import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/shape_picker.dart';

void main() {
  testWidgets('I-F11-S2-shape-picker selects rectangle', (tester) async {
    int? selected;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ShapePicker(onSelected: (value) => selected = value),
        ),
      ),
    );

    await tester.tap(find.byKey(const Key('shape_picker_rectangle')));
    await tester.pump();

    expect(selected, EditorController.shapeRectangle);
  });
}
