import 'dart:typed_data';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
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
  group('F10.S3 Wrap and position', () {
    testWidgets('I-F10-S3-wrap-square commits wrap via engine', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

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

      await controller.setSelectedImageWrap(1);
      await settleEngineStyle(tester);

      expect(engine.lastImageWrap, 'square');
      expect(controller.sessionController.statusText, contains('Square'));
    });

    testWidgets('I-F10-S3-move-image commits anchor via engine', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

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
      controller.beginImageMove(const Offset(80, 80));
      controller.updateImageMove(const Offset(120, 100));
      await controller.commitImageMove();
      await settleEngineStyle(tester);

      expect(engine.lastImageAnchorX, 40);
      expect(engine.lastImageAnchorY, 20);
      expect(engine.lastImageWrap, 'square');
      expect(controller.sessionController.statusText, contains('moved'));
    });
  });
}
