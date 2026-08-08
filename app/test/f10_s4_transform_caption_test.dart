import 'dart:typed_data';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/image_hit_test.dart';

import 'editor_test_helpers.dart';

final _png1x1 = Uint8List.fromList([
  0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
  0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
  0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
  0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
  0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
]);

void main() {
  group('F10.S4 Transform and caption', () {
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

    testWidgets('I-F10-S4-rotate-picture commits transform via engine', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await selectMockImage(tester, controller, engine);
      await controller.rotateSelectedImage();
      await settleEngineStyle(tester);

      expect(engine.lastImageRotationDeg, 90);
      expect(controller.sessionController.statusText, contains('rotated'));
    });

    testWidgets('I-F10-S4-insert-caption commits via engine', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await selectMockImage(tester, controller, engine);
      await controller.insertSelectedImageCaption();
      await settleEngineStyle(tester);

      expect(engine.lastImageCaptionInserted, isTrue);
      expect(controller.sessionController.statusText, contains('Caption'));
    });

    testWidgets('I-F10-S4-compress-picture commits via engine', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await selectMockImage(tester, controller, engine);
      await controller.compressSelectedImage();
      await settleEngineStyle(tester);

      expect(engine.lastInsertedImageMime, 'image/jpeg');
      expect(controller.sessionController.statusText, contains('compressed'));
    });
  });
}
