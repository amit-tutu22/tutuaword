import 'dart:async';

Future<void> twWasmInit() async {}

Object createWasmEngine() {
  throw UnsupportedError('WASM is only available on web');
}

Object callMethod(Object target, String method, List<Object?> args) {
  throw UnsupportedError('WASM is only available on web');
}

Object? callMethodOrNull(Object target, String method, List<Object?> args) {
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
