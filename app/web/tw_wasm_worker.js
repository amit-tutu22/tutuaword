import init, { TwEngine } from './wasm/tw_wasm.js';

let engine = null;
let pumpTimer = null;

function drainEvents() {
  while (true) {
    const json = engine.pop_event();
    if (!json) break;
    const event = JSON.parse(json);
    postMessage({ type: 'event', ...event });
  }
}

function pushCache() {
  if (!engine) return;
  const pages = {};
  const count = engine.page_count();
  for (let i = 0; i < count; i += 1) {
    const bytes = engine.page_display_list_bytes(i);
    if (bytes && bytes.length) {
      pages[i] = bytes;
    }
  }
  const atlasBytes = engine.atlas_bytes();
  const transfers = [];
  for (const bytes of Object.values(pages)) {
    transfers.push(bytes.buffer);
  }
  if (atlasBytes.length) {
    transfers.push(atlasBytes.buffer);
  }
  postMessage(
    {
      type: 'cache',
      pages,
      pageCount: count,
      displayVersion: engine.display_list_version(),
      atlas: {
        generation: engine.atlas_generation(),
        width: engine.atlas_width(),
        height: engine.atlas_height(),
        bytes: atlasBytes,
      },
    },
    transfers,
  );
}

function startPump() {
  if (pumpTimer != null) return;
  const tick = () => {
    if (!engine) return;
    engine.pump();
    drainEvents();
    pushCache();
    pumpTimer = setTimeout(tick, 8);
  };
  tick();
}

async function boot() {
  await init();
  engine = new TwEngine();
  startPump();
  pushCache();
  postMessage({ type: 'ready' });
}

function transferablesFromArgs(args) {
  const transfers = [];
  for (const arg of args) {
    if (arg instanceof ArrayBuffer) {
      transfers.push(arg);
    } else if (arg instanceof Uint8Array) {
      transfers.push(arg.buffer);
    }
  }
  return transfers;
}

function normalizeResult(result) {
  if (result instanceof Uint8Array) {
    return { value: result, transfer: [result.buffer] };
  }
  return { value: result, transfer: [] };
}

onmessage = async (e) => {
  const msg = e.data;
  if (msg.type !== 'invoke') return;
  const { id, method, args } = msg;
  try {
    if (!engine) {
      throw new Error('worker engine not ready');
    }
    const fn = engine[method];
    if (typeof fn !== 'function') {
      throw new Error(`unknown method: ${method}`);
    }
    const result = fn.apply(engine, args ?? []);
    engine.pump();
    drainEvents();
    pushCache();
    const { value, transfer } = normalizeResult(result);
    postMessage({ type: 'response', id, ok: true, result: value }, transfer);
  } catch (err) {
    postMessage({ type: 'response', id, ok: false, error: String(err) });
  }
};

boot();
