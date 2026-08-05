use thiserror::Error;
use tw_core::Session;
use tw_edit::Command;

#[derive(Debug, Error)]
pub enum WasmError {
    #[error("session not initialized")]
    NotInitialized,
}

/// WASM-facing wrapper around `tw_core::Session`.
pub struct WasmSession {
    inner: Option<Session>,
}

impl WasmSession {
    pub fn new() -> Self {
        Self {
            inner: Some(Session::new()),
        }
    }

    pub fn session(&self) -> Result<&Session, WasmError> {
        self.inner.as_ref().ok_or(WasmError::NotInitialized)
    }

    pub fn apply(&self, command: Command) -> Result<(), WasmError> {
        self.session()?.apply(command);
        Ok(())
    }

    pub fn new_document(&self) -> Result<(), WasmError> {
        self.session()?.new_document();
        Ok(())
    }

    pub fn open_bytes(&self, data: Vec<u8>) -> Result<(), WasmError> {
        self.session()?.open_bytes(data);
        Ok(())
    }

    pub fn save(&self) -> Result<(), WasmError> {
        self.session()?.save();
        Ok(())
    }

    pub fn page_count(&self) -> Result<u32, WasmError> {
        Ok(self.session()?.page_count())
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
