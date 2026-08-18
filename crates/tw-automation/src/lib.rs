//! Public document automation API (F26.S4 / Layer 6).
//!
//! Stable JSON request schema over the internal Command path. Not VBA/COM parity.

pub mod schema;

use schema::{AutomationRequest, AutomationResponse, AutomationResult};
use std::path::Path;
use tw_core::{
    document_plain_text, export_document, import_document_bundle, DetectedFormat, FormatContext,
    SyncSession,
};
use tw_edit::{command_from_json, command_to_json, Command, EditSession};
use tw_model::Document;
use tw_pdf::{prepare_print_pdf, PrintLayoutOptions};
use tw_policy::{PolicyCapability, PolicyEngine};

pub use schema::{AUTOMATION_SCHEMA_VERSION, AutomationEnvelope, AutomationError};

/// Headless automation session with policy gates.
pub struct AutomationSession {
    sync: SyncSession,
    format_ctx: FormatContext,
    policy: PolicyEngine,
}

impl AutomationSession {
    pub fn new() -> Self {
        Self {
            sync: SyncSession::new(),
            format_ctx: FormatContext::new_document(),
            policy: PolicyEngine::permissive(),
        }
    }

    pub fn with_policy(policy: PolicyEngine) -> Self {
        Self {
            sync: SyncSession::new(),
            format_ctx: FormatContext::new_document(),
            policy,
        }
    }

    pub fn policy(&self) -> &PolicyEngine {
        &self.policy
    }

    pub fn document(&self) -> &Document {
        &self.sync.edit.document
    }

    /// Execute a versioned automation request.
    pub fn execute(&mut self, envelope: AutomationEnvelope) -> AutomationResponse {
        if envelope.schema_version != AUTOMATION_SCHEMA_VERSION {
            return AutomationResponse::err(AutomationError::UnsupportedSchema {
                expected: AUTOMATION_SCHEMA_VERSION,
                got: envelope.schema_version,
            });
        }

        self.policy
            .require(PolicyCapability::AutomationDispatch)
            .map_err(|detail| AutomationError::PolicyDenied { detail })
            .and_then(|_| self.dispatch_inner(envelope.request))
            .into()
    }

    fn dispatch_inner(&mut self, request: AutomationRequest) -> Result<AutomationResult, AutomationError> {
        match request {
            AutomationRequest::NewDocument => {
                self.sync = SyncSession::new();
                self.format_ctx = FormatContext::new_document();
                Ok(AutomationResult::Ok {
                    message: "new_document".into(),
                })
            }
            AutomationRequest::OpenDocx { path } => {
                let bytes = std::fs::read(&path).map_err(|e| AutomationError::Io {
                    path: path.clone(),
                    detail: e.to_string(),
                })?;
                self.open_docx_bytes(&bytes, Some(path))?;
                Ok(AutomationResult::Ok {
                    message: "opened".into(),
                })
            }
            AutomationRequest::DispatchCommand { command_json } => {
                let command = command_from_json(command_json.as_bytes())
                    .map_err(|e| AutomationError::InvalidCommand {
                        detail: e.to_string(),
                    })?;
                self.sync.apply(command);
                self.format_ctx.mark_document_modified();
                Ok(AutomationResult::Ok {
                    message: "command_applied".into(),
                })
            }
            AutomationRequest::GetPlainText => Ok(AutomationResult::Text {
                text: document_plain_text(self.document()),
            }),
            AutomationRequest::ExportDocx { path } => {
                self.policy
                    .require(PolicyCapability::ExportDocx)
                    .map_err(|detail| AutomationError::PolicyDenied { detail })?;
                self.format_ctx.save_format = DetectedFormat::Docx;
                let bytes = export_document(self.document(), &self.format_ctx).map_err(|e| {
                    AutomationError::Export {
                        detail: e.to_string(),
                    }
                })?;
                std::fs::write(&path, &bytes).map_err(|e| AutomationError::Io {
                    path: path.clone(),
                    detail: e.to_string(),
                })?;
                Ok(AutomationResult::Ok {
                    message: format!("exported_docx:{path}"),
                })
            }
            AutomationRequest::ExportPdf { path } => {
                self.policy
                    .require(PolicyCapability::ExportPdf)
                    .map_err(|detail| AutomationError::PolicyDenied { detail })?;
                let bytes = prepare_print_pdf(self.document(), &PrintLayoutOptions::default())
                    .map_err(|e| AutomationError::Export {
                        detail: e.to_string(),
                    })?;
                std::fs::write(&path, &bytes).map_err(|e| AutomationError::Io {
                    path: path.clone(),
                    detail: e.to_string(),
                })?;
                Ok(AutomationResult::Ok {
                    message: format!("exported_pdf:{path}"),
                })
            }
        }
    }

    fn open_docx_bytes(&mut self, bytes: &[u8], path_hint: Option<String>) -> Result<(), AutomationError> {
        let bundle = import_document_bundle(bytes, path_hint.as_deref()).map_err(|e| {
            AutomationError::Import {
                detail: e.to_string(),
            }
        })?;
        if let Some(pkg) = &bundle.docx_package {
            if tw_docx::package_has_vba_parts(pkg) {
                self.policy
                    .require(PolicyCapability::OpenMacroDocument)
                    .map_err(|detail| AutomationError::PolicyDenied { detail })?;
            }
        }
        let tw_core::ImportBundle {
            document,
            source_format,
            docx_package,
            odt_package,
            embedded_fonts,
        } = bundle;
        self.format_ctx = FormatContext::from_bundle(
            tw_core::ImportBundle {
                document: document.clone(),
                source_format,
                docx_package,
                odt_package,
                embedded_fonts: embedded_fonts.clone(),
            },
            path_hint,
        );
        // Fonts the document embeds must be live before the first layout pass.
        for font in &embedded_fonts {
            let _ = self
                .sync
                .layout
                .register_face(&font.spec, font.data.clone());
        }
        self.sync.edit = EditSession::from_document(document);
        self.sync.relayout(None);
        Ok(())
    }
}

impl Default for AutomationSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a JSON automation envelope (CLI helper).
pub fn envelope_from_json_str(json: &str) -> Result<AutomationEnvelope, AutomationError> {
    serde_json::from_str(json).map_err(|e| AutomationError::InvalidCommand {
        detail: e.to_string(),
    })
}

/// Back-compat alias.
pub fn request_from_json_str(json: &str) -> Result<AutomationEnvelope, AutomationError> {
    envelope_from_json_str(json)
}

/// Serialize a Command for automation clients.
pub fn serialize_command(command: &Command) -> Result<String, AutomationError> {
    let bytes = command_to_json(command).map_err(|e| AutomationError::InvalidCommand {
        detail: e.to_string(),
    })?;
    String::from_utf8(bytes).map_err(|e| AutomationError::InvalidCommand {
        detail: e.to_string(),
    })
}

/// Open DOCX from path into a document (test helper).
pub fn open_docx_path(path: &Path) -> Result<Document, AutomationError> {
    let bytes = std::fs::read(path).map_err(|e| AutomationError::Io {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;
    import_document_bundle(&bytes, None)
        .map(|b| b.document)
        .map_err(|e| AutomationError::Import {
            detail: e.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u_f26_s4_new_document_and_plaintext() {
        let mut auto = AutomationSession::new();
        let resp = auto.execute(AutomationEnvelope {
            schema_version: AUTOMATION_SCHEMA_VERSION,
            request: AutomationRequest::NewDocument,
        });
        assert!(resp.success);
        let text = auto.execute(AutomationEnvelope {
            schema_version: AUTOMATION_SCHEMA_VERSION,
            request: AutomationRequest::GetPlainText,
        });
        assert!(text.success);
    }

    #[test]
    fn u_f26_s4_dispatch_insert_text() {
        let mut auto = AutomationSession::new();
        auto.execute(AutomationEnvelope {
            schema_version: AUTOMATION_SCHEMA_VERSION,
            request: AutomationRequest::NewDocument,
        });
        let run_id = auto.document().sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .runs[0]
            .id;
        let cmd = Command::InsertText {
            run_id,
            offset: 0,
            text: "Hello".into(),
        };
        let json = serialize_command(&cmd).unwrap();
        let resp = auto.execute(AutomationEnvelope {
            schema_version: AUTOMATION_SCHEMA_VERSION,
            request: AutomationRequest::DispatchCommand {
                command_json: json,
            },
        });
        assert!(resp.success);
        let text_resp = auto.execute(AutomationEnvelope {
            schema_version: AUTOMATION_SCHEMA_VERSION,
            request: AutomationRequest::GetPlainText,
        });
        if let AutomationResult::Text { text } = text_resp.result.unwrap() {
            assert!(text.contains("Hello"));
        } else {
            panic!("expected text result");
        }
    }

    #[test]
    fn u_f26_s4_policy_blocks_pdf_export() {
        let policy = PolicyEngine::new(tw_policy::PolicyConfig {
            allow_export_pdf: false,
            ..Default::default()
        });
        let mut auto = AutomationSession::with_policy(policy);
        auto.execute(AutomationEnvelope {
            schema_version: AUTOMATION_SCHEMA_VERSION,
            request: AutomationRequest::NewDocument,
        });
        let resp = auto.execute(AutomationEnvelope {
            schema_version: AUTOMATION_SCHEMA_VERSION,
            request: AutomationRequest::ExportPdf {
                path: "/tmp/should-not-write.pdf".into(),
            },
        });
        assert!(!resp.success);
    }
}
