import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/bridge/native_event_router.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/formatting_marks.dart';
import 'package:tutuaword/editor/text_to_speech.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Cross-phase: formatting marks without FFI', () {
    test('marks come from snapshot, not caretAtPosition', () {
      final engine = _CountingCaretEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.toggleFormattingMarks();

      final snapshot = DisplayListSnapshot(
        version: 1,
        pageWidth: 612,
        pageHeight: 792,
        atlasPixels: Uint8List(0),
        atlasWidth: 0,
        atlasHeight: 0,
        glyphOffsets: Float32List(0),
        glyphSrcRects: Float32List(0),
        glyphColors: Int32List(0),
        rectBatch: Float32List(0),
        rectColors: Int32List(0),
        pathPoints: Float32List(0),
        pathColors: Int32List(0),
        imageTransforms: Float32List(0),
        imageSizes: Float32List(0),
        imageAssetIds: const [],
        imageIds: const [],
        imagePayloads: const [],
        imageRotations: Float32List(0),
        imageOpacities: Float32List(0),
        imageCropRects: Float32List(0),
        shapeIds: const [],
        shapeRects: Float32List(0),
        formattingMarks: const [
          FormattingMark(
            kind: FormattingMarkKind.space,
            x: 10,
            y: 20,
            height: 12,
          ),
          FormattingMark(
            kind: FormattingMarkKind.paragraph,
            x: 40,
            y: 20,
            height: 12,
          ),
        ],
      );

      final marks = controller.formattingMarksForPage(0, snapshot);
      expect(marks, hasLength(2));
      expect(engine.caretAtPositionCalls, 0);
    });
  });

  group('Cross-phase: edit timeout honesty', () {
    tearDown(() {
      NativeEventRouter.instance.reset();
    });

    test('awaitEditCompletion returns false on timeout', () async {
      final router = NativeEventRouter.instance;
      router.reset();
      router.attachPump(() => 0);

      // NativeEngine needs a loaded library; exercise the timeout contract via
      // the same router path awaitEditCompletion uses.
      final wait = router.waitFor(4242, timeout: const Duration(milliseconds: 30));
      await expectLater(wait, throwsA(isA<TimeoutException>()));
      expect(router.pendingCount, 0,
          reason: 'timed-out request id must be removed from the router');
    });
  });

  group('Cross-phase: Read Aloud chunk cancel', () {
    testWidgets('long document is spoken in chunks and stop cancels remainder',
        (tester) async {
      final tts = RecordingTextToSpeech(completeImmediately: false);
      final para = 'word ' * 800; // ~4000 chars → multiple chunks
      final body = '$para\n\n$para\n\n$para';
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: body),
        textToSpeech: tts,
      );
      addTearDown(controller.dispose);
      await controller.selectAll();

      final speakFuture = controller.readAloudSelection();
      await tester.pump();
      expect(controller.isReadingAloud, isTrue);
      expect(tts.spoken, isNotEmpty);
      expect(tts.spoken.first.length, lessThanOrEqualTo(3000));

      await controller.stopReadAloud();
      await speakFuture;
      expect(controller.isReadingAloud, isFalse);
      // Without cancel, multiple paragraph chunks would all complete.
      expect(tts.spoken.length, lessThan(3));
    });
  });
}

class _CountingCaretEngine extends MockDocumentEngine {
  _CountingCaretEngine() : super(initialText: 'A B C');

  int caretAtPositionCalls = 0;

  @override
  CaretGeometry? caretAtPosition(int page, String runId, int charOffset) {
    caretAtPositionCalls++;
    return super.caretAtPosition(page, runId, charOffset);
  }
}
