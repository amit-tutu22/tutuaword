//! wasmtime sandbox — no WASI filesystem/network (F26.S3).

use crate::host::SandboxHostState;
use crate::PluginError;
use wasmtime::{Config, Engine, Linker, Module, Store};

/// Restricted wasmtime engine for plugin guests.
pub struct WasmSandbox {
    engine: Engine,
}

impl WasmSandbox {
    pub fn new() -> Result<Self, PluginError> {
        let mut config = Config::new();
        // Keep the wasm feature set conservative for untrusted guests.
        config.wasm_simd(false);
        config.wasm_relaxed_simd(false);
        config.wasm_threads(false);
        config.consume_fuel(true);
        let engine = Engine::new(&config).map_err(|e| PluginError::Message(e.to_string()))?;
        Ok(Self { engine })
    }

    pub fn compile(&self, wasm_bytes: &[u8]) -> Result<Module, PluginError> {
        Module::new(&self.engine, wasm_bytes)
            .map_err(|e| PluginError::Message(format!("compile plugin: {e}")))
    }

    /// Instantiate a precompiled module, call exported `export`, return its i32.
    pub fn invoke_module(
        &self,
        module: &Module,
        state: SandboxHostState,
        export: &str,
        fuel: u64,
    ) -> Result<(i32, SandboxHostState), PluginError> {
        let mut linker = Linker::new(&self.engine);
        define_host_imports(&mut linker)?;

        let mut store = Store::new(&self.engine, state);
        store
            .set_fuel(fuel)
            .map_err(|e| PluginError::Message(e.to_string()))?;

        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|e| PluginError::Message(format!("instantiate plugin: {e}")))?;
        let func = instance
            .get_typed_func::<(), i32>(&mut store, export)
            .map_err(|e| PluginError::Message(format!("missing export `{export}`: {e}")))?;
        let result = func
            .call(&mut store, ())
            .map_err(|e| PluginError::Message(format!("plugin trap: {e}")))?;
        let state = store.into_data();
        Ok((result, state))
    }

    /// Instantiate `wasm_bytes` (binary or WAT), call exported `export`, return its i32.
    ///
    /// Takes ownership of `state` for the call and returns it afterward so the
    /// document session (including undo history) is preserved.
    pub fn invoke(
        &self,
        wasm_bytes: &[u8],
        state: SandboxHostState,
        export: &str,
        fuel: u64,
    ) -> Result<(i32, SandboxHostState), PluginError> {
        let module = self.compile(wasm_bytes)?;
        self.invoke_module(&module, state, export, fuel)
    }

    /// Cloneable engine handle for invoke outside a host mutex.
    pub fn engine(&self) -> Engine {
        self.engine.clone()
    }
}

/// Run a precompiled module with a cloned engine (no sandbox struct required).
pub fn invoke_with_engine(
    engine: &Engine,
    module: &Module,
    state: SandboxHostState,
    export: &str,
    fuel: u64,
) -> Result<(i32, SandboxHostState), PluginError> {
    let mut linker = Linker::new(engine);
    define_host_imports(&mut linker)?;

    let mut store = Store::new(engine, state);
    store
        .set_fuel(fuel)
        .map_err(|e| PluginError::Message(e.to_string()))?;

    let instance = linker
        .instantiate(&mut store, module)
        .map_err(|e| PluginError::Message(format!("instantiate plugin: {e}")))?;
    let func = instance
        .get_typed_func::<(), i32>(&mut store, export)
        .map_err(|e| PluginError::Message(format!("missing export `{export}`: {e}")))?;
    let result = func
        .call(&mut store, ())
        .map_err(|e| PluginError::Message(format!("plugin trap: {e}")))?;
    let state = store.into_data();
    Ok((result, state))
}

fn define_host_imports(linker: &mut Linker<SandboxHostState>) -> Result<(), PluginError> {
    linker
        .func_wrap(
            "tw",
            "require_cap",
            |mut caller: wasmtime::Caller<'_, SandboxHostState>, code: i32| -> i32 {
                caller.data_mut().require_code(code)
            },
        )
        .map_err(|e| PluginError::Message(e.to_string()))?;
    linker
        .func_wrap(
            "tw",
            "get_paragraph_count",
            |mut caller: wasmtime::Caller<'_, SandboxHostState>| -> i32 {
                match caller.data_mut().get_paragraph_count() {
                    Ok(n) => n as i32,
                    Err(_) => -1,
                }
            },
        )
        .map_err(|e| PluginError::Message(e.to_string()))?;
    linker
        .func_wrap(
            "tw",
            "insert_hello",
            |mut caller: wasmtime::Caller<'_, SandboxHostState>| -> i32 {
                match caller
                    .data_mut()
                    .insert_text_at_start("Hello from plugin")
                {
                    Ok(()) => 0,
                    Err(PluginError::CapabilityDenied(_)) => -1,
                    Err(_) => -2,
                }
            },
        )
        .map_err(|e| PluginError::Message(e.to_string()))?;
    Ok(())
}

impl Default for WasmSandbox {
    fn default() -> Self {
        Self::new().expect("wasmtime engine")
    }
}

/// Sample WAT: require DocumentEdit then insert hello text.
pub const SAMPLE_EDIT_PLUGIN_WAT: &str = r#"
(module
  (import "tw" "require_cap" (func $require_cap (param i32) (result i32)))
  (import "tw" "insert_hello" (func $insert_hello (result i32)))
  (func (export "run") (result i32)
    (if (result i32) (i32.ne (call $require_cap (i32.const 1)) (i32.const 0))
      (then (i32.const -1))
      (else (call $insert_hello))
    )
  )
)
"#;

/// Sample WAT: require DocumentRead then return paragraph count.
pub const SAMPLE_READ_PLUGIN_WAT: &str = r#"
(module
  (import "tw" "require_cap" (func $require_cap (param i32) (result i32)))
  (import "tw" "get_paragraph_count" (func $get_count (result i32)))
  (func (export "run") (result i32)
    (if (result i32) (i32.ne (call $require_cap (i32.const 0)) (i32.const 0))
      (then (i32.const -1))
      (else (call $get_count))
    )
  )
)
"#;
