import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/native_event_router.dart';
import 'package:tutuaword/bridge/native_engine.dart';

import 'native_ffi_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('R1.4 async FFI', () {
    tearDown(() {
      NativeEventRouter.instance.reset();
    });

    test('edit FFI returns immediately without blocking', () async {
      final engine = NativeEngine.load();
      if (engine == null) return;
      if (!await nativeFfiEventsAvailable()) return;

      final runId = nativeDefaultRunId(engine)!;
      const iterations = 60;
      final ffiDurations = <Duration>[];

      for (var i = 0; i < iterations; i++) {
        final sw = Stopwatch()..start();
        final code = engine.tryInsertText(runId, 1 + i, 'x') ? 0 : -1;
        sw.stop();
        expect(code, 0, reason: 'fire-and-forget enqueue should succeed');
        ffiDurations.add(sw.elapsed);
      }

      for (final elapsed in ffiDurations) {
        expect(
          elapsed.inMicroseconds,
          lessThan(16 * 1000),
          reason: 'single FFI edit call must not block ≥16 ms (was ${elapsed.inMicroseconds} µs)',
        );
      }

      final requestId = engine.lastRequestId();
      expect(requestId, greaterThan(0));
      final eventType = await NativeEventRouter.instance.waitFor(
        requestId,
        timeout: const Duration(seconds: 5),
      );
      expect(eventType, NativeEventTypes.displayListReady);
    });

    test('60 fire-and-forget inserts complete within 1s', () async {
      final engine = NativeEngine.load();
      if (engine == null) return;
      if (!await nativeFfiEventsAvailable()) return;

      final runId = nativeDefaultRunId(engine)!;
      final ffiSamples = <int>[];

      final burst = Stopwatch()..start();
      for (var i = 0; i < 60; i++) {
        final sw = Stopwatch()..start();
        final code = engine.tryInsertText(runId, 1 + i, 'k') ? 0 : -1;
        sw.stop();
        expect(code, 0);
        ffiSamples.add(sw.elapsed.inMicroseconds);
      }
      burst.stop();

      expect(burst.elapsed.inMilliseconds, lessThan(1000));
      expect(ffiSamples.every((us) => us < 16 * 1000), isTrue);

      final requestId = engine.lastRequestId();
      final eventType = await NativeEventRouter.instance.waitFor(
        requestId,
        timeout: const Duration(seconds: 5),
      );
      expect(eventType, NativeEventTypes.displayListReady);
    });

    test('NativeEventRouter buffers early events before waitFor', () async {
      final router = NativeEventRouter.instance;
      router.reset();

      router.onEvent(NativeEventTypes.displayListReady, 77);
      expect(await router.waitFor(77), NativeEventTypes.displayListReady);
    });
  });
}
