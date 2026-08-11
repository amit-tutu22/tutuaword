use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::executor::EngineExecutor;
use crate::snapshot::SnapshotBuffer;
use crate::worker::{
    BridgeCommand, BridgeEvent, Flow, QueuedCommand, WorkerCore, COMMAND_QUEUE_CAPACITY,
    EVENT_CHANNEL_CAPACITY, STARTUP_REQUEST_ID,
};
use tw_layout::{FontFaceSpec, FontId, FontRegistrationError};

type FontRegisterReply = crossbeam_channel::Sender<Result<FontId, FontRegistrationError>>;
type FontRegisterRequest = (FontFaceSpec, Vec<u8>, FontRegisterReply);

/// How long the worker sleeps between retries while correlation acks are still
/// queued behind a full event channel.
const EVENT_DRAIN_POLL: std::time::Duration = std::time::Duration::from_millis(2);

/// Runs the engine on a dedicated OS thread — the native default.
///
/// The thread blocks on the command channel when idle, so an idle session costs
/// nothing, and `submit` blocks when the bounded queue is full. This is the
/// pre-R3.1 behaviour verbatim; the type is compiled out on `wasm32`.
pub struct ThreadedExecutor {
    cmd_tx: Sender<QueuedCommand>,
    events: Receiver<BridgeEvent>,
    font_tx: Sender<FontRegisterRequest>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl ThreadedExecutor {
    pub fn spawn(snapshot: Arc<SnapshotBuffer>, layout_cache: crate::SharedLayoutCache) -> Self {
        let (cmd_tx, cmd_rx) = crossbeam_channel::bounded(COMMAND_QUEUE_CAPACITY);
        let (event_tx, events) = crossbeam_channel::bounded(EVENT_CHANNEL_CAPACITY);
        let (font_tx, font_rx) = crossbeam_channel::unbounded();

        let join = thread::spawn(move || {
            worker_loop(cmd_rx, event_tx, snapshot, layout_cache, font_rx);
        });

        Self {
            cmd_tx,
            events,
            font_tx,
            join: Mutex::new(Some(join)),
        }
    }
}

impl EngineExecutor for ThreadedExecutor {
    fn submit(&self, command: QueuedCommand) -> bool {
        // Block until the worker accepts the command (bounded queue backpressure).
        // Fails only if the worker channel is disconnected (shutdown).
        self.cmd_tx.send(command).is_ok()
    }

    fn try_next_event(&self) -> Option<BridgeEvent> {
        self.events.try_recv().ok()
    }

    fn drive(&self) -> usize {
        0
    }

    fn requires_drive(&self) -> bool {
        false
    }

    fn shutdown(&self) {
        let _ = self.cmd_tx.send(QueuedCommand {
            request_id: STARTUP_REQUEST_ID,
            inner: BridgeCommand::Shutdown,
        });
        if let Some(join) = self.join.lock().take() {
            let _ = join.join();
        }
    }

    fn register_face(
        &self,
        spec: &FontFaceSpec,
        data: Vec<u8>,
    ) -> Option<Result<FontId, FontRegistrationError>> {
        let (reply_tx, reply_rx) = crossbeam_channel::bounded(1);
        if self
            .font_tx
            .send((spec.clone(), data, reply_tx))
            .is_err()
        {
            return None;
        }
        reply_rx.recv().ok()
    }
}

fn worker_loop(
    cmd_rx: Receiver<QueuedCommand>,
    event_tx: Sender<BridgeEvent>,
    snapshot: Arc<SnapshotBuffer>,
    layout_cache: crate::SharedLayoutCache,
    font_rx: Receiver<FontRegisterRequest>,
) {
    let mut core = WorkerCore::new(event_tx, snapshot, layout_cache);

    loop {
        while let Ok((spec, data, reply)) = font_rx.try_recv() {
            let result = core.register_face(&spec, data);
            let _ = reply.send(result);
        }

        let queued = match core.take_pending() {
            Some(queued) => Some(queued),
            None => loop {
                // Anything the caller sent preempts background work, so the queue
                // is drained first and re-checked after every unit of idle work.
                match cmd_rx.try_recv() {
                    Ok(queued) => break Some(queued),
                    Err(TryRecvError::Disconnected) => break None,
                    Err(TryRecvError::Empty) => {}
                }
                while let Ok((spec, data, reply)) = font_rx.try_recv() {
                    let result = core.register_face(&spec, data);
                    let _ = reply.send(result);
                }
                if core.flush_events() {
                    continue;
                }
                if core.has_pending_reflow() {
                    core.run_background_chunk();
                    continue;
                }
                if core.has_backlog() {
                    // Correlation acks are still queued behind a full channel; wake
                    // periodically to retry instead of sleeping until the next command.
                    match cmd_rx.recv_timeout(EVENT_DRAIN_POLL) {
                        Ok(queued) => break Some(queued),
                        Err(RecvTimeoutError::Timeout) => continue,
                        Err(RecvTimeoutError::Disconnected) => break None,
                    }
                }
                // Idle wait must cover font registration too: a host that parks
                // here on `cmd_rx` alone never answers `register_face`, and the
                // caller blocks on its reply channel forever.
                crossbeam_channel::select! {
                    recv(cmd_rx) -> msg => match msg {
                        Ok(queued) => break Some(queued),
                        Err(_) => break None,
                    },
                    recv(font_rx) -> msg => match msg {
                        Ok((spec, data, reply)) => {
                            let result = core.register_face(&spec, data);
                            let _ = reply.send(result);
                            continue;
                        }
                        Err(_) => match cmd_rx.recv() {
                            Ok(queued) => break Some(queued),
                            Err(_) => break None,
                        },
                    },
                }
            },
        };
        let Some(queued) = queued else {
            break;
        };
        let mut next_command = || cmd_rx.try_recv().ok();
        if core.execute(queued, &mut next_command) == Flow::Shutdown {
            break;
        }
    }
}
