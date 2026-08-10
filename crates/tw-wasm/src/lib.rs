//! WASM bindings for the inline (driven) document engine (R3.1).
//!
//! Native hosts should use `tw-ffi`; this crate is the `wasm32-unknown-unknown`
//! entry point. The host must register font bytes before opening a document and
//! call [`TwEngine::pump`] (or rely on correlated waits that drive internally).

use std::time::Duration;
use tw_core::{Session, WaitOutcome};
use tw_layout::FontFaceSpec;

/// WASM-facing wrapper around an inline [`Session`].
pub struct WasmSession {
    inner: Option<Session>,
}

impl WasmSession {
    pub fn new() -> Self {
        Self {
            inner: Some(Session::new()),
        }
    }

    pub fn session(&self) -> Option<&Session> {
        self.inner.as_ref()
    }

    pub fn register_face(
        &self,
        spec: &FontFaceSpec,
        data: Vec<u8>,
    ) -> Result<tw_layout::FontId, tw_layout::FontRegistrationError> {
        self.session()
            .expect("WasmSession not initialized")
            .register_face(spec, data)
    }

    pub fn open_bytes_and_wait(&self, data: Vec<u8>) -> Result<(), OpenError> {
        self.open_bytes_with_path_and_password_and_wait(data, None, None)
    }

    pub fn open_bytes_with_path_and_password_and_wait(
        &self,
        data: Vec<u8>,
        path_hint: Option<String>,
        password: Option<String>,
    ) -> Result<(), OpenError> {
        let session = self.session().expect("WasmSession not initialized");
        let request_id = session
            .open_bytes_with_path_and_password(data, path_hint, password)
            .ok_or(OpenError::EngineShutDown)?;
        match session.wait_for_response(request_id, Duration::from_secs(30)) {
            WaitOutcome::Matched(event) => match event {
                tw_core::BridgeEvent::DocumentOpened { .. } => Ok(()),
                tw_core::BridgeEvent::Error { message, .. } => Err(OpenError::Failed(message)),
                _ => Err(OpenError::Failed("unexpected open response".into())),
            },
            WaitOutcome::Timeout => Err(OpenError::TimedOut),
        }
    }

    pub fn page_count(&self) -> u32 {
        self.session()
            .map(|s| s.page_count())
            .unwrap_or(0)
    }

    pub fn document_text(&self) -> String {
        self.session()
            .map(|s| s.document_text())
            .unwrap_or_default()
    }

    pub fn first_line_width(&self, page: u32) -> f32 {
        self.session()
            .map(|s| s.first_line_width(page))
            .unwrap_or(0.0)
    }

    pub fn pump(&self) -> usize {
        self.session().map(|s| s.pump_events()).unwrap_or(0)
    }
}

impl Default for WasmSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WasmSession {
    fn drop(&mut self) {
        self.inner.take();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenError {
    EngineShutDown,
    TimedOut,
    Failed(String),
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EngineShutDown => f.write_str("engine shut down"),
            Self::TimedOut => f.write_str("open timed out"),
            Self::Failed(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for OpenError {}

mod web_exports;

#[cfg(feature = "wasm-bindgen")]
pub use web_exports::bindgen_exports::TwEngine;
