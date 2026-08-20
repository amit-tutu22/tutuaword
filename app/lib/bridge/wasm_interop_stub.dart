import 'dart:async';
import 'dart:typed_data';

Future<void> twWasmInit() async {}

Object createWasmEngine() {
  throw UnsupportedError('WASM is only available on web');
}

Future<Object?> invokeWasm(String method, List<Object?> args) {
  throw UnsupportedError('WASM is only available on web');
}

Uint8List? cachedPageDisplayListBytes(int page) {
  throw UnsupportedError('WASM is only available on web');
}

Object? wasmCacheState() {
  throw UnsupportedError('WASM is only available on web');
}

void attachWasmEventListener(void Function(Object? event) listener) {
  throw UnsupportedError('WASM is only available on web');
}

Object? getProperty(Object target, String name) {
  throw UnsupportedError('WASM is only available on web');
}

Object getGlobalThis() {
  throw UnsupportedError('WASM is only available on web');
}

Future<Object?> promiseToFuture(Object promise) {
  throw UnsupportedError('WASM is only available on web');
}
