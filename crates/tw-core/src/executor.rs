//! Execution strategies for the engine worker (R3.1).
//!
//! [`Session`](crate::Session) owns the protocol — commands in, correlated
//! events out — but not the thread the engine runs on. That choice lives behind
//! [`EngineExecutor`]:
//!
//! - [`ThreadedExecutor`] runs the engine on a dedicated OS thread and blocks on
//!   the command channel when idle. Native default; behaviour is unchanged from
//!   before the abstraction existed.
//! - [`InlineExecutor`] owns no thread. Commands queue on `submit` and run on the
//!   caller's thread when the host *drives* the session. Nothing spawns, blocks
//!   or sleeps, which is what makes `wasm32-unknown-unknown` viable.
//!
//! `ThreadedExecutor` is compiled out entirely on `wasm32`, so no reachable
//! `std::thread::spawn` exists in a web build.

mod inline;
#[cfg(not(target_arch = "wasm32"))]
mod threaded;

pub use inline::InlineExecutor;
#[cfg(not(target_arch = "wasm32"))]
pub use threaded::ThreadedExecutor;

use crate::worker::{BridgeEvent, QueuedCommand};

use tw_layout::{FontFaceSpec, FontId, FontRegistrationError};

/// How a [`Session`](crate::Session) gets its commands executed.
///
/// Implementations must be usable from any thread: `Session` is `Send + Sync`
/// and the FFI layer keeps one in a global.
pub trait EngineExecutor: Send + Sync {
    /// Queue a command for the engine.
    ///
    /// Returns `false` only when the engine has shut down, matching the previous
    /// "channel disconnected" contract. Implementations apply backpressure at
    /// [`COMMAND_QUEUE_CAPACITY`](crate::COMMAND_QUEUE_CAPACITY) rather than
    /// dropping commands.
    fn submit(&self, command: QueuedCommand) -> bool;

    /// Next event produced by the engine, if one is already available. Never blocks.
    fn try_next_event(&self) -> Option<BridgeEvent>;

    /// Run outstanding engine work on the calling thread and return the number of
    /// work units performed (commands executed, event flushes, background reflow
    /// chunks).
    ///
    /// Returns `0` when there is nothing left to do. A driving caller can treat
    /// `0` as "no further events will appear without new input" and stop waiting.
    /// [`ThreadedExecutor`] always returns `0`: its worker thread does the work.
    fn drive(&self) -> usize;

    /// True when the engine only makes progress while [`EngineExecutor::drive`]
    /// is called, so a caller must drive instead of sleeping.
    fn requires_drive(&self) -> bool;

    /// Stop the engine, joining the worker thread if there is one. Idempotent.
    fn shutdown(&self);

    /// Inject a font face into an inline engine's layout shaper.
    ///
    /// Threaded executors return `None`; only inline / wasm sessions support
    /// host-provided font bytes (R3.1).
    fn register_face(
        &self,
        spec: &FontFaceSpec,
        data: Vec<u8>,
    ) -> Option<Result<FontId, FontRegistrationError>> {
        let _ = (spec, data);
        None
    }
}
