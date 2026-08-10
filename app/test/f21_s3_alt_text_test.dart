import 'dart:typed_data';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/image_hit_test.dart';
import 'package:tutuaword/editor/picture_inspector_pane.dart';

import 'editor_test_helpers.dart';

final _png1x1 = Uint8List.fromList([
  0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
  0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
  0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
  0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
  0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
]);

void main() {
  group('F21.S3 Alt text on images', () {
    Future<void> selectMockImage(
      WidgetTester tester,
      EditorController controller,
      MockDocumentEngine engine,
    ) async {
      await controller.insertImageBytes(_png1x1, 'image/png');
      await settleEngineStyle(tester);
      controller.selectImage(
        0,
        ImageBounds(
          imageId: engine.mockImageId!,
          index: 0,
          rect: const Rect.fromLTWH(72, 72, 72, 72),
        ),
      );
    }

    testWidgets('I-F21-S3-alt-text-field shows inspector and applies alt text',
        (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await selectMockImage(tester, controller, engine);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: PictureInspectorPane(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('picture_inspector_pane')), findsOneWidget);
      expect(find.byKey(const Key('picture_alt_text_field')), findsOneWidget);

      await tester.enterText(
        find.byKey(const Key('picture_alt_text_field')),
        'Company logo',
      );
      await tester.tap(find.byKey(const Key('picture_alt_text_apply')));
      await tester.pumpAndSettle();
      await settleEngineStyle(tester);

      expect(engine.lastImageAltText, 'Company logo');
      expect(controller.selectedImageAltText, 'Company logo');
      expect(controller.sessionController.statusText, contains('Alt text'));
    });

    testWidgets('I-F21-S3-alt-text-field appears in document view when selected',
        (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await selectMockImage(tester, controller, engine);
      await pumpTestDocumentView(tester, controller);

      expect(find.byType(PictureInspectorPane), findsOneWidget);
    });

    testWidgets('U-F21-S3-set-alt-text via controller', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await selectMockImage(tester, controller, engine);
      await controller.setSelectedImageAltText('Diagram of workflow');
      await settleEngineStyle(tester);

      expect(engine.lastImageAltText, 'Diagram of workflow');
      expect(engine.fetchImageAltText(engine.mockImageId!), 'Diagram of workflow');
    });
  });
}
