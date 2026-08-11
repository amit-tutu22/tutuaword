import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/native_event_router.dart';
import 'package:tutuaword/bridge/native_engine.dart';

const _pageMargin = 72.0;
const _firstLineHeight = 11.0;

/// The document's actual first run id. The engine assigns random UUIDs, so a
/// hardcoded id addresses no run: the edit is rejected and the engine answers
/// with an error event rather than a repaint.
String? nativeDefaultRunId(NativeEngine engine) =>
    engine.hitTestPage(0, _pageMargin, _pageMargin + _firstLineHeight)?.runId;

/// True when the native engine applies an edit and reports it back as a
/// repaint. Returns false only when there is no native library to talk to, so
/// callers skip on an unsupported host.
///
/// A loaded engine that cannot round-trip an event is a failure, not a skip.
/// These guards previously swallowed exactly that case, and a broken event
/// correlation left the whole native suite passing without running.
Future<bool> nativeFfiEventsAvailable() async {
  final engine = NativeEngine.load();
  if (engine == null) return false;

  // `tw_init` returns before the worker has laid out the startup document, so
  // hit testing page 0 first would find an empty page. Wait before `reset`,
  // which discards the buffered startup event.
  await NativeEngine.ensureStartupReady(timeout: const Duration(seconds: 5));

  NativeEventRouter.instance.reset();
  final runId = nativeDefaultRunId(engine);
  if (runId == null) {
    fail('engine loaded but no run resolved on page 0 — document is not laid out');
  }
  if (!engine.tryInsertText(runId, 0, 'q')) {
    fail('engine loaded but rejected an insert into its own first run ($runId)');
  }

  try {
    final eventType = await NativeEventRouter.instance.waitFor(
      engine.lastRequestId(),
      timeout: const Duration(seconds: 2),
    );
    if (eventType != NativeEventTypes.displayListReady) {
      fail('edit answered with event type $eventType, expected a repaint');
    }
    return true;
  } on TimeoutException {
    fail(
      'no completion event for the edit: the engine is not being pumped, or '
      'request ids are not surviving the callback boundary',
    );
  }
}
