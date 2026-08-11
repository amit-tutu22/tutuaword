//! Injectable HTTP transport for cloud / local-HTTP AI providers (F28.S1).

use crate::provider::AiError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Minimal POST client used by OpenAI / Gemini / Ollama adapters.
pub trait HttpClient: Send + Sync {
    fn post_json(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: &str,
    ) -> Result<String, AiError>;
}

/// In-memory HTTP stub for unit/integration tests (no network).
#[derive(Clone, Default)]
pub struct MockHttpClient {
    inner: Arc<Mutex<MockHttpInner>>,
}

#[derive(Default)]
struct MockHttpInner {
    /// Exact URL → response body.
    responses: HashMap<String, String>,
    /// Prefix match fallbacks (first match wins).
    prefix_responses: Vec<(String, String)>,
    calls: Vec<MockHttpCall>,
}

#[derive(Debug, Clone)]
pub struct MockHttpCall {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue(&self, url: impl Into<String>, response_body: impl Into<String>) {
        let mut inner = self.inner.lock().expect("mock http lock");
        inner.responses.insert(url.into(), response_body.into());
    }

    pub fn enqueue_prefix(&self, prefix: impl Into<String>, response_body: impl Into<String>) {
        let mut inner = self.inner.lock().expect("mock http lock");
        inner
            .prefix_responses
            .push((prefix.into(), response_body.into()));
    }

    pub fn calls(&self) -> Vec<MockHttpCall> {
        self.inner.lock().expect("mock http lock").calls.clone()
    }
}

impl HttpClient for MockHttpClient {
    fn post_json(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: &str,
    ) -> Result<String, AiError> {
        let mut inner = self.inner.lock().expect("mock http lock");
        inner.calls.push(MockHttpCall {
            url: url.to_string(),
            headers: headers
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
            body: body.to_string(),
        });
        if let Some(resp) = inner.responses.get(url).cloned() {
            return Ok(resp);
        }
        for (prefix, resp) in &inner.prefix_responses {
            if url.starts_with(prefix.as_str()) {
                return Ok(resp.clone());
            }
        }
        Err(AiError::CompletionFailed(format!(
            "mock HTTP: no fixture for {url}"
        )))
    }
}
