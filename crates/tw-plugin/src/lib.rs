use serde::{Deserialize, Serialize};
use thiserror::Error;
use tw_edit::{Command, DocRange};

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin error: {0}")]
    Message(String),
    #[error("capability denied: {0:?}")]
    CapabilityDenied(Capability),
    #[error("not implemented")]
    NotImplemented,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    DocumentRead,
    DocumentEdit,
    DocumentSuggest,
    UiSidebar,
    UiContextMenu,
    UiToolbar,
    UiDialog,
    EventsDocument,
    EventsSelection,
    Network,
    Storage,
    FilesystemRead,
    AiProvider,
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

pub trait Plugin: Send + Sync {
    fn on_activate(&self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn on_deactivate(&self) -> Result<(), PluginError>;
    fn on_command(
        &self,
        command: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, PluginError>;
}

pub trait PluginDocument {
    fn get_text(&self, range: Option<DocRange>) -> String;
    fn get_paragraph_count(&self) -> u32;
    fn get_paragraph_text(&self, index: u32) -> Option<String>;
    fn get_selection(&self) -> Option<DocRange>;
    fn apply_edit(&self, command: Command) -> Result<(), PluginError>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
