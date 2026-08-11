import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/native_engine.dart';
import 'package:tutuaword/bridge/native_event_router.dart';
import 'package:tutuaword/editor/editor_controller.dart';

import 'native_ffi_test_helpers.dart';

DocumentSessionStore _isolatedStore(String prefix) {
  return DocumentSessionStore(root: Directory.systemTemp.createTempSync(prefix));
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('R0.2 event router', () {
    tearDown(() {
      NativeEventRouter.instance.reset();
    });

    test('completer completes only on matching request_id', () async {
      final router = NativeEventRouter.instance;
      router.reset();

      final wait42 = router.waitFor(42);
      final wait99 = router.waitFor(99);

      router.onEvent(NativeEventTypes.displayListReady, 99);
      expect(await wait99, NativeEventTypes.displayListReady);
      expect(router.pendingCount, 1);

      router.onEvent(NativeEventTypes.documentSaved, 42);
      expect(await wait42, NativeEventTypes.documentSaved);
      expect(router.pendingCount, 0);
    });

    test('unmatched request_id does not complete other completers', () async {
      final router = NativeEventRouter.instance;
      router.reset();

      final wait7 = router.waitFor(7);
      router.onEvent(NativeEventTypes.error, 8);

      expect(router.pendingCount, 1);
      router.onEvent(NativeEventTypes.displayListReady, 7);
      expect(await wait7, NativeEventTypes.displayListReady);
    });
  });

  group('R0.2 autosave isolation', () {
    tearDown(() async {
      await NativeEngine.shutdownAsync();
    });

    test('typing during autosave completes without cross-talk', () async {
      final store = _isolatedStore('tutuaword_r02_autosave_');
      final controller = EditorController(
        sessionStore: store,
        enableAutosave: false,
      );
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;
      if (!await nativeFfiEventsAvailable()) return;

      await controller.newDocument();
      controller.ensureGlyphCaret();

      await controller.insertGlyphCharacter('A');
      expect(controller.documentText.toLowerCase(), contains('a'));

      final autosaveFuture = controller.performAutosave();
      for (final ch in 'BCDE'.split('')) {
        if (ch.isEmpty) continue;
        await controller.insertGlyphCharacter(ch);
      }

      await autosaveFuture;
      await controller.performAutosave();
      await controller.ensureLayoutReady();

      expect(controller.documentText.toLowerCase(), contains('abcde'));
      expect(controller.nativeEditDepth, 0);

      final snapshot = await store.readAutosave();
      expect(snapshot, isNotNull);
      expect(snapshot!.bytes, isNotEmpty);
    });
  });
}
