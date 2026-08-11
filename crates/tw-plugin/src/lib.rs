//! Plugin host — wasmtime sandbox + capability gates (F26.S3).

mod capability;
mod host;
mod manager;
mod sandbox;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tw_edit::{Command, DocRange};

pub use capability::{grant_capabilities, Capability};
pub use host::SandboxHostState;
pub use manager::{PluginInfo, PluginManager};
pub use sandbox::{WasmSandbox, SAMPLE_EDIT_PLUGIN_WAT, SAMPLE_READ_PLUGIN_WAT};

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin error: {0}")]
    Message(String),
    #[error("capability denied: {0:?}")]
    CapabilityDenied(Capability),
    #[error("not implemented")]
    NotImplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub capabilities: Vec<Capability>,
}

pub struct PluginContext {
    pub manifest: PluginManifest,
    pub granted: Vec<Capability>,
}

impl PluginContext {
    pub fn has_capability(&self, cap: Capability) -> bool {
        self.granted.contains(&cap)
    }
}

/// In-process plugin trait (dev / native plugins).
pub trait Plugin: Send + Sync {
    fn on_activate(&self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn on_deactivate(&self) -> Result<(), PluginError>;
    fn on_command(
        &self,
        command: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, PluginError>;
}

/// Capability-gated document surface for plugins.
pub trait PluginDocument {
    fn get_text(&self, range: Option<DocRange>) -> Result<String, PluginError>;
    fn get_paragraph_count(&self) -> Result<u32, PluginError>;
    fn get_paragraph_text(&self, index: u32) -> Result<Option<String>, PluginError>;
    fn get_selection(&self) -> Result<Option<DocRange>, PluginError>;
    fn apply_edit(&self, command: Command) -> Result<(), PluginError>;
}
