//! Local llama.cpp / Ollama-compatible HTTP adapter (F28.S1).

use crate::http::HttpClient;
use crate::provider::{
    AiCapabilities, AiError, AiProvider, CompletionRequest, CompletionResponse, CompletionStream,
    LLAMA_CPP,
};
use serde_json::{json, Value};
use std::sync::Arc;

/// Talks to an OpenAI-compatible local endpoint (Ollama / llama.cpp server).
pub struct LlamaCppProvider {
    http: Arc<dyn HttpClient>,
    endpoint: String,
    model: String,
    /// When false, provider reports unavailable (offline local daemon).
    available: bool,
}

impl LlamaCppProvider {
    pub fn new(http: Arc<dyn HttpClient>, endpoint: impl Into<String>) -> Self {
        Self {
            http,
            endpoint: endpoint.into(),
            model: "llama3.2".into(),
            available: true,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    fn chat_url(&self) -> String {
        format!(
            "{}/v1/chat/completions",
            self.endpoint.trim_end_matches('/')
        )
    }
}

impl AiProvider for LlamaCppProvider {
    fn id(&self) -> &str {
        LLAMA_CPP
    }

    fn name(&self) -> &str {
        "llama.cpp (local)"
    }

    fn capabilities(&self) -> AiCapabilities {
        AiCapabilities {
            max_context_tokens: 8_192,
            supports_streaming: false,
            supports_function_calling: false,
            local: true,
        }
    }

    fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, AiError> {
        if !self.available {
            return Err(AiError::ProviderUnavailable(
                "local llama endpoint unavailable".into(),
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
            &[("Content-Type", "application/json")],
            &body,
        )?;
        parse_llama_chat_response(&response)
    }

    fn stream(&self, _: &CompletionRequest) -> Result<CompletionStream, AiError> {
        Err(AiError::NotImplemented)
    }

    fn is_available(&self) -> bool {
        self.available
    }
}

pub(crate) fn parse_llama_chat_response(body: &str) -> Result<CompletionResponse, AiError> {
    // Prefer OpenAI-compatible shape; fall back to Ollama native.
    if let Ok(parsed) = crate::providers::openai::parse_openai_chat_response(body) {
        return Ok(parsed);
    }
    let value: Value = serde_json::from_str(body)
        .map_err(|e| AiError::CompletionFailed(format!("llama JSON: {e}")))?;
    let text = value
        .pointer("/message/content")
        .or_else(|| value.pointer("/response"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| AiError::CompletionFailed("llama response missing content".into()))?
        .to_string();
    Ok(CompletionResponse {
        text,
        tokens_used: 0,
    })
}
