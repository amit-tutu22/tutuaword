import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/bridge/native_event_router.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';

import 'editor_test_helpers.dart';

/// Holds each edit open until its gate is released, so a test can observe the
/// controller while the worker round-trip is still outstanding.
class _GatedEngine extends MockDocumentEngine {
  final List<Completer<void>> gates = [];

  @override
  Future<bool> tryInsertTextAsync(String runId, int offset, String text) async {
    final gate = Completer<void>();
    gates.add(gate);
    await gate.future;
    return super.tryInsertTextAsync(runId, offset, text);
  }

  void releaseAll() {
    for (final gate in gates) {
      if (!gate.isCompleted) gate.complete();
    }
  }
}

/// Gates every insert like [_GatedEngine], but reports the edit at
/// [failIndex] as failed so a test can observe optimistic-caret rollback.
class _FlakyEngine extends MockDocumentEngine {
  final List<Completer<void>> gates = [];
  int failIndex = -1;
  int _dispatched = 0;

  @override
  Future<bool> tryInsertTextAsync(String runId, int offset, String text) async {
    final index = _dispatched++;
    final gate = Completer<void>();
    gates.add(gate);
    await gate.future;
    if (index == failIndex) return false;
    return super.tryInsertTextAsync(runId, offset, text);
  }
}

/// Reports no hit on the last page while insisting that page is fresh — the
/// genuinely-empty case, which must still resolve a caret.
class _EmptyPageEngine extends MockDocumentEngine {
  @override
  HitTestResult? hitTestPage(int page, double x, double y) =>
      page == 3 ? null : super.hitTestPage(page, x, y);
}

/// Counts the engine reads a refresh performs, and — unlike
/// [MockDocumentEngine] — serves a non-empty per-page display list so the
/// incremental refresh path is exercised.
class _CountingEngine extends MockDocumentEngine {
  int displayListFetches = 0;
  int pageFetches = 0;
  int atlasFetches = 0;
  int atlasGenerationQueries = 0;
  int textFetches = 0;
  int atlasGeneration = 1;

  void resetCounts() {
    displayListFetches = 0;
    pageFetches = 0;
    atlasFetches = 0;
    atlasGenerationQueries = 0;
    textFetches = 0;
  }

  @override
  int? fetchAtlasGeneration() {
    atlasGenerationQueries++;
    return atlasGeneration;
  }

  @override
  DisplayListData? fetchDisplayList() {
    displayListFetches++;
    return super.fetchDisplayList();
  }

  @override
  PageDisplayListData? fetchPageDisplayList(int page) {
    pageFetches++;
    return PageDisplayListData(
      bytes: Uint8List.fromList(const [1, 2, 3, 4]),
      version: super.fetchDisplayList()!.version,
      pageWidth: pageWidth,
      pageHeight: pageHeight,
    );
  }

  @override
  AtlasData? fetchAtlas() {
    atlasFetches++;
    final wire = Uint8List(28);
    ByteData.sublistView(wire, 20, 24).setUint32(0, 4, Endian.little);
    return AtlasData(generation: atlasGeneration, bytes: wire, width: 2, height: 2);
  }

  @override
  String? fetchDocumentText() {
    textFetches++;
    return super.fetchDocumentText();
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('P0-1 typing is event-driven, not await-blocking', () {
    test('edit completion budget is interactive', () {
      expect(kEditCompletionTimeout, lessThanOrEqualTo(const Duration(seconds: 2)));
    });

    test('caret advances before the worker acknowledges the edit', () async {
      final engine = _GatedEngine();
      final controller = EditorController.forTest(engine: engine);
      addTearDown(controller.dispose);
      controller.ensureGlyphCaret();

      unawaited(controller.insertGlyphCharacter('a'));
      expect(controller.caretOffset, 1);
      unawaited(controller.insertGlyphCharacter('b'));
      expect(controller.caretOffset, 2);
      expect(engine.text, isEmpty, reason: 'both edits are still in flight');

      engine.releaseAll();
      await controller.ensureLayoutReady();
      expect(controller.documentText, 'ab');
    });

    test('a stalled edit does not stall the next keystroke', () async {
      final engine = _GatedEngine();
      final controller = EditorController.forTest(engine: engine);
      addTearDown(controller.dispose);
      controller.ensureGlyphCaret();

      unawaited(controller.insertGlyphCharacter('a'));
      for (final ch in 'bcde'.split('')) {
        unawaited(controller.insertGlyphCharacter(ch));
      }
      expect(controller.caretOffset, 5);
      expect(engine.gates.length, 5);

      engine.releaseAll();
      await controller.ensureLayoutReady();
      expect(controller.documentText, 'abcde');
    });

    test('a failed edit does not yank the caret behind a later keystroke',
        () async {
      final engine = _FlakyEngine()..failIndex = 0;
      final controller = EditorController.forTest(engine: engine);
      addTearDown(controller.dispose);
      controller.ensureGlyphCaret();

      unawaited(controller.insertGlyphCharacter('a'));
      unawaited(controller.insertGlyphCharacter('b'));
      expect(controller.caretOffset, 2);

      engine.gates[0].complete();
      await pumpEventQueue();
      expect(controller.caretOffset, 2,
          reason: "the later keystroke's caret outranks the stale rollback");

      engine.gates[1].complete();
      await controller.ensureLayoutReady();
    });

    test('a lone failed edit rolls the caret back', () async {
      final engine = _FlakyEngine()..failIndex = 0;
      final controller = EditorController.forTest(engine: engine);
      addTearDown(controller.dispose);
      controller.ensureGlyphCaret();

      final edit = controller.insertGlyphCharacter('a');
      expect(controller.caretOffset, 1);

      engine.gates[0].complete();
      await edit;
      expect(controller.caretOffset, 0);
    });
  });

  group('P1-8 key press reaches the screen without the worker', () {
    testWidgets('a keystroke repaints while its edit is still in flight',
        (tester) async {
      final engine = _GatedEngine();
      final controller = EditorController.forTest(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();
      controller.ensureGlyphCaret();

      var repaints = 0;
      void countRepaint() => repaints++;
      controller.addListener(countRepaint);
      addTearDown(() => controller.removeListener(countRepaint));

      await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
      await tester.pump();

      expect(repaints, greaterThan(0),
          reason: 'visible feedback lands in the frame after the key press');
      expect(controller.caretOffset, 1);
      expect(engine.text, isEmpty, reason: 'the worker has not acknowledged');

      engine.releaseAll();
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();
    });

    testWidgets('no key press waits on a worker round-trip', (tester) async {
      final engine = _GatedEngine();
      final controller = EditorController.forTest(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();
      controller.ensureGlyphCaret();

      // Every edit stays gated, so any per-key await would hang here until the
      // completion timeout rather than merely being slow.
      var worstMs = 0;
      for (var i = 0; i < 20; i++) {
        final clock = Stopwatch()..start();
        await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
        await tester.pump();
        clock.stop();
        worstMs = clock.elapsedMilliseconds > worstMs
            ? clock.elapsedMilliseconds
            : worstMs;
      }

      expect(controller.caretOffset, 20);
      expect(worstMs, lessThan(kEditCompletionTimeout.inMilliseconds),
          reason: 'key-to-repaint must not be gated on edit completion');

      engine.releaseAll();
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();
    });
  });

  group('event pump drains the native channel', () {
    tearDown(() {
      NativeEventRouter.instance.reset();
      NativeEventRouter.instance.detachPump();
    });

    test('an outstanding wait is pumped until it resolves', () async {
      final router = NativeEventRouter.instance;
      var pumps = 0;
      router.attachPump(() {
        pumps++;
        // The engine answers request 7 only once someone drains the channel.
        if (pumps == 3) router.onEvent(NativeEventTypes.displayListReady, 7);
        return 0;
      });

      final eventType = await router.waitFor(7, timeout: const Duration(seconds: 5));

      expect(eventType, NativeEventTypes.displayListReady);
      expect(pumps, greaterThanOrEqualTo(3));
    });

    test('pumping speeds up while a wait is outstanding', () async {
      final router = NativeEventRouter.instance;
      router.attachPump(() => 0);
      expect(router.pumpInterval, const Duration(milliseconds: 16));

      final wait = router.waitFor(11, timeout: const Duration(milliseconds: 200));
      expect(router.pumpInterval, const Duration(milliseconds: 2),
          reason: 'a pending request must not wait a frame for its event');

      router.onEvent(NativeEventTypes.displayListReady, 11);
      await wait;
      expect(router.pumpInterval, const Duration(milliseconds: 16));
    });

    test('resetting correlation state keeps the pump alive', () {
      final router = NativeEventRouter.instance;
      router.attachPump(() => 0);

      router.reset();

      expect(router.isPumping, isTrue,
          reason: 'callers reset between documents and still need events');
    });

    test('a router with no pump attached starts no timers', () {
      final router = NativeEventRouter.instance;
      unawaited(router.waitFor(3, timeout: const Duration(milliseconds: 10))
          .catchError((_) => 0));

      expect(router.isPumping, isFalse);
    });
  });

  group('P1-4 a page awaiting background reflow is not treated as empty', () {
    test('clicking a stale page leaves the caret where it was', () {
      final engine = MockDocumentEngine();
      final controller = EditorController.forTest(engine: engine);
      addTearDown(controller.dispose);
      controller.ensureGlyphCaret();
      controller.hitTestAt(0, 100, 100);
      final runId = controller.caretRunId;
      final offset = controller.caretOffset;

      engine.setStalePagesForTest({3});
      controller.hitTestAt(3, 100, 100);

      expect(controller.caretRunId, runId);
      expect(controller.caretOffset, offset,
          reason: 'a stale page must not be mistaken for an empty one');
    });

    test('the caret still lands when the page is genuinely empty', () {
      final engine = _EmptyPageEngine();
      final controller = EditorController.forTest(engine: engine);
      addTearDown(controller.dispose);
      controller.ensureGlyphCaret();

      controller.hitTestAt(3, 100, 100);

      expect(controller.caretRunId, isNotNull);
    });
  });

  group('P0-3 refreshes stay off the whole-document path', () {
    late _CountingEngine engine;
    late EditorController controller;

    setUp(() {
      engine = _CountingEngine();
      controller = EditorController.forTest(engine: engine);
      controller.ensureGlyphCaret();
      engine.resetCounts();
    });

    tearDown(() => controller.dispose());

    test('an edit reads back only the dirty page', () async {
      await controller.insertGlyphCharacter('a');
      await controller.ensureLayoutReady();

      expect(engine.pageFetches, 1);
      expect(engine.displayListFetches, 0);
    });

    test('a burst of edits refreshes once', () async {
      await Future.wait([
        for (var i = 0; i < 5; i++) controller.insertGlyphCharacter('x'),
      ]);
      await controller.ensureLayoutReady();

      expect(engine.pageFetches, 1);
      expect(engine.displayListFetches, 0);
      expect(controller.documentText, 'xxxxx');
    });

    test('an unchanged atlas generation is not re-copied', () async {
      await controller.insertGlyphCharacter('a');
      await controller.ensureLayoutReady();
      final pixels = controller.atlasPixels;
      final fetchesAfterFirst = engine.atlasFetches;

      await controller.insertGlyphCharacter('b');
      await controller.ensureLayoutReady();

      expect(controller.atlasGeneration, 1);
      expect(identical(controller.atlasPixels, pixels), isTrue);
      expect(engine.atlasFetches, fetchesAfterFirst,
          reason: 'the generation query must stand in for the pixel copy');
      expect(engine.atlasGenerationQueries, greaterThan(0));
    });

    test('a refresh with no layout change skips the atlas entirely', () {
      controller.setCurrentPage(0);

      expect(engine.atlasFetches, 0);
      expect(engine.displayListFetches, 0);
    });

    test('document text crosses FFI only when a caller reads it', () async {
      await controller.insertGlyphCharacter('a');
      await controller.insertGlyphCharacter('b');
      await controller.ensureLayoutReady();
      expect(engine.textFetches, 0);

      expect(controller.documentText, 'ab');
      expect(engine.textFetches, 1);
      expect(controller.documentText, 'ab');
      expect(engine.textFetches, 1, reason: 'cached until the next refresh');
    });
  });

  group('P2-11 ensureLayoutReady awaits every in-flight edit', () {
    test('the newest edit completing alone does not release the wait', () async {
      final engine = _GatedEngine();
      final controller = EditorController.forTest(engine: engine);
      addTearDown(controller.dispose);
      controller.ensureGlyphCaret();

      for (final ch in 'abc'.split('')) {
        unawaited(controller.insertGlyphCharacter(ch));
      }
      expect(engine.gates.length, 3);

      var ready = false;
      final wait = controller.ensureLayoutReady().then((_) => ready = true);

      engine.gates.last.complete();
      await pumpEventQueue();
      expect(ready, isFalse, reason: 'two older edits are still outstanding');

      engine.gates[0].complete();
      engine.gates[1].complete();
      await wait;

      expect(ready, isTrue);
      expect(controller.documentText.length, 3);
    });
  });
}
