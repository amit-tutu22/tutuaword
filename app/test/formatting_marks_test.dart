import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/formatting_marks.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Show formatting marks (¶)', () {
    test('U-formatting-marks-from-snapshot', () {
      final snapshot = _snapshotWithMarks(const [
        FormattingMark(
          kind: FormattingMarkKind.space,
          x: 1,
          y: 2,
          height: 10,
        ),
        FormattingMark(
          kind: FormattingMarkKind.tab,
          x: 3,
          y: 2,
          height: 10,
        ),
        FormattingMark(
          kind: FormattingMarkKind.paragraph,
          x: 5,
          y: 2,
          height: 10,
        ),
      ]);
      final marks = formattingMarksFromSnapshot(snapshot);
      expect(marks.where((m) => m.kind == FormattingMarkKind.space), hasLength(1));
      expect(marks.where((m) => m.kind == FormattingMarkKind.tab), hasLength(1));
      expect(
        marks.where((m) => m.kind == FormattingMarkKind.paragraph),
        hasLength(1),
      );
    });

    test('U-formatting-marks-toggle', () {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      expect(controller.showFormattingMarks, isFalse);
      controller.toggleFormattingMarks();
      expect(controller.showFormattingMarks, isTrue);
      expect(
        controller.sessionController.statusText,
        contains('Formatting marks shown'),
      );
      controller.toggleFormattingMarks();
      expect(controller.showFormattingMarks, isFalse);
    });

    testWidgets('I-formatting-marks-from-home-eye', (tester) async {
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: 'Hi there'),
      );
      addTearDown(controller.dispose);

      await pumpWideRibbon(
        tester,
        SizedBox(height: 140, child: HomeTab(controller: controller)),
      );

      await tester.tap(find.byKey(const Key('show_formatting_marks')));
      await tester.pumpAndSettle();
      expect(controller.showFormattingMarks, isTrue);

      final snapshot = _snapshotWithMarks(const [
        FormattingMark(
          kind: FormattingMarkKind.paragraph,
          x: 10,
          y: 20,
          height: 12,
        ),
      ]);
      expect(controller.formattingMarksForPage(0, snapshot), isNotEmpty);
      expect(controller.formattingMarksForPage(0, null), isEmpty);

      await tester.tap(find.byKey(const Key('show_formatting_marks')));
      await tester.pumpAndSettle();
      expect(controller.showFormattingMarks, isFalse);
      expect(controller.formattingMarksForPage(0, snapshot), isEmpty);
    });
  });
}

DisplayListSnapshot _snapshotWithMarks(List<FormattingMark> marks) {
  final empty = DisplayListSnapshot.empty();
  return DisplayListSnapshot(
    version: empty.version,
    pageWidth: empty.pageWidth,
    pageHeight: empty.pageHeight,
    atlasPixels: empty.atlasPixels,
    atlasWidth: empty.atlasWidth,
    atlasHeight: empty.atlasHeight,
    glyphOffsets: empty.glyphOffsets,
    glyphSrcRects: empty.glyphSrcRects,
    glyphColors: empty.glyphColors,
    rectBatch: empty.rectBatch,
    rectColors: empty.rectColors,
    pathPoints: empty.pathPoints,
    pathColors: empty.pathColors,
    imageTransforms: empty.imageTransforms,
    imageSizes: empty.imageSizes,
    imageAssetIds: empty.imageAssetIds,
    imageIds: empty.imageIds,
    imagePayloads: empty.imagePayloads,
    imageRotations: empty.imageRotations,
    imageOpacities: empty.imageOpacities,
    imageCropRects: empty.imageCropRects,
    shapeIds: empty.shapeIds,
    shapeRects: empty.shapeRects,
    formattingMarks: marks,
  );
}
