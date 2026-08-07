use crossbeam_channel::Receiver;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::executor::EngineExecutor;
use crate::snapshot::SnapshotBuffer;
use crate::worker::{
    BridgeEvent, Flow, QueuedCommand, WorkerCore, COMMAND_QUEUE_CAPACITY, EVENT_CHANNEL_CAPACITY,
};

/// Runs the engine on the caller's thread, with no worker thread at all.
///
/// This is the executor a `wasm32` build uses, and it is fully usable (and
/// tested) on native hosts. The contract is:
///
/// - [`submit`](EngineExecutor::submit) only enqueues. It never lays out, never
///   blocks and never drops a command.
/// - Work happens in [`drive`](EngineExecutor::drive), which the host calls —
///   directly, or indirectly through [`Session::pump_events`](crate::Session::pump_events),
///   [`Session::poll_event`](crate::Session::poll_event) or the correlated waits.
/// - One `drive` runs *every* queued command, then flushes the outbound event
///   backlog, then — only if no command is waiting — one chunk of background
///   forward relayout.
///
/// Backpressure keeps the shape of the threaded path: when the queue reaches
/// [`COMMAND_QUEUE_CAPACITY`], `submit` drives the engine instead of blocking, so
/// a producer that outruns the engine pays the layout cost inline rather than
/// growing an unbounded queue.
pub struct InlineExecutor {
    /// `None` once shut down; dropping the core drops the event sender, which is
    /// the same terminal state a joined worker thread leaves behind.
    core: Mutex<Option<WorkerCore>>,
    queue: Mutex<VecDeque<QueuedCommand>>,
    events: Receiver<BridgeEvent>,
    shutdown: AtomicBool,
}

impl InlineExecutor {
    /// Builds the engine and lays out the startup document **on the calling
    /// thread**, publishing `DocumentOpened { request_id: STARTUP_REQUEST_ID }`
    /// before returning. A threaded session does the same work on its worker
    /// thread; inline there is nowhere else to put it.
    pub fn new(snapshot: Arc<SnapshotBuffer>, layout_cache: crate::SharedLayoutCache) -> Self {
        let (event_tx, events) = crossbeam_channel::bounded(EVENT_CHANNEL_CAPACITY);
        let core = WorkerCore::new(event_tx, snapshot, layout_cache);
        Self {
            core: Mutex::new(Some(core)),
            queue: Mutex::new(VecDeque::new()),
            events,
            shutdown: AtomicBool::new(false),
        }
    }

    fn pop_command(&self) -> Option<QueuedCommand> {
        self.queue.lock().pop_front()
    }

    fn queue_is_empty(&self) -> bool {
        self.queue.lock().is_empty()
    }

    fn dispose(&self) {
        self.shutdown.store(true, Ordering::Release);
        self.queue.lock().clear();
    }
}

impl EngineExecutor for InlineExecutor {
    fn submit(&self, command: QueuedCommand) -> bool {
        if self.shutdown.load(Ordering::Acquire) {
            return false;
        }
        let at_capacity = {
            let mut queue = self.queue.lock();
            queue.push_back(command);
            queue.len() >= COMMAND_QUEUE_CAPACITY
        };
        if at_capacity {
            self.drive();
        }
        true
    }

    fn try_next_event(&self) -> Option<BridgeEvent> {
        self.events.try_recv().ok()
    }

    fn drive(&self) -> usize {
        if self.shutdown.load(Ordering::Acquire) {
            return 0;
        }
        // A re-entrant drive (a host calling back into the session from inside
        // engine work) returns immediately rather than deadlocking: the outer
        // drive is already making progress and will pick up the new command.
        let Some(mut guard) = self.core.try_lock() else {
            return 0;
        };
        let Some(core) = guard.as_mut() else {
            return 0;
        };

        let mut work = 0usize;
        let mut stopped = false;
        loop {
            let Some(queued) = core.take_pending().or_else(|| self.pop_command()) else {
                break;
            };
            work += 1;
            let mut next_command = || self.pop_command();
            if core.execute(queued, &mut next_command) == Flow::Shutdown {
                stopped = true;
                break;
            }
        }
        if stopped {
            *guard = None;
            drop(guard);
            self.dispose();
            return work;
        }

        // Correlation acks can still be queued behind a full event channel; the
        // host drains between drives, so retrying here makes room and progress.
        if core.has_backlog() && core.flush_events() {
            work += 1;
        }
        // Background forward relayout is idle-only work, exactly as on the
        // worker thread: it runs only once the command queue is empty, so a
        // keystroke arriving mid-catch-up is still served first. One chunk per
        // drive keeps a host that pumps on a frame timer responsive; successive
        // pumps walk the pending window forward.
        if self.queue_is_empty() && core.has_pending_reflow() && core.run_background_chunk() {
            work += 1;
        }
        work
    }

    fn requires_drive(&self) -> bool {
        true
    }

    fn shutdown(&self) {
        if self.shutdown.swap(true, Ordering::AcqRel) {
            return;
        }
        // Queued-but-unrun commands are discarded. A session only shuts down as
        // it is dropped, at which point no observer can see their events, and
        // running them would make `drop` do unbounded layout work.
        self.queue.lock().clear();
        let core = self.core.lock().take();
        drop(core);
    }

    fn register_face(
        &self,
        spec: &tw_layout::FontFaceSpec,
        data: Vec<u8>,
    ) -> Option<Result<tw_layout::FontId, tw_layout::FontRegistrationError>> {
        if self.shutdown.load(Ordering::Acquire) {
            return None;
        }
        let mut guard = self.core.lock();
        match guard.as_mut() {
            Some(core) => Some(core.register_face(spec, data)),
            None => None,
        }
    }
}
