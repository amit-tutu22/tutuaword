//! Google Gemini generateContent adapter (F28.S1).

use crate::http::HttpClient;
use crate::provider::{
    AiCapabilities, AiError, AiProvider, CompletionRequest, CompletionResponse, CompletionStream,
    GEMINI,
};
use serde_json::{json, Value};
use std::sync::Arc;

pub struct GeminiProvider {
    http: Arc<dyn HttpClient>,
    api_key: String,
    model: String,
    base_url: String,
}

impl GeminiProvider {
    pub fn new(http: Arc<dyn HttpClient>, api_key: impl Into<String>) -> Self {
        Self {
            http,
            api_key: api_key.into(),
            model: "gemini-2.0-flash".into(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".into(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    fn generate_url(&self) -> String {
        format!(
            "{}/models/{}:generateContent",
            self.base_url.trim_end_matches('/'),
            self.model,
        )
    }
}

impl AiProvider for GeminiProvider {
    fn id(&self) -> &str {
        GEMINI
    }

    fn name(&self) -> &str {
        "Google Gemini"
    }

    fn capabilities(&self) -> AiCapabilities {
        AiCapabilities {
            max_context_tokens: 1_000_000,
            supports_streaming: false,
            supports_function_calling: true,
            local: false,
        }
    }

    fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, AiError> {
        if self.api_key.trim().is_empty() {
            return Err(AiError::ProviderUnavailable(
                "Gemini API key not configured".into(),
            ));
        }
        let mut parts = Vec::new();
        if let Some(system) = &request.system_prompt {
            parts.push(json!({"text": system}));
        }
        parts.push(json!({"text": request.prompt}));
        let body = json!({
            "contents": [{ "parts": parts }],
            "generationConfig": {
                "maxOutputTokens": request.max_tokens,
                "temperature": request.temperature,
            }
        })
        .to_string();
        let response = self.http.post_json(
            &self.generate_url(),
            &[
                ("Content-Type", "application/json"),
                ("x-goog-api-key", self.api_key.as_str()),
            ],
            &body,
        )?;
        parse_gemini_response(&response)
    }

    fn stream(&self, _: &CompletionRequest) -> Result<CompletionStream, AiError> {
        Err(AiError::NotImplemented)
    }

    fn is_available(&self) -> bool {
        !self.api_key.trim().is_empty()
    }
}

pub(crate) fn parse_gemini_response(body: &str) -> Result<CompletionResponse, AiError> {
    let value: Value = serde_json::from_str(body)
        .map_err(|e| AiError::CompletionFailed(format!("Gemini JSON: {e}")))?;
    let text = value
        .pointer("/candidates/0/content/parts/0/text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AiError::CompletionFailed("Gemini response missing text".into()))?
        .to_string();
    let tokens = value
        .pointer("/usageMetadata/totalTokenCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    Ok(CompletionResponse {
        text,
        tokens_used: tokens,
    })
}
