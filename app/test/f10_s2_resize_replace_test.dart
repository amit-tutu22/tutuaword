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

final _png2x2 = Uint8List.fromList([
  0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
  0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x08, 0x06, 0x00, 0x00, 0x00, 0x72,
  0xB6, 0x0D, 0x24, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x60,
  0x60, 0x60, 0x00, 0x00, 0x00, 0x04, 0x00, 0x01, 0x5C, 0xCD, 0xFF, 0x69, 0x00, 0x00, 0x00,
  0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
]);

void main() {
  group('F10.S2 Resize and replace', () {
    testWidgets('I-F10-S2-resize-image commits size via engine', (tester) async {
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
      controller.beginImageResize(ImageResizeHandle.bottomRight);
      controller.updateImageResize(const Offset(200, 160), lockAspectRatio: false);
      await controller.commitImageResize();
      await settleEngineStyle(tester);

      expect(engine.lastImageDisplayWidth, 128);
      expect(engine.lastImageDisplayHeight, 88);
      expect(controller.sessionController.statusText, contains('resized'));
    });

    testWidgets('I-F10-S2-replace-picture keeps wrap metadata', (tester) async {
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

      await controller.replaceSelectedImageBytes(_png2x2, 'image/png');
      await settleEngineStyle(tester);

      expect(engine.lastReplacedImageBytes, _png2x2);
      expect(engine.lastImageWrap, 'inline');
      expect(controller.sessionController.statusText, contains('replaced'));
    });
  });
}
