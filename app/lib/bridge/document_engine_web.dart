import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/wasm_document_engine.dart';
import 'package:tutuaword/bridge/wasm_engine_bootstrap_web.dart';

DocumentEngine? loadDocumentEngine() {
  final engine = WasmEngineBootstrap.cachedEngine;
  return engine == null ? null : WasmDocumentEngine(engine);
}
