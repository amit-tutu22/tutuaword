//! OpenAI Chat Completions adapter (F28.S1).

use crate::http::HttpClient;
use crate::provider::{
    AiCapabilities, AiError, AiProvider, CompletionRequest, CompletionResponse, CompletionStream,
    OPENAI,
};
use serde_json::{json, Value};
use std::sync::Arc;

pub struct OpenAiProvider {
    http: Arc<dyn HttpClient>,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(http: Arc<dyn HttpClient>, api_key: impl Into<String>) -> Self {
        Self {
            http,
            api_key: api_key.into(),
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
    }
}

impl AiProvider for OpenAiProvider {
    fn id(&self) -> &str {
        OPENAI
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    fn capabilities(&self) -> AiCapabilities {
        AiCapabilities {
            max_context_tokens: 128_000,
            supports_streaming: false,
            supports_function_calling: true,
            local: false,
        }
    }

    fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, AiError> {
        if self.api_key.trim().is_empty() {
            return Err(AiError::ProviderUnavailable(
                "OpenAI API key not configured".into(),
            ));
        }
        let mut messages = Vec::new();
        if let Some(system) = &request.system_prompt {
            messages.push(json!({"role": "system", "content": system}));
        }
        messages.push(json!({"role": "user", "content": request.prompt}));
        let body = json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
        })
        .to_string();
        let response = self.http.post_json(
            &self.chat_url(),
            &[
                ("Authorization", &format!("Bearer {}", self.api_key)),
                ("Content-Type", "application/json"),
            ],
            &body,
        )?;
        parse_openai_chat_response(&response)
    }

    fn stream(&self, _: &CompletionRequest) -> Result<CompletionStream, AiError> {
        Err(AiError::NotImplemented)
    }

    fn is_available(&self) -> bool {
        !self.api_key.trim().is_empty()
    }
}

pub(crate) fn parse_openai_chat_response(body: &str) -> Result<CompletionResponse, AiError> {
    let value: Value = serde_json::from_str(body)
        .map_err(|e| AiError::CompletionFailed(format!("OpenAI JSON: {e}")))?;
    let text = value
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AiError::CompletionFailed("OpenAI response missing content".into()))?
        .to_string();
    let tokens = value
        .pointer("/usage/total_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    Ok(CompletionResponse {
        text,
        tokens_used: tokens,
    })
}
