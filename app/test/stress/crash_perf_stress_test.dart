import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/formatting_marks.dart';
import 'package:tutuaword/editor/page_snapshot_lru.dart';

import '../editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('S-crash-perf multipage typing with ¶ marks', () {
    test('typing burst never probes caret for formatting marks', () async {
      final engine = _CountingCaretEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.toggleFormattingMarks();
      controller.setDisplayListForTest(
        _pageBytesWithMarks(),
        pageCount: 50,
      );

      final snapshot = DisplayListSnapshot.fromBytes(_pageBytesWithMarks());
      expect(snapshot.formattingMarks, isNotEmpty);
      expect(controller.pageCount, 50);

      for (var i = 0; i < 60; i++) {
        await controller.insertGlyphCharacter(i.isEven ? 'x' : ' ');
        final before = engine.caretAtPositionCalls;
        final marks = controller.formattingMarksForPage(0, snapshot);
        expect(marks, isNotEmpty);
        expect(
          engine.caretAtPositionCalls,
          before,
          reason: '¶ overlay must not call caretAtPosition',
        );
      }
    });
  });

  group('S-crash-perf image-heavy scroll LRU', () {
    test('evicts farthest pages once cache exceeds 32', () {
      final cached = List<int>.generate(50, (i) => i);
      final victims = pagesToEvictForLru(
        cachedPages: cached,
        anchorPage: 40,
        maxCached: 32,
      );
      expect(victims, hasLength(18));
      expect(victims, containsAll([0, 1, 2]));
      expect(victims, isNot(contains(40)));
      expect(victims, isNot(contains(39)));

      final retained = cached.toSet()..removeAll(victims);
      expect(retained.length, 32);
      expect(retained.contains(40), isTrue);
    });

    testWidgets('DocumentView stays interactive while scrolling 50 pages',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(
        fakeGlyphDisplayList(),
        pageCount: 50,
      );

      await tester.binding.setSurfaceSize(const Size(900, 700));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DocumentView(controller: controller),
          ),
        ),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));

      final scrollable = find.byType(Scrollable);
      expect(scrollable, findsWidgets);
      final position =
          tester.state<ScrollableState>(scrollable.first).position;

      // Jump (no drag timers) through the document — must not hang or throw.
      for (final fraction in [0.2, 0.4, 0.6, 0.8, 1.0]) {
        position.jumpTo(position.maxScrollExtent * fraction);
        await tester.pump(const Duration(milliseconds: 16));
      }
      expect(controller.pageCount, 50);
      expect(tester.takeException(), isNull);

      // Clear any deferred page-load callbacks before dispose.
      await tester.pump(const Duration(seconds: 1));
    });
  });
}

class _CountingCaretEngine extends MockDocumentEngine {
  _CountingCaretEngine() : super(initialText: 'Stress');

  int caretAtPositionCalls = 0;

  @override
  CaretGeometry? caretAtPosition(int page, String runId, int charOffset) {
    caretAtPositionCalls++;
    return super.caretAtPosition(page, runId, charOffset);
  }
}

Uint8List _pageBytesWithMarks() {
  final writer = _ByteWriter();
  writer.writeU32(8);
  writer.writeU64(1);
  writer.writeF32(612);
  writer.writeF32(792);
  writer.writeU32(0); // glyphs
  writer.writeU32(0); // rects
  writer.writeU32(0); // paths
  writer.writeU32(0); // images
  writer.writeU32(0); // shapes
  writer.writeU32(2); // marks
  writer.writeF32(10);
  writer.writeF32(20);
  writer.writeF32(12);
  writer.writeF32(40);
  writer.writeF32(20);
  writer.writeF32(12);
  writer.writeBytes([0, 2]);
  return writer.toBytes();
}

class _ByteWriter {
  final _bytes = BytesBuilder();

  void writeU32(int value) {
    final data = ByteData(4)..setUint32(0, value, Endian.little);
    _bytes.add(data.buffer.asUint8List());
  }

  void writeU64(int value) {
    final data = ByteData(8)..setUint64(0, value, Endian.little);
    _bytes.add(data.buffer.asUint8List());
  }

  void writeF32(double value) {
    final data = ByteData(4)..setFloat32(0, value, Endian.little);
    _bytes.add(data.buffer.asUint8List());
  }

  void writeBytes(List<int> bytes) {
    _bytes.add(bytes);
  }

  Uint8List toBytes() => _bytes.toBytes();
}
