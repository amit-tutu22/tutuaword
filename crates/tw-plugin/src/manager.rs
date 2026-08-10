//! Plugin lifecycle: install / enable / disable / invoke (F26.S3).

use crate::capability::{grant_capabilities, Capability};
use crate::host::SandboxHostState;
use crate::sandbox::{WasmSandbox, SAMPLE_EDIT_PLUGIN_WAT};
use crate::{PluginContext, PluginError, PluginManifest};
use serde::Serialize;
use std::collections::HashMap;
use tw_edit::EditSession;

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
    wasm_bytes: Vec<u8>,
    granted: Vec<Capability>,
    enabled: bool,
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
        self.plugins.insert(
            id,
            InstalledPlugin {
                manifest,
                wasm_bytes: wasm_bytes.into(),
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

    /// Run the plugin's `run` export against `session` (mutated in place).
    pub fn invoke(
        &self,
        id: &str,
        session: &mut EditSession,
    ) -> Result<i32, PluginError> {
        let plugin = self
            .plugins
            .get(id)
            .ok_or_else(|| PluginError::Message(format!("unknown plugin: {id}")))?;
        if !plugin.enabled {
            return Err(PluginError::Message(format!("plugin disabled: {id}")));
        }
        let context = PluginContext {
            manifest: plugin.manifest.clone(),
            granted: plugin.granted.clone(),
        };
        // Move session into host state for the sandbox call.
        let taken = std::mem::replace(session, EditSession::new());
        let state = SandboxHostState::new(context, taken);
        let (code, state) =
            self.sandbox
                .invoke(&plugin.wasm_bytes, state, "run", DEFAULT_FUEL)?;
        *session = state.session;
        if code < 0 {
            if let Some(cap) = state.last_denial {
                return Err(PluginError::CapabilityDenied(cap));
            }
            return Err(PluginError::Message(format!(
                "plugin `{id}` returned error code {code}"
            )));
        }
        Ok(code)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new().expect("plugin manager")
    }
}
