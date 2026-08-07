// Loads tw-wasm (ES module) and exposes a small global API for Dart.
let initPromise = null;

async function twWasmInit() {
  if (!initPromise) {
    initPromise = import('./wasm/tw_wasm.js').then(async (mod) => {
      await mod.default();
      globalThis.twWasm = {
        createEngine: () => new mod.TwEngine(),
      };
    });
  }
  return initPromise;
}

globalThis.twWasmInit = twWasmInit;
