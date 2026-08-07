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
        let session = self.session().expect("WasmSession not initialized");
        let request_id = session
            .open_bytes(data)
            .ok_or(OpenError::EngineShutDown)?;
        match session.wait_for_response(request_id, Duration::from_secs(30)) {
            WaitOutcome::Matched(_) => Ok(()),
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
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EngineShutDown => f.write_str("engine shut down"),
            Self::TimedOut => f.write_str("open timed out"),
        }
    }
}

impl std::error::Error for OpenError {}

#[cfg(feature = "wasm-bindgen")]
mod bindgen_exports {
    use super::*;
    use tw_layout::WEIGHT_REGULAR;
    use wasm_bindgen::prelude::*;

  /// Browser/Node engine handle. Uses the inline executor and requires the host
  /// to register fonts before opening documents.
    #[wasm_bindgen]
    pub struct TwEngine {
        session: WasmSession,
    }

    #[wasm_bindgen]
    impl TwEngine {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                session: WasmSession::new(),
            }
        }

        /// Register a font face from raw bytes. `family` is matched case-insensitively.
        pub fn register_font(
            &mut self,
            family: &str,
            bold: bool,
            italic: bool,
            data: &[u8],
        ) -> Result<(), JsValue> {
            let mut spec = FontFaceSpec::new(family);
            if bold {
                spec = spec.bold();
            } else {
                spec = spec.weight(WEIGHT_REGULAR);
            }
            if italic {
                spec = spec.italic();
            }
            self.session
                .register_face(&spec, data.to_vec())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            Ok(())
        }

        /// Open a document from raw bytes (DOCX, ODT, plain text, …).
        pub fn open_document(&mut self, data: &[u8]) -> Result<(), JsValue> {
            self.session
                .open_bytes_and_wait(data.to_vec())
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        pub fn page_count(&self) -> u32 {
            self.session.page_count()
        }

        pub fn text(&self) -> String {
            self.session.document_text()
        }

        /// Sum of glyph advances on the first line of page 0 (layout width proxy).
        pub fn first_line_advance(&self) -> f32 {
            self.session.first_line_width(0)
        }

        /// Drive the inline engine and deliver any pending events.
        pub fn pump(&mut self) -> u32 {
            self.session.pump() as u32
        }
    }
}
