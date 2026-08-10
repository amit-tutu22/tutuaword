use crate::capability::Capability;
use crate::{PluginContext, PluginError};
use tw_edit::{Command, EditSession};
use tw_model::NodeId;

/// Mutable host state shared with wasmtime guest imports (F26.S3).
pub struct SandboxHostState {
    pub context: PluginContext,
    pub session: EditSession,
    /// Last capability denial (for diagnostics).
    pub last_denial: Option<Capability>,
}

impl SandboxHostState {
    pub fn new(context: PluginContext, session: EditSession) -> Self {
        Self {
            context,
            session,
            last_denial: None,
        }
    }

    pub fn require(&mut self, cap: Capability) -> Result<(), PluginError> {
        if self.context.has_capability(cap) {
            Ok(())
        } else {
            self.last_denial = Some(cap);
            Err(PluginError::CapabilityDenied(cap))
        }
    }

    pub fn require_code(&mut self, code: i32) -> i32 {
        match Capability::from_code(code) {
            Some(cap) => match self.require(cap) {
                Ok(()) => 0,
                Err(_) => -1,
            },
            None => -2,
        }
    }

    pub fn get_paragraph_count(&mut self) -> Result<u32, PluginError> {
        self.require(Capability::DocumentRead)?;
        Ok(self
            .session
            .document
            .sections
            .iter()
            .flat_map(|s| s.blocks.iter())
            .filter(|b| b.paragraph().is_some())
            .count() as u32)
    }

    pub fn get_plain_text(&mut self) -> Result<String, PluginError> {
        self.require(Capability::DocumentRead)?;
        Ok(document_plain_text(&self.session.document))
    }

    /// Insert text at the start of the first paragraph run (undoable).
    pub fn insert_text_at_start(&mut self, text: &str) -> Result<(), PluginError> {
        self.require(Capability::DocumentEdit)?;
        let run_id = first_run_id(&self.session.document).ok_or_else(|| {
            PluginError::Message("document has no text run".into())
        })?;
        self.session
            .apply(Command::InsertText {
                run_id,
                offset: 0,
                text: text.to_string(),
            })
            .map_err(|e| PluginError::Message(e.to_string()))?;
        Ok(())
    }
}

fn first_run_id(doc: &tw_model::Document) -> Option<NodeId> {
    doc.paragraph_at(0, 0)
        .and_then(|p| p.runs.first().map(|r| r.id))
}

fn document_plain_text(doc: &tw_model::Document) -> String {
    let mut out = String::new();
    for section in &doc.sections {
        for block in &section.blocks {
            if let Some(para) = block.paragraph() {
                if !out.is_empty() {
                    out.push('\n');
                }
                for run in &para.runs {
                    out.push_str(run.text());
                }
            }
        }
    }
    out
}
