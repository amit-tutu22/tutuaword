//! Plugin lifecycle: install / enable / disable / invoke (F26.S3).

use crate::capability::{grant_capabilities, Capability};
use crate::host::SandboxHostState;
use crate::sandbox::{invoke_with_engine, WasmSandbox, SAMPLE_EDIT_PLUGIN_WAT};
use crate::{PluginContext, PluginError, PluginManifest};
use serde::Serialize;
use std::collections::HashMap;
use tw_edit::EditSession;
use wasmtime::{Engine, Module};

const DEFAULT_FUEL: u64 = 1_000_000;

#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub capabilities: Vec<Capability>,
    pub granted: Vec<Capability>,
}

struct InstalledPlugin {
    manifest: PluginManifest,
    module: Module,
    granted: Vec<Capability>,
    enabled: bool,
}

/// Precompiled invoke payload that can run after releasing the host mutex.
pub struct PreparedInvoke {
    module: Module,
    engine: Engine,
    context: PluginContext,
    plugin_id: String,
}

impl PreparedInvoke {
    pub fn run(self, session: &mut EditSession) -> Result<i32, PluginError> {
        let taken = std::mem::replace(session, EditSession::new());
        let state = SandboxHostState::new(self.context, taken);
        let (code, state) =
            invoke_with_engine(&self.engine, &self.module, state, "run", DEFAULT_FUEL)?;
        *session = state.session;
        if code < 0 {
            if let Some(cap) = state.last_denial {
                return Err(PluginError::CapabilityDenied(cap));
            }
            return Err(PluginError::Message(format!(
                "plugin `{}` returned error code {code}",
                self.plugin_id
            )));
        }
        Ok(code)
    }
}

/// Host-side plugin manager with wasmtime sandbox (F26.S3).
pub struct PluginManager {
    plugins: HashMap<String, InstalledPlugin>,
    sandbox: WasmSandbox,
}

impl PluginManager {
    pub fn new() -> Result<Self, PluginError> {
        Ok(Self {
            plugins: HashMap::new(),
            sandbox: WasmSandbox::new()?,
        })
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        let mut infos: Vec<_> = self
            .plugins
            .values()
            .map(|p| PluginInfo {
                id: p.manifest.id.clone(),
                name: p.manifest.name.clone(),
                version: p.manifest.version.clone(),
                enabled: p.enabled,
                capabilities: p.manifest.capabilities.clone(),
                granted: p.granted.clone(),
            })
            .collect();
        infos.sort_by(|a, b| a.id.cmp(&b.id));
        infos
    }

    pub fn list_json(&self) -> Result<String, PluginError> {
        serde_json::to_string(&self.list()).map_err(|e| PluginError::Message(e.to_string()))
    }

    /// Install from manifest + wasm/WAT bytes. `user_granted` is the capability set
    /// approved by the user (intersected with the manifest request).
    pub fn install(
        &mut self,
        manifest: PluginManifest,
        wasm_bytes: impl Into<Vec<u8>>,
        user_granted: &[Capability],
    ) -> Result<(), PluginError> {
        if manifest.id.trim().is_empty() {
            return Err(PluginError::Message("plugin id required".into()));
        }
        let granted = grant_capabilities(&manifest.capabilities, user_granted);
        let id = manifest.id.clone();
        let module = self.sandbox.compile(&wasm_bytes.into())?;
        self.plugins.insert(
            id,
            InstalledPlugin {
                manifest,
                module,
                granted,
                enabled: true,
            },
        );
        Ok(())
    }

    /// Convenience: install the built-in sample edit plugin WAT.
    pub fn install_sample_edit_plugin(
        &mut self,
        grant_edit: bool,
    ) -> Result<(), PluginError> {
        let caps = vec![Capability::DocumentRead, Capability::DocumentEdit];
        let granted: Vec<Capability> = if grant_edit {
            caps.clone()
        } else {
            vec![Capability::DocumentRead]
        };
        self.install(
            PluginManifest {
                id: "com.tutuaword.sample.edit".into(),
                name: "Sample Edit Plugin".into(),
                version: "1.0.0".into(),
                capabilities: caps,
            },
            SAMPLE_EDIT_PLUGIN_WAT.as_bytes().to_vec(),
            &granted,
        )
    }

    pub fn enable(&mut self, id: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::Message(format!("unknown plugin: {id}")))?;
        plugin.enabled = true;
        Ok(())
    }

    pub fn disable(&mut self, id: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::Message(format!("unknown plugin: {id}")))?;
        plugin.enabled = false;
        Ok(())
    }

    pub fn uninstall(&mut self, id: &str) -> Result<(), PluginError> {
        self.plugins
            .remove(id)
            .ok_or_else(|| PluginError::Message(format!("unknown plugin: {id}")))?;
        Ok(())
    }

    /// Snapshot module + engine so the caller can drop the host lock before WASM runs.
    pub fn prepare_invoke(&self, id: &str) -> Result<PreparedInvoke, PluginError> {
        let plugin = self
            .plugins
            .get(id)
            .ok_or_else(|| PluginError::Message(format!("unknown plugin: {id}")))?;
        if !plugin.enabled {
            return Err(PluginError::Message(format!("plugin disabled: {id}")));
        }
        Ok(PreparedInvoke {
            module: plugin.module.clone(),
            engine: self.sandbox.engine(),
            context: PluginContext {
                manifest: plugin.manifest.clone(),
                granted: plugin.granted.clone(),
            },
            plugin_id: id.to_string(),
        })
    }

    /// Run the plugin's `run` export against `session` (mutated in place).
    pub fn invoke(
        &self,
        id: &str,
        session: &mut EditSession,
    ) -> Result<i32, PluginError> {
        self.prepare_invoke(id)?.run(session)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new().expect("plugin manager")
    }
}
