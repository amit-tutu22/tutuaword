import 'dart:async';
import 'dart:js_util' as js_util;
import 'dart:typed_data';

Future<void> twWasmInit() {
  return js_util.promiseToFuture<void>(
    js_util.callMethod(js_util.globalThis, 'twWasmInit', []),
  );
}

Object createWasmEngine() {
  final twWasm = js_util.getProperty<Object>(js_util.globalThis, 'twWasm');
  return js_util.callMethod<Object>(twWasm, 'createEngine', []);
}

Future<Object?> invokeWasm(String method, List<Object?> args) {
  final twWasm = js_util.getProperty<Object>(js_util.globalThis, 'twWasm');
  final promise = js_util.callMethod<Object>(
    twWasm,
    'invoke',
    [method, args],
  );
  return js_util.promiseToFuture<Object?>(promise);
}

Uint8List? cachedPageDisplayListBytes(int page) {
  final twWasm = js_util.getProperty<Object>(js_util.globalThis, 'twWasm');
  final bytes = js_util.callMethod<Object?>(
    twWasm,
    'getCachedPageBytes',
    [page],
  );
  if (bytes == null) return null;
  if (bytes is Uint8List) return bytes;
  return Uint8List.fromList(List<int>.from(js_util.dartify(bytes) as List));
}

Object? wasmCacheState() {
  final twWasm = js_util.getProperty<Object>(js_util.globalThis, 'twWasm');
  return js_util.callMethod<Object?>(twWasm, 'getCacheState', []);
}

void attachWasmEventListener(void Function(Object? event) listener) {
  final twWasm = js_util.getProperty<Object>(js_util.globalThis, 'twWasm');
  js_util.callMethod<void>(twWasm, 'onWorkerEvent', [
    js_util.allowInterop(listener),
  ]);
}

Object? getProperty(Object target, String name) {
  return js_util.getProperty<Object?>(target, name);
}

Object getGlobalThis() => js_util.globalThis;

Future<Object?> promiseToFuture(Object promise) {
  return js_util.promiseToFuture<Object?>(promise);
}
