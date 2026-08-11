//! Versioned automation request/response schema (F26.S4).

use serde::{Deserialize, Serialize};

pub const AUTOMATION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationEnvelope {
    pub schema_version: u32,
    #[serde(flatten)]
    pub request: AutomationRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutomationRequest {
    NewDocument,
    OpenDocx { path: String },
    DispatchCommand { command_json: String },
    GetPlainText,
    ExportDocx { path: String },
    ExportPdf { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AutomationResult {
    Ok { message: String },
    Text { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationResponse {
    pub schema_version: u32,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AutomationResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AutomationError>,
}

impl AutomationResponse {
    pub fn ok(result: AutomationResult) -> Self {
        Self {
            schema_version: AUTOMATION_SCHEMA_VERSION,
            success: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(error: AutomationError) -> Self {
        Self {
            schema_version: AUTOMATION_SCHEMA_VERSION,
            success: false,
            result: None,
            error: Some(error),
        }
    }
}

impl From<Result<AutomationResult, AutomationError>> for AutomationResponse {
    fn from(value: Result<AutomationResult, AutomationError>) -> Self {
        match value {
            Ok(result) => Self::ok(result),
            Err(error) => Self::err(error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum AutomationError {
    UnsupportedSchema { expected: u32, got: u32 },
    PolicyDenied { detail: String },
    Io { path: String, detail: String },
    Import { detail: String },
    Export { detail: String },
    InvalidCommand { detail: String },
    Command { detail: String },
}

impl From<String> for AutomationError {
    fn from(detail: String) -> Self {
        Self::PolicyDenied { detail }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u_f26_s4_envelope_roundtrip_json() {
        let env = AutomationEnvelope {
            schema_version: AUTOMATION_SCHEMA_VERSION,
            request: AutomationRequest::GetPlainText,
        };
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"schema_version\":1"));
        let decoded: AutomationEnvelope = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded.request, AutomationRequest::GetPlainText));
    }
}
