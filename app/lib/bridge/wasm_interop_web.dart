import 'dart:async';
import 'dart:js_util' as js_util;

Future<void> twWasmInit() {
  return js_util.promiseToFuture<void>(
    js_util.callMethod(js_util.globalThis, 'twWasmInit', []),
  );
}

Object createWasmEngine() {
  final twWasm = js_util.getProperty<Object>(js_util.globalThis, 'twWasm');
  return js_util.callMethod<Object>(twWasm, 'createEngine', []);
}

Object callMethod(Object target, String method, List<Object?> args) {
  return js_util.callMethod<Object>(target, method, args);
}

Object? getProperty(Object target, String name) {
  return js_util.getProperty<Object?>(target, name);
}

Object getGlobalThis() => js_util.globalThis;

Future<Object?> promiseToFuture(Object promise) {
  return js_util.promiseToFuture<Object?>(promise);
}
