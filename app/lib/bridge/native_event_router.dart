import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:tutuaword/bridge/event_pump.dart';

/// Wire format event types (LE u32 in callback payload bytes 0..4).
abstract final class NativeEventTypes {
  static const displayListReady = 1;
  static const documentOpened = 2;
  static const documentSaved = 3;
  static const spellCheckResult = 4;
  static const error = 5;
}

/// Drains the native event channel, returning how many events were forwarded.
typedef NativeEventPump = EventPump;

/// Routes FFI worker events to per-[requestId] completers (R0.2 correlation).
class NativeEventRouter {
  NativeEventRouter._();

  static final NativeEventRouter instance = NativeEventRouter._();

  /// The engine only invokes the Dart callback while its event channel is being
  /// drained, and nothing drains it spontaneously — so without a pump an
  /// awaited request is never answered. Fast while a wait is outstanding; slow
  /// otherwise, because background reflow repaints arrive with no waiter.
  static const _pumpWhileWaiting = Duration(milliseconds: 2);
  static const _pumpWhenIdle = Duration(milliseconds: 16);

  final Map<int, Completer<int>> _pending = {};
  final Map<int, int> _earlyEvents = {};
  NativeEventPump? _pump;
  Timer? _pumpTimer;
  Duration? _pumpInterval;

  /// Optional hook for events that are not correlated to an awaited request
  /// (e.g. background `DisplayListReady` under BACKGROUND_REQUEST_ID).
  void Function(int eventType, int requestId)? onUnsolicitedEvent;

  /// Installed once the native library is loaded; mock-backed tests never
  /// attach one, so they run without timers.
  void attachPump(NativeEventPump pump) {
    _pump = pump;
    _tunePump();
  }

  void detachPump() {
    _pump = null;
    _pumpTimer?.cancel();
    _pumpTimer = null;
    _pumpInterval = null;
  }

  void _tunePump() {
    final pump = _pump;
    if (pump == null) return;
    final wanted = _pending.isEmpty ? _pumpWhenIdle : _pumpWhileWaiting;
    if (_pumpTimer != null && _pumpInterval == wanted) return;
    _pumpTimer?.cancel();
    _pumpInterval = wanted;
    _pumpTimer = Timer.periodic(wanted, (_) => pump());
  }

  /// Decode LE u32 event_type + LE u64 request_id from callback [data].
  /// Decodes the LE u32 event_type + LE u64 request_id wire payload.
  ///
  /// Only safe while the engine's buffer is alive, meaning a synchronous
  /// callback on the draining thread. Correlation deliberately does not depend
  /// on this: the engine passes both values by value, because reading a stale
  /// pointer from a deferred callback yields recycled stack bytes and request
  /// ids that match nothing.
  @visibleForTesting
  void handleWireEvent(int eventType, Uint8List data) {
    if (data.length < 12) return;
    onEvent(eventType, _readLeU64(data, 4));
  }

  void onEvent(int eventType, int requestId) {
    final completer = _pending.remove(requestId);
    if (completer != null && !completer.isCompleted) {
      completer.complete(eventType);
      _tunePump();
      return;
    }
    _earlyEvents[requestId] = eventType;
    onUnsolicitedEvent?.call(eventType, requestId);
  }

  /// Register interest in [requestId]; completes with event type when matched.
  Future<int> waitFor(int requestId, {Duration? timeout}) {
    // Anything the worker has already finished is sitting undelivered until
    // someone drains it, so drain before concluding the event has not arrived.
    _pump?.call();
    final early = _earlyEvents.remove(requestId);
    if (early != null) {
      return Future.value(early);
    }

    final existing = _pending[requestId];
    if (existing != null && !existing.isCompleted) {
      return _withTimeout(existing.future, timeout, requestId);
    }
    final completer = Completer<int>();
    _pending[requestId] = completer;
    _tunePump();
    return _withTimeout(completer.future, timeout, requestId);
  }

  Future<int> _withTimeout(Future<int> future, Duration? timeout, int requestId) {
    if (timeout == null) return future;
    return future.timeout(timeout, onTimeout: () {
      final completer = _pending.remove(requestId);
      if (completer != null && !completer.isCompleted) {
        completer.completeError(
          TimeoutException('event wait timed out', timeout),
        );
      }
      _tunePump();
      throw TimeoutException('event wait timed out', timeout);
    });
  }

  @visibleForTesting
  void reset() {
    for (final completer in _pending.values) {
      if (!completer.isCompleted) {
        completer.completeError(StateError('router reset'));
      }
    }
    _pending.clear();
    _earlyEvents.clear();
    // The pump is a property of the loaded library, not of correlation state:
    // callers reset between documents and still expect events to arrive.
    _tunePump();
  }

  @visibleForTesting
  bool get isPumping => _pumpTimer != null;

  @visibleForTesting
  Duration? get pumpInterval => _pumpInterval;

  @visibleForTesting
  int get pendingCount => _pending.length;

  /// Events that arrived with no registered waiter, keyed by request id.
  @visibleForTesting
  Map<int, int> get unmatchedEvents => Map.unmodifiable(_earlyEvents);
}

int _readLeU64(Uint8List bytes, int offset) {
  final low = bytes[offset] |
      (bytes[offset + 1] << 8) |
      (bytes[offset + 2] << 16) |
      (bytes[offset + 3] << 24);
  final high = bytes[offset + 4] |
      (bytes[offset + 5] << 8) |
      (bytes[offset + 6] << 16) |
      (bytes[offset + 7] << 24);
  return low + high * 0x100000000;
}
