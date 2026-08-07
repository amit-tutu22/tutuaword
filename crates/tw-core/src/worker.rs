use crate::bundle::{export_document, import_document_bundle, FormatContext};
use crate::import::DetectedFormat;
use crate::snapshot::{
    document_plain_text, document_properties_json, snapshot_from_pages, SinglePageSnapshot,
    SnapshotBuffer,
};
use crossbeam_channel::{Receiver, Sender, TrySendError};
use std::collections::VecDeque;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tw_edit::{paste, Command, EditSession};
use tw_layout::LayoutEngine;
use tw_model::NodeId;
use tw_pdf::PdfExporter;
use tw_render::DisplayListBuilder;

/// Reserved for the worker's initial document-ready event (not tied to a caller command).
pub const STARTUP_REQUEST_ID: u64 = 0;

/// Stamped on repaints produced by background forward relayout. Never issued to a
/// caller, so it can never satisfy (or steal) a pending edit correlation.
pub const BACKGROUND_REQUEST_ID: u64 = u64::MAX;

/// How long the worker sleeps between retries while correlation acks are still
/// queued behind a full event channel.
const EVENT_DRAIN_POLL: std::time::Duration = std::time::Duration::from_millis(2);

/// Bounded command queue depth (session blocks on `send` when full).
pub const COMMAND_QUEUE_CAPACITY: usize = 512;

/// Bounded worker → session event channel; overflow uses drop-oldest backpressure.
pub const EVENT_CHANNEL_CAPACITY: usize = 256;

/// Outbound event queue with per-page `DisplayListReady` coalescing.
///
/// Coalescing collapses redundant repaints, but the correlation id of every
/// superseded request is retained in `acks` (8 bytes each) instead of being
/// dropped with the event, so a caller awaiting an edit always gets a
/// completion even when the consumer falls far behind.
struct EventPublisher {
    tx: Sender<BridgeEvent>,
    buffer: VecDeque<BridgeEvent>,
    acks: VecDeque<u64>,
    /// Page/version stamped onto ack-only completions (latest published layout).
    ack_page: u32,
    ack_version: u64,
    disconnected: bool,
}

impl EventPublisher {
    fn new(tx: Sender<BridgeEvent>) -> Self {
        Self {
            tx,
            buffer: VecDeque::new(),
            acks: VecDeque::new(),
            ack_page: 0,
            ack_version: 0,
            disconnected: false,
        }
    }

    fn send(&mut self, event: BridgeEvent) {
        if let BridgeEvent::DisplayListReady { page, version, .. } = &event {
            self.ack_page = *page;
            self.ack_version = *version;
            self.coalesce_display_ready(*page);
        }
        self.buffer.push_back(event);
        self.flush();
    }

    /// True while events or correlation acks are still waiting for channel space.
    fn has_backlog(&self) -> bool {
        !self.disconnected && (!self.buffer.is_empty() || !self.acks.is_empty())
    }

    /// Move superseded same-page layout events out of the buffer, keeping their
    /// request ids so callers can still correlate the completed edit.
    fn coalesce_display_ready(&mut self, page: u32) {
        let acks = &mut self.acks;
        self.buffer.retain(|event| {
            if let BridgeEvent::DisplayListReady {
                request_id,
                page: p,
                ..
            } = event
            {
                if *p == page {
                    acks.push_back(*request_id);
                    return false;
                }
            }
            true
        });
    }

    /// Push as much of the backlog as the channel will accept. Returns whether any
    /// progress was made, so the worker's idle loop cannot spin without draining.
    fn flush(&mut self) -> bool {
        let mut progressed = false;
        while let Some(event) = self.buffer.front() {
            match self.tx.try_send(event.clone()) {
                Ok(()) => {
                    self.buffer.pop_front();
                    progressed = true;
                }
                Err(TrySendError::Full(_)) => return progressed,
                Err(TrySendError::Disconnected(_)) => {
                    self.drop_backlog();
                    return progressed;
                }
            }
        }
        while let Some(&request_id) = self.acks.front() {
            let ack = BridgeEvent::DisplayListReady {
                request_id,
                page: self.ack_page,
                version: self.ack_version,
            };
            match self.tx.try_send(ack) {
                Ok(()) => {
                    self.acks.pop_front();
                    progressed = true;
                }
                Err(TrySendError::Full(_)) => return progressed,
                Err(TrySendError::Disconnected(_)) => {
                    self.drop_backlog();
                    return progressed;
                }
            }
        }
        progressed
    }

    fn drop_backlog(&mut self) {
        self.buffer.clear();
        self.acks.clear();
        self.disconnected = true;
    }
}

#[derive(Debug, Clone)]
pub enum BridgeCommand {
    NewDocument,
    OpenDocument {
        data: Vec<u8>,
        path_hint: Option<String>,
    },
    SaveDocument,
    SaveDocumentAs { format: DetectedFormat },
    SpellCheckDocument,
    ToggleTrackChanges { enabled: bool },
    ApplyEdit { command: Command },
    SetCurrentPage { page: u32 },
    Undo,
    Redo,
    ExportPdf,
    PasteHtml {
        run_id: NodeId,
        offset: usize,
        html: Vec<u8>,
    },
    PasteDocx {
        run_id: NodeId,
        offset: usize,
        bytes: Vec<u8>,
    },
    Shutdown,
}

/// Command envelope with correlation id for FFI/event routing.
#[derive(Debug, Clone)]
pub struct QueuedCommand {
    pub request_id: u64,
    pub inner: BridgeCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeEvent {
    DisplayListReady {
        request_id: u64,
        page: u32,
        version: u64,
    },
    DocumentOpened {
        request_id: u64,
        page_count: u32,
    },
    DocumentSaved {
        request_id: u64,
        data: Vec<u8>,
    },
    SpellCheckResult {
        request_id: u64,
        misspellings: Vec<String>,
    },
    Error {
        request_id: u64,
        message: String,
    },
}

impl BridgeEvent {
    pub fn request_id(&self) -> u64 {
        match self {
            BridgeEvent::DisplayListReady { request_id, .. } => *request_id,
            BridgeEvent::DocumentOpened { request_id, .. } => *request_id,
            BridgeEvent::DocumentSaved { request_id, .. } => *request_id,
            BridgeEvent::SpellCheckResult { request_id, .. } => *request_id,
            BridgeEvent::Error { request_id, .. } => *request_id,
        }
    }
}

pub struct WorkerHandle {
    pub cmd_tx: Sender<QueuedCommand>,
    pub event_rx: Receiver<BridgeEvent>,
    join: Option<JoinHandle<()>>,
}

impl WorkerHandle {
    pub fn spawn(snapshot: Arc<SnapshotBuffer>, layout_cache: crate::SharedLayoutCache) -> Self {
        let (cmd_tx, cmd_rx) = crossbeam_channel::bounded(COMMAND_QUEUE_CAPACITY);
        let (event_tx, event_rx) = crossbeam_channel::bounded(EVENT_CHANNEL_CAPACITY);

        let join = thread::spawn(move || {
            worker_loop(cmd_rx, event_tx, snapshot, layout_cache);
        });

        Self {
            cmd_tx,
            event_rx,
            join: Some(join),
        }
    }

    pub fn shutdown(&mut self) {
        let _ = self.cmd_tx.send(QueuedCommand {
            request_id: STARTUP_REQUEST_ID,
            inner: BridgeCommand::Shutdown,
        });
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

/// What the next layout pass should do.
enum Relayout<'a> {
    Full,
    Nodes(&'a [NodeId]),
    /// Continue the reflow a page-capped incremental pass left unfinished.
    Forward,
}

fn worker_loop(
    cmd_rx: Receiver<QueuedCommand>,
    event_tx: Sender<BridgeEvent>,
    snapshot: Arc<SnapshotBuffer>,
    layout_cache: crate::SharedLayoutCache,
) {
    let mut events = EventPublisher::new(event_tx);
    let mut session = EditSession::new();
    let mut format_ctx = FormatContext::new_document();
    let mut layout = LayoutEngine::new();
    let mut version: u64 = 0;
    let mut current_page: u32 = 0;
    let mut cached_pages: Vec<Arc<SinglePageSnapshot>> = Vec::new();

    struct RebuildOutcome {
        version: u64,
        rebuilt_pages: Vec<u32>,
        ready_page: u32,
    }

    let rebuild = |session: &EditSession,
                   layout: &mut LayoutEngine,
                   version: &mut u64,
                   current_page: u32,
                   relayout: Relayout<'_>,
                   cached_pages: &mut Vec<Arc<SinglePageSnapshot>>|
     -> Option<RebuildOutcome> {
        let doc_layout = match relayout {
            Relayout::Nodes(nodes) if !nodes.is_empty() => {
                layout.invalidate_nodes(&session.document, nodes);
                layout.layout_document(&session.document)
            }
            Relayout::Forward => layout.continue_layout(&session.document)?,
            _ => {
                layout.invalidate_all();
                layout.layout_document(&session.document)
            }
        };
        *version += 1;
        let text = document_plain_text(&session.document);
        let read_only = session.document.settings.read_only;
        let relayout_start = layout.relayout_start_page() as usize;
        let is_incremental = layout.last_pass_incremental();

        let mut rebuilt_pages = Vec::new();
        let new_len = doc_layout.pages.len();

        // An incremental pass leaves page indices stable, so only the pages the
        // engine reported dirty (plus any pages the document grew by) can differ.
        if is_incremental && !cached_pages.is_empty() {
            cached_pages.truncate(new_len);
            let carried_over = cached_pages.len();
            let mut targets: Vec<usize> = layout
                .dirty_pages()
                .iter()
                .map(|&page| page as usize)
                .filter(|&idx| idx < new_len)
                .collect();
            targets.extend(carried_over..new_len);
            targets.sort_unstable();
            targets.dedup();

            for idx in targets {
                let page = &doc_layout.pages[idx];
                let list = DisplayListBuilder::from_page_without_atlas(page, *version);
                let bytes = DisplayListBuilder::to_page_bytes(&list);
                if idx < carried_over
                    && DisplayListBuilder::page_bytes_match_content(
                        &cached_pages[idx].bytes,
                        &bytes,
                    )
                {
                    // Reflowed to an identical result: keep the shared Arc so
                    // downstream readers see no churn on untouched pages.
                    continue;
                }
                let snapshot = Arc::new(SinglePageSnapshot {
                    version: *version,
                    bytes: Arc::new(bytes),
                    page_width: page.width,
                    page_height: page.height,
                });
                if idx < cached_pages.len() {
                    cached_pages[idx] = snapshot;
                } else {
                    cached_pages.push(snapshot);
                }
                rebuilt_pages.push(idx as u32);
            }
        } else {
            cached_pages.clear();
            cached_pages.reserve(new_len);
            for (idx, page) in doc_layout.pages.iter().enumerate() {
                let list = DisplayListBuilder::from_page_without_atlas(page, *version);
                cached_pages.push(Arc::new(SinglePageSnapshot {
                    version: *version,
                    bytes: Arc::new(DisplayListBuilder::to_page_bytes(&list)),
                    page_width: page.width,
                    page_height: page.height,
                }));
                rebuilt_pages.push(idx as u32);
            }
        }

        let atlas = layout.atlas();
        let atlas_generation = atlas.generation;
        let atlas_width = atlas.width;
        let atlas_height = atlas.height;
        let atlas_bytes = Arc::new(DisplayListBuilder::atlas_to_bytes(atlas, atlas_generation));

        let page_count = cached_pages.len().max(1) as u32;
        let props_json = document_properties_json(&session.document, page_count);
        let page_index = current_page.min(page_count.saturating_sub(1));

        if is_incremental && !cached_pages.is_empty() {
            let updated: Vec<Arc<SinglePageSnapshot>> =
                rebuilt_pages.iter().map(|&i| cached_pages[i as usize].clone()).collect();
            snapshot.publish_incremental(
                *version,
                page_index,
                page_count,
                &rebuilt_pages,
                &updated,
                atlas_generation,
                atlas_width,
                atlas_height,
                atlas_bytes,
                text,
                props_json,
                read_only,
            );
        } else {
            let pages: Vec<SinglePageSnapshot> = cached_pages
                .iter()
                .map(|p| (**p).clone())
                .collect();
            snapshot.publish(snapshot_from_pages(
                pages,
                page_index,
                *version,
                atlas_generation,
                atlas_width,
                atlas_height,
                atlas_bytes,
                text,
                props_json,
                read_only,
            ));
        }

        // A forward pass only moves layout, never the document, so the cache can
        // keep the Arc it already holds instead of paying for a deep clone per chunk.
        let doc_arc = match relayout {
            Relayout::Forward => layout_cache.read().document_snapshot(),
            _ => Arc::new(session.document.clone()),
        };
        {
            let mut cache = layout_cache.write();
            if is_incremental {
                cache.update_from_session_incremental(
                    layout,
                    Arc::clone(&doc_arc),
                    *version,
                    page_count,
                    layout.has_pending_reflow(),
                );
            } else {
                cache.update_from_session(layout, doc_arc);
            }
        }
        // Prefer the visible page when it was part of this pass so the caller's
        // completion event names the page it is actually waiting to repaint.
        let ready_page = if rebuilt_pages.contains(&current_page) {
            current_page
        } else {
            rebuilt_pages
                .first()
                .copied()
                .unwrap_or_else(|| relayout_start.min(page_count.saturating_sub(1) as usize) as u32)
        };
        Some(RebuildOutcome {
            version: *version,
            rebuilt_pages,
            ready_page,
        })
    };

    version = rebuild(
        &session,
        &mut layout,
        &mut version,
        current_page,
        Relayout::Full,
        &mut cached_pages,
    )
    .expect("full rebuild always produces a layout")
    .version;
    let page_count = layout.page_count() as u32;
    events.send(BridgeEvent::DocumentOpened {
        request_id: STARTUP_REQUEST_ID,
        page_count,
    });

    let mut pending: Option<QueuedCommand> = None;

    'commands: loop {
        let queued = match pending.take() {
            Some(queued) => Some(queued),
            None => loop {
                // Anything the caller sent preempts background work, so the queue
                // is drained first and re-checked after every unit of idle work.
                match cmd_rx.try_recv() {
                    Ok(queued) => break Some(queued),
                    Err(crossbeam_channel::TryRecvError::Disconnected) => break None,
                    Err(crossbeam_channel::TryRecvError::Empty) => {}
                }
                if events.flush() {
                    continue;
                }
                if layout.has_pending_reflow() {
                    if let Some(outcome) = rebuild(
                        &session,
                        &mut layout,
                        &mut version,
                        current_page,
                        Relayout::Forward,
                        &mut cached_pages,
                    ) {
                        version = outcome.version;
                        if !outcome.rebuilt_pages.is_empty() {
                            events.send(BridgeEvent::DisplayListReady {
                                request_id: BACKGROUND_REQUEST_ID,
                                page: outcome.ready_page,
                                version,
                            });
                        }
                    }
                    continue;
                }
                if events.has_backlog() {
                    // Correlation acks are still queued behind a full channel; wake
                    // periodically to retry instead of sleeping until the next command.
                    match cmd_rx.recv_timeout(EVENT_DRAIN_POLL) {
                        Ok(queued) => break Some(queued),
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break None,
                    }
                }
                match cmd_rx.recv() {
                    Ok(queued) => break Some(queued),
                    Err(_) => break None,
                }
            },
        };
        let Some(QueuedCommand {
            request_id: req_id,
            inner: cmd,
        }) = queued
        else {
            break 'commands;
        };

        match cmd {
            BridgeCommand::NewDocument => {
                session = EditSession::new();
                format_ctx = FormatContext::new_document();
                current_page = 0;
                version = rebuild(
                    &session,
                    &mut layout,
                    &mut version,
                    current_page,
                    Relayout::Full,
                    &mut cached_pages,
                )
                .expect("full rebuild always produces a layout")
                .version;
                let page_count = layout.page_count() as u32;
                events.send(BridgeEvent::DocumentOpened { request_id: req_id, page_count });
            }
            BridgeCommand::OpenDocument { data, path_hint } => {
                match import_document_bundle(&data, path_hint.as_deref()) {
                    Ok(bundle) => {
                        format_ctx = FormatContext::from_bundle(bundle.clone(), path_hint);
                        session = EditSession::from_document(bundle.document);
                        current_page = 0;
                        version = rebuild(
                    &session,
                    &mut layout,
                    &mut version,
                    current_page,
                    Relayout::Full,
                    &mut cached_pages,
                )
                .expect("full rebuild always produces a layout")
                .version;
                        let page_count = layout.page_count() as u32;
                        events.send(BridgeEvent::DocumentOpened { request_id: req_id, page_count });
                    }
                    Err(e) => {
                        events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: e.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::SaveDocument => match export_document(&session.document, &format_ctx) {
                Ok(data) => {
                    events.send(BridgeEvent::DocumentSaved { request_id: req_id, data });
                }
                Err(e) => {
                    events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: e.to_string(),
                    });
                }
            },
            BridgeCommand::SaveDocumentAs { format } => {
                format_ctx.save_format = format;
                match export_document(&session.document, &format_ctx) {
                    Ok(data) => {
                        events.send(BridgeEvent::DocumentSaved { request_id: req_id, data });
                    }
                    Err(e) => {
                        events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: e.to_string(),
                        });
                    }
                }
            },
            BridgeCommand::SpellCheckDocument => {
                let checker = tw_spell::SpellChecker::english();
                let text = document_plain_text(&session.document);
                let issues = checker.check_text(&text);
                let words: Vec<String> = issues.into_iter().map(|i| i.word).collect();
                events.send(BridgeEvent::SpellCheckResult {
                    request_id: req_id,
                    misspellings: words,
                });
            },
            BridgeCommand::ToggleTrackChanges { enabled } => {
                session.document.settings.track_changes_enabled = enabled;
            }
            BridgeCommand::PasteHtml {
                run_id,
                offset,
                html,
            } => match tw_html::import(&html) {
                Ok(doc) => {
                    if paste::paste_fragment_at(&mut session, run_id, offset, &doc).is_ok() {
                        format_ctx.mark_document_modified();
                        version = rebuild(
                    &session,
                    &mut layout,
                    &mut version,
                    current_page,
                    Relayout::Full,
                    &mut cached_pages,
                )
                .expect("full rebuild always produces a layout")
                .version;
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: req_id,
                            page: current_page,
                            version,
                        });
                    } else {
                        events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: "paste HTML failed".into(),
                        });
                    }
                }
                Err(err) => {
                    events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: err.to_string(),
                    });
                }
            },
            BridgeCommand::PasteDocx {
                run_id,
                offset,
                bytes,
            } => match tw_docx::import(&bytes) {
                Ok(result) => {
                    if paste::paste_fragment_at(&mut session, run_id, offset, &result.document).is_ok()
                    {
                        format_ctx.mark_document_modified();
                        version = rebuild(
                    &session,
                    &mut layout,
                    &mut version,
                    current_page,
                    Relayout::Full,
                    &mut cached_pages,
                )
                .expect("full rebuild always produces a layout")
                .version;
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: req_id,
                            page: current_page,
                            version,
                        });
                    } else {
                        events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: "paste DOCX fragment failed".into(),
                        });
                    }
                }
                Err(err) => {
                    events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: err.to_string(),
                    });
                }
            },
            BridgeCommand::ApplyEdit { command } => {
                let mut commands = vec![command];
                let mut request_ids = vec![req_id];
                while let Ok(next) = cmd_rx.try_recv() {
                    match next {
                        QueuedCommand {
                            request_id: batch_id,
                            inner: BridgeCommand::ApplyEdit { command: c },
                        } => {
                            commands.push(c);
                            request_ids.push(batch_id);
                        }
                        other => {
                            pending = Some(other);
                            break;
                        }
                    }
                }

                let mut apply_error = None;
                let mut affected_nodes: Vec<NodeId> = Vec::new();
                let mut tx = session.begin_transaction(None);
                for command in commands {
                    match tx.apply(command) {
                        Ok(result) => {
                            affected_nodes.extend(result.affected_nodes);
                        }
                        Err(e) => {
                            apply_error = Some(e);
                            break;
                        }
                    }
                }

                if let Some(e) = apply_error {
                    if let Err(abort_err) = tx.abort() {
                        events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: abort_err.to_string(),
                        });
                        continue;
                    }
                    format_ctx.mark_document_modified();
                    let outcome = rebuild(
                        &session,
                        &mut layout,
                        &mut version,
                        current_page,
                        if affected_nodes.is_empty() {
                            Relayout::Full
                        } else {
                            Relayout::Nodes(affected_nodes.as_slice())
                        },
                        &mut cached_pages,
                    )
                    .expect("edit rebuild always produces a layout");
                    version = outcome.version;
                    let ready_page = outcome.ready_page;
                    for batch_id in &request_ids {
                        events.send(BridgeEvent::Error {
                            request_id: *batch_id,
                            message: e.to_string(),
                        });
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: *batch_id,
                            page: ready_page,
                            version,
                        });
                    }
                } else {
                    tx.commit();
                    format_ctx.mark_document_modified();
                    let outcome = rebuild(
                        &session,
                        &mut layout,
                        &mut version,
                        current_page,
                        if affected_nodes.is_empty() {
                            Relayout::Full
                        } else {
                            Relayout::Nodes(affected_nodes.as_slice())
                        },
                        &mut cached_pages,
                    )
                    .expect("edit rebuild always produces a layout");
                    version = outcome.version;
                    let ready_page = outcome.ready_page;
                    for batch_id in request_ids {
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: batch_id,
                            page: ready_page,
                            version,
                        });
                    }
                }
            }
            BridgeCommand::ExportPdf => {
                let exporter = tw_pdf::DisplayListPdfExporter;
                match exporter.export(&session.document, &tw_pdf::PdfExportOptions::default()) {
                    Ok(data) => {
                        events.send(BridgeEvent::DocumentSaved { request_id: req_id, data });
                    }
                    Err(e) => {
                        events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: e.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::SetCurrentPage { page } => {
                current_page = page;
                snapshot.set_current_page(page);
                let snap = snapshot.read();
                events.send(BridgeEvent::DisplayListReady {
                    request_id: req_id,
                    page: snap.page_index,
                    version: snap.version,
                });
            }
            BridgeCommand::Undo => match session.undo() {
                Ok(Some(_)) => {
                    version = rebuild(
                    &session,
                    &mut layout,
                    &mut version,
                    current_page,
                    Relayout::Full,
                    &mut cached_pages,
                )
                .expect("full rebuild always produces a layout")
                .version;
                    events.send(BridgeEvent::DisplayListReady {
                        request_id: req_id,
                        page: current_page,
                        version,
                    });
                }
                Ok(None) => {
                    events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: "nothing to undo".into(),
                    });
                }
                Err(e) => {
                    events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: e.to_string(),
                    });
                }
            },
            BridgeCommand::Redo => match session.redo() {
                Ok(Some(_)) => {
                    version = rebuild(
                    &session,
                    &mut layout,
                    &mut version,
                    current_page,
                    Relayout::Full,
                    &mut cached_pages,
                )
                .expect("full rebuild always produces a layout")
                .version;
                    events.send(BridgeEvent::DisplayListReady {
                        request_id: req_id,
                        page: current_page,
                        version,
                    });
                }
                Ok(None) => {
                    events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: "nothing to redo".into(),
                    });
                }
                Err(e) => {
                    events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: e.to_string(),
                    });
                }
            },
            BridgeCommand::Shutdown => break,
        }
    }
}
