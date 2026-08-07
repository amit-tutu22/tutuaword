/// No-op bootstrap on VM targets (Android, iOS, desktop).
class WasmEngineBootstrap {
  WasmEngineBootstrap._();

  static Future<void> initialize() async {}
}
