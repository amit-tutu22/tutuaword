//! Enterprise policy engine (Layer 8).
//!
//! Gates automation API calls, plugin capabilities, and AI provider access.

use serde::{Deserialize, Serialize};

/// Capability tokens enforced by tenant / admin policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCapability {
    ExportPdf,
    ExportDocx,
    RunPlugin,
    UseAiProvider,
    OpenMacroDocument,
    AutomationDispatch,
}

impl PolicyCapability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExportPdf => "export_pdf",
            Self::ExportDocx => "export_docx",
            Self::RunPlugin => "run_plugin",
            Self::UseAiProvider => "ai_provider",
            Self::OpenMacroDocument => "open_macro_document",
            Self::AutomationDispatch => "automation_dispatch",
        }
    }
}

/// Tenant policy configuration (defaults allow all for local/dev).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    #[serde(default = "default_true")]
    pub allow_export_pdf: bool,
    #[serde(default = "default_true")]
    pub allow_export_docx: bool,
    #[serde(default = "default_true")]
    pub allow_plugins: bool,
    #[serde(default = "default_true")]
    pub allow_ai: bool,
    #[serde(default = "default_true")]
    pub allow_macro_documents: bool,
    #[serde(default = "default_true")]
    pub allow_automation: bool,
    /// Plugin capability strings denied even when `allow_plugins` is true
    /// (e.g. `"network"`, `"document.edit"` — matches `tw-plugin` capability names).
    #[serde(default)]
    pub denied_plugin_capabilities: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            allow_export_pdf: true,
            allow_export_docx: true,
            allow_plugins: true,
            allow_ai: true,
            allow_macro_documents: true,
            allow_automation: true,
            denied_plugin_capabilities: Vec::new(),
        }
    }
}

/// Result of a policy check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny { reason: String },
}

/// Evaluates policy for automation, plugins, and AI surfaces.
#[derive(Debug, Clone)]
pub struct PolicyEngine {
    config: PolicyConfig,
}

impl PolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    pub fn permissive() -> Self {
        Self::new(PolicyConfig::default())
    }

    pub fn config(&self) -> &PolicyConfig {
        &self.config
    }

    pub fn check(&self, capability: PolicyCapability) -> PolicyDecision {
        let allowed = match capability {
            PolicyCapability::ExportPdf => self.config.allow_export_pdf,
            PolicyCapability::ExportDocx => self.config.allow_export_docx,
            PolicyCapability::RunPlugin => self.config.allow_plugins,
            PolicyCapability::UseAiProvider => self.config.allow_ai,
            PolicyCapability::OpenMacroDocument => self.config.allow_macro_documents,
            PolicyCapability::AutomationDispatch => self.config.allow_automation,
        };
        if allowed {
            PolicyDecision::Allow
        } else {
            PolicyDecision::Deny {
                reason: format!("policy denies {}", capability.as_str()),
            }
        }
    }

    /// Filter plugin capability strings through tenant deny list.
    pub fn filter_plugin_capabilities<'a, I>(&self, requested: I) -> Vec<String>
    where
        I: IntoIterator<Item = &'a str>,
    {
        if !self.config.allow_plugins {
            return Vec::new();
        }
        requested
            .into_iter()
            .filter(|cap| !self.config.denied_plugin_capabilities.contains(&cap.to_string()))
            .map(|cap| cap.to_string())
            .collect()
    }

    pub fn require(&self, capability: PolicyCapability) -> Result<(), String> {
        match self.check(capability) {
            PolicyDecision::Allow => Ok(()),
            PolicyDecision::Deny { reason } => Err(reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u_layer8_policy_denies_export_pdf_when_disabled() {
        let engine = PolicyEngine::new(PolicyConfig {
            allow_export_pdf: false,
            ..Default::default()
        });
        assert!(matches!(
            engine.check(PolicyCapability::ExportPdf),
            PolicyDecision::Deny { .. }
        ));
    }

    #[test]
    fn u_layer8_plugin_capability_filter() {
        let engine = PolicyEngine::new(PolicyConfig {
            denied_plugin_capabilities: vec!["network".into()],
            ..Default::default()
        });
        let caps = vec!["document.read", "network"];
        let filtered = engine.filter_plugin_capabilities(caps);
        assert_eq!(filtered, vec!["document.read".to_string()]);
    }
}
