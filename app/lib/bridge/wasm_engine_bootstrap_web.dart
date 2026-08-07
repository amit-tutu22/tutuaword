import 'package:tutuaword/bridge/wasm_engine_web.dart';

/// Initializes the browser WASM module.
class WasmEngineBootstrap {
  WasmEngineBootstrap._();

  static WasmEngine? cachedEngine;

  static Future<void> initialize() async {
    cachedEngine = await WasmEngine.load();
  }
}
