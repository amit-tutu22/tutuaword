let worker = null;
let readyPromise = null;
let nextId = 1;
const pending = new Map();
const eventListeners = [];

const cache = {
  pages: {},
  pageCount: 0,
  displayVersion: 0,
  atlas: {
    generation: 0,
    width: 0,
    height: 0,
    bytes: new Uint8Array(0),
  },
};

function ensureWorker() {
  if (!readyPromise) {
    readyPromise = new Promise((resolve, reject) => {
      worker = new Worker('tw_wasm_worker.js', { type: 'module' });
      worker.onmessage = (e) => {
        const msg = e.data;
        if (msg.type === 'ready') {
          resolve();
          return;
        }
        if (msg.type === 'cache') {
          cache.pages = msg.pages ?? {};
          cache.pageCount = msg.pageCount ?? 0;
          cache.displayVersion = msg.displayVersion ?? 0;
          cache.atlas = msg.atlas ?? cache.atlas;
          return;
        }
        if (msg.type === 'event') {
          for (const fn of eventListeners) {
            fn(msg);
          }
          return;
        }
        if (msg.type === 'response') {
          const waiter = pending.get(msg.id);
          if (!waiter) return;
          pending.delete(msg.id);
          if (msg.ok) waiter.resolve(msg.result);
          else waiter.reject(new Error(msg.error || 'worker invoke failed'));
        }
      };
      worker.onerror = (err) => reject(err);
    });
  }
  return readyPromise;
}

async function twWasmInit() {
  await ensureWorker();
}

function onWorkerEvent(listener) {
  eventListeners.push(listener);
}

function invoke(method, args = [], transfer = []) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    pending.set(id, { resolve, reject });
    worker.postMessage({ type: 'invoke', id, method, args }, transfer);
  });
}

function getCachedPageBytes(page) {
  return cache.pages[page] ?? null;
}

function getCacheState() {
  return cache;
}

function createEngine() {
  return {
    invoke,
    onEvent: onWorkerEvent,
    cache,
    getCachedPageBytes,
  };
}

globalThis.twWasmInit = twWasmInit;
globalThis.twWasm = {
  createEngine,
  invoke,
  onWorkerEvent,
  getCachedPageBytes,
  getCacheState,
};
