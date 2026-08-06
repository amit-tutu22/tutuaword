import 'dart:async';
import 'dart:ffi';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';

/// Wire format event types (LE u32 in callback payload bytes 0..4).
abstract final class NativeEventTypes {
  static const displayListReady = 1;
  static const documentOpened = 2;
  static const documentSaved = 3;
  static const spellCheckResult = 4;
  static const error = 5;
}

/// Routes FFI worker events to per-[requestId] completers (R0.2 correlation).
class NativeEventRouter {
  NativeEventRouter._();

  static final NativeEventRouter instance = NativeEventRouter._();

  final Map<int, Completer<int>> _pending = {};

  /// Decode LE u32 event_type + LE u64 request_id from callback [data].
  void handleWireEvent(int eventType, Pointer<Uint8> data, int len) {
    if (len < 12) return;
    final bytes = data.asTypedList(len);
    final bd = ByteData.sublistView(Uint8List.fromList(bytes));
    final requestId = bd.getUint64(4, Endian.little);
    onEvent(eventType, requestId);
  }

  void onEvent(int eventType, int requestId) {
    final completer = _pending.remove(requestId);
    if (completer != null && !completer.isCompleted) {
      completer.complete(eventType);
    }
  }

  /// Register interest in [requestId]; completes with event type when matched.
  Future<int> waitFor(int requestId) {
    final existing = _pending[requestId];
    if (existing != null && !existing.isCompleted) {
      return existing.future;
    }
    final completer = Completer<int>();
    _pending[requestId] = completer;
    return completer.future;
  }

  @visibleForTesting
  void reset() {
    for (final completer in _pending.values) {
      if (!completer.isCompleted) {
        completer.completeError(StateError('router reset'));
      }
    }
    _pending.clear();
  }

  @visibleForTesting
  int get pendingCount => _pending.length;
}
