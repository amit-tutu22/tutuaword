use crate::bundle::{
    export_document, import_document_bundle_with_password, FormatContext,
};
use crate::import::DetectedFormat;
use crate::snapshot::{
    document_plain_text, document_properties_json, snapshot_from_pages, SinglePageSnapshot,
    SnapshotBuffer,
};
use crossbeam_channel::{Sender, TrySendError};
use std::collections::VecDeque;
use std::sync::Arc;
use tw_edit::{paste, Command, EditSession};
use tw_layout::LayoutEngine;
use tw_model::NodeId;
use tw_pdf::PdfExporter;
use tw_policy::{PolicyCapability, PolicyEngine};
use tw_render::DisplayListBuilder;

/// Reserved for the worker's initial document-ready event (not tied to a caller command).
pub const STARTUP_REQUEST_ID: u64 = 0;

/// Stamped on repaints produced by background forward relayout. Never issued to a
/// caller, so it can never satisfy (or steal) a pending edit correlation.
pub const BACKGROUND_REQUEST_ID: u64 = u64::MAX;

/// Bounded command queue depth (the session applies backpressure when full).
pub const COMMAND_QUEUE_CAPACITY: usize = 512;

/// Bounded worker → session event channel; overflow uses drop-oldest backpressure.
pub const EVENT_CHANNEL_CAPACITY: usize = 256;

/// Bounded outbound event backlog before drop-oldest backpressure applies.
pub const EVENT_BUFFER_CAPACITY: usize = 64;

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
        // Drop oldest non-coalesced events (especially large DocumentSaved payloads)
        // so a slow consumer cannot grow worker memory without bound.
        while self.buffer.len() >= EVENT_BUFFER_CAPACITY {
            if let Some(idx) = self
                .buffer
                .iter()
                .position(|e| !matches!(e, BridgeEvent::DisplayListReady { .. }))
            {
                self.buffer.remove(idx);
            } else {
                self.buffer.pop_front();
            }
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
        password: Option<String>,
    },
    SaveDocument,
    SaveDocumentAs { format: DetectedFormat },
    SpellCheckDocument,
    GrammarCheckDocument,
    SetReadOnly { enabled: bool },
    /// Set or clear the password used to encrypt DOCX on save (F22.S2).
    SetEncryptionPassword { password: Option<String> },
    ToggleTrackChanges { enabled: bool },
    ApplyEdit { command: Command },
    SetCurrentPage { page: u32 },
    Undo,
    Redo,
    ExportPdf,
    /// Font-embedded / VisualMatch PDF for the OS print dialog (F25.S1–S3).
    ExportPdfForPrint {
        layout: tw_pdf::PrintLayoutOptions,
        /// When set, export only this body range (F25.S3 print selection).
        selection: Option<tw_edit::DocRange>,
    },
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
    /// Export the given body range as a standalone DOCX fragment (L3 clipboard).
    ExportSelectionDocx {
        range: tw_edit::DocRange,
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
        /// Parallel suggestions per misspelling (F17.S1 UX).
        spell_issues: Vec<(String, usize, usize, Vec<String>)>,
    },
    GrammarCheckResult {
        request_id: u64,
        issues: Vec<String>,
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
            BridgeEvent::GrammarCheckResult { request_id, .. } => *request_id,
            BridgeEvent::Error { request_id, .. } => *request_id,
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

struct RebuildOutcome {
    version: u64,
    rebuilt_pages: Vec<u32>,
    ready_page: u32,
}

/// Whether the driver should keep feeding commands to the core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Flow {
    Continue,
    Shutdown,
}

/// The engine itself: document, layout, snapshots and outbound events.
///
/// The core owns no thread and no scheduling policy. It exposes one step of work
/// at a time ([`WorkerCore::execute`], [`WorkerCore::run_background_chunk`],
/// [`WorkerCore::flush_events`]) so that an executor can run it on a dedicated OS
/// thread (`ThreadedExecutor`) or on the caller's thread (`InlineExecutor`).
pub(crate) struct WorkerCore {
    events: EventPublisher,
    session: EditSession,
    format_ctx: FormatContext,
    layout: LayoutEngine,
    snapshot: Arc<SnapshotBuffer>,
    layout_cache: crate::SharedLayoutCache,
    version: u64,
    current_page: u32,
    cached_pages: Vec<Arc<SinglePageSnapshot>>,
    cached_atlas_generation: u64,
    cached_atlas_bytes: Arc<Vec<u8>>,
    /// Tenant policy (defaults permissive for local/dev).
    policy: PolicyEngine,
    /// A non-edit command pulled off the queue while batching consecutive edits;
    /// the driver must hand it back before reading the queue again.
    pending: Option<QueuedCommand>,
}

impl WorkerCore {
    /// Builds the engine, lays out the empty document and publishes the startup
    /// `DocumentOpened` event under [`STARTUP_REQUEST_ID`].
    pub(crate) fn new(
        event_tx: Sender<BridgeEvent>,
        snapshot: Arc<SnapshotBuffer>,
        layout_cache: crate::SharedLayoutCache,
    ) -> Self {
        let mut core = Self {
            events: EventPublisher::new(event_tx),
            session: EditSession::new(),
            format_ctx: FormatContext::new_document(),
            #[cfg(target_arch = "wasm32")]
            layout: LayoutEngine::with_injected_fonts(),
            #[cfg(all(not(target_arch = "wasm32"), any(target_os = "android", target_os = "ios")))]
            layout: LayoutEngine::with_injected_fonts(),
            #[cfg(all(
                not(target_arch = "wasm32"),
                not(any(target_os = "android", target_os = "ios"))
            ))]
            layout: LayoutEngine::new(),
            snapshot,
            layout_cache,
            version: 0,
            current_page: 0,
            cached_pages: Vec::new(),
            cached_atlas_generation: 0,
            cached_atlas_bytes: Arc::new(Vec::new()),
            policy: PolicyEngine::permissive(),
            pending: None,
        };
        core.rebuild(Relayout::Full)
            .expect("full rebuild always produces a layout");
        let page_count = core.layout.page_count() as u32;
        core.events.send(BridgeEvent::DocumentOpened {
            request_id: STARTUP_REQUEST_ID,
            page_count,
        });
        core
    }

    /// Command deferred by edit batching, if any.
    pub(crate) fn take_pending(&mut self) -> Option<QueuedCommand> {
        self.pending.take()
    }

    /// Push queued events into the channel. Returns whether progress was made.
    pub(crate) fn flush_events(&mut self) -> bool {
        self.events.flush()
    }

    /// True while events or correlation acks are still waiting for channel space.
    pub(crate) fn has_backlog(&self) -> bool {
        self.events.has_backlog()
    }

    /// True while a page-capped incremental pass still owes a forward reflow.
    pub(crate) fn has_pending_reflow(&self) -> bool {
        self.layout.has_pending_reflow()
    }

    /// Inject a host-provided font face into this engine's shaper (inline / wasm).
    pub(crate) fn register_face(
        &mut self,
        spec: &tw_layout::FontFaceSpec,
        data: Vec<u8>,
    ) -> Result<tw_layout::FontId, tw_layout::FontRegistrationError> {
        self.layout.register_face(spec, data)
    }

    /// Run one chunk of background forward relayout and publish its repaint under
    /// [`BACKGROUND_REQUEST_ID`]. Returns whether the chunk made progress; a
    /// caller must treat `false` as "the engine is done" rather than retrying.
    pub(crate) fn run_background_chunk(&mut self) -> bool {
        let Some(outcome) = self.rebuild(Relayout::Forward) else {
            return false;
        };
        if !outcome.rebuilt_pages.is_empty() {
            self.events.send(BridgeEvent::DisplayListReady {
                request_id: BACKGROUND_REQUEST_ID,
                page: outcome.ready_page,
                version: outcome.version,
            });
        }
        true
    }

    fn rebuild(&mut self, relayout: Relayout<'_>) -> Option<RebuildOutcome> {
        let doc_layout = match relayout {
            Relayout::Nodes(nodes) if !nodes.is_empty() => {
                self.layout.invalidate_nodes(&self.session.document, nodes);
                self.layout.layout_document(&self.session.document)
            }
            Relayout::Forward => self.layout.continue_layout(&self.session.document)?,
            _ => {
                self.layout.invalidate_all();
                self.layout.layout_document(&self.session.document)
            }
        };
        self.version += 1;
        let version = self.version;
        let current_page = self.current_page;
        let text = match relayout {
            Relayout::Forward => self.snapshot.read().document_text.clone(),
            _ => document_plain_text(&self.session.document),
        };
        let read_only = self.session.document.settings.read_only;
        let relayout_start = self.layout.relayout_start_page() as usize;
        let is_incremental = self.layout.last_pass_incremental();

        let mut rebuilt_pages = Vec::new();
        let new_len = doc_layout.pages.len();

        // An incremental pass leaves page indices stable, so only the pages the
        // engine reported dirty (plus any pages the document grew by) can differ.
        if is_incremental && !self.cached_pages.is_empty() {
            self.cached_pages.truncate(new_len);
            let carried_over = self.cached_pages.len();
            let mut targets: Vec<usize> = self
                .layout
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
                let list = DisplayListBuilder::from_page_without_atlas(page, version);
                let bytes = DisplayListBuilder::to_page_bytes(&list);
                if idx < carried_over
                    && DisplayListBuilder::page_bytes_match_content(
                        &self.cached_pages[idx].bytes,
                        &bytes,
                    )
                {
                    // Reflowed to an identical result: keep the shared Arc so
                    // downstream readers see no churn on untouched pages.
                    continue;
                }
                let snapshot = Arc::new(SinglePageSnapshot {
                    version,
                    bytes: Arc::new(bytes),
                    page_width: page.width,
                    page_height: page.height,
                });
                if idx < self.cached_pages.len() {
                    self.cached_pages[idx] = snapshot;
                } else {
                    self.cached_pages.push(snapshot);
                }
                rebuilt_pages.push(idx as u32);
            }
        } else {
            self.cached_pages.clear();
            self.cached_pages.reserve(new_len);
            for (idx, page) in doc_layout.pages.iter().enumerate() {
                let list = DisplayListBuilder::from_page_without_atlas(page, version);
                self.cached_pages.push(Arc::new(SinglePageSnapshot {
                    version,
                    bytes: Arc::new(DisplayListBuilder::to_page_bytes(&list)),
                    page_width: page.width,
                    page_height: page.height,
                }));
                rebuilt_pages.push(idx as u32);
            }
        }

        let atlas = self.layout.atlas();
        let atlas_generation = atlas.generation;
        let atlas_width = atlas.width;
        let atlas_height = atlas.height;
        let atlas_bytes = if atlas_generation == self.cached_atlas_generation
            && !self.cached_atlas_bytes.is_empty()
        {
            Arc::clone(&self.cached_atlas_bytes)
        } else {
            let bytes = Arc::new(DisplayListBuilder::atlas_to_bytes(atlas, atlas_generation));
            self.cached_atlas_generation = atlas_generation;
            self.cached_atlas_bytes = Arc::clone(&bytes);
            bytes
        };

        let page_count = self.cached_pages.len().max(1) as u32;
        let props_json = document_properties_json(&self.session.document, page_count);
        let page_index = current_page.min(page_count.saturating_sub(1));

        if is_incremental && !self.cached_pages.is_empty() {
            let updated: Vec<Arc<SinglePageSnapshot>> = rebuilt_pages
                .iter()
                .map(|&i| self.cached_pages[i as usize].clone())
                .collect();
            self.snapshot.publish_incremental(
                version,
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
            let pages: Vec<SinglePageSnapshot> =
                self.cached_pages.iter().map(|p| (**p).clone()).collect();
            self.snapshot.publish(snapshot_from_pages(
                pages,
                page_index,
                version,
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
            Relayout::Forward => self.layout_cache.read().document_snapshot(),
            _ => Arc::new(self.session.document.clone()),
        };
        {
            let mut cache = self.layout_cache.write();
            if is_incremental {
                cache.update_from_session_incremental(
                    &self.layout,
                    Arc::clone(&doc_arc),
                    version,
                    page_count,
                    self.layout.has_pending_reflow(),
                );
            } else {
                cache.update_from_session(&self.layout, doc_arc);
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
            version,
            rebuilt_pages,
            ready_page,
        })
    }

    /// Run one command to completion, emitting its events.
    ///
    /// `next_command` is a non-blocking source of further queued commands, used
    /// only to batch consecutive `ApplyEdit`s into a single transaction and
    /// layout pass. A non-edit command pulled from it is stashed in
    /// [`WorkerCore::take_pending`] instead of being consumed.
    pub(crate) fn execute(
        &mut self,
        queued: QueuedCommand,
        next_command: &mut dyn FnMut() -> Option<QueuedCommand>,
    ) -> Flow {
        let QueuedCommand {
            request_id: req_id,
            inner: cmd,
        } = queued;

        match cmd {
            BridgeCommand::NewDocument => {
                self.session = EditSession::new();
                self.format_ctx = FormatContext::new_document();
                self.current_page = 0;
                self.rebuild(Relayout::Full)
                    .expect("full rebuild always produces a layout");
                let page_count = self.layout.page_count() as u32;
                self.events.send(BridgeEvent::DocumentOpened {
                    request_id: req_id,
                    page_count,
                });
            }
            BridgeCommand::OpenDocument {
                data,
                path_hint,
                password,
            } => {
                match import_document_bundle_with_password(
                    &data,
                    path_hint.as_deref(),
                    password.as_deref(),
                ) {
                    Ok(bundle) => {
                        if let Some(pkg) = &bundle.docx_package {
                            if tw_docx::package_has_vba_parts(pkg) {
                                if let Err(message) = self
                                    .policy
                                    .require(PolicyCapability::OpenMacroDocument)
                                {
                                    self.events.send(BridgeEvent::Error {
                                        request_id: req_id,
                                        message,
                                    });
                                    return Flow::Continue;
                                }
                            }
                        }
                        self.format_ctx = FormatContext::from_bundle(bundle.clone(), path_hint);
                        // Retain open password so subsequent DOCX saves stay encrypted (F22.S2).
                        self.format_ctx.encryption_password = password
                            .as_deref()
                            .filter(|p| !p.is_empty())
                            .map(|p| p.to_string());
                        for font in &bundle.embedded_fonts {
                            let _ = self.layout.register_face(&font.spec, font.data.clone());
                        }
                        self.session = EditSession::from_document(bundle.document);
                        self.current_page = 0;
                        self.rebuild(Relayout::Full)
                            .expect("full rebuild always produces a layout");
                        let page_count = self.layout.page_count() as u32;
                        self.events.send(BridgeEvent::DocumentOpened {
                            request_id: req_id,
                            page_count,
                        });
                    }
                    Err(e) => {
                        self.events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: e.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::SaveDocument => {
                match export_document(&self.session.document, &self.format_ctx) {
                    Ok(data) => {
                        self.events.send(BridgeEvent::DocumentSaved {
                            request_id: req_id,
                            data,
                        });
                    }
                    Err(e) => {
                        self.events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: e.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::SaveDocumentAs { format } => {
                self.format_ctx.save_format = format;
                match export_document(&self.session.document, &self.format_ctx) {
                    Ok(data) => {
                        self.events.send(BridgeEvent::DocumentSaved {
                            request_id: req_id,
                            data,
                        });
                    }
                    Err(e) => {
                        self.events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: e.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::SpellCheckDocument => {
                let checker = tw_spell::SpellChecker::english();
                let text = tw_edit::body_plain_text(&self.session.document);
                let issues = checker.check_text(&text);
                let spell_issues: Vec<(String, usize, usize, Vec<String>)> = issues
                    .iter()
                    .map(|i| (i.word.clone(), i.start, i.end, i.suggestions.clone()))
                    .collect();
                let words: Vec<String> = spell_issues.iter().map(|(w, _, _, _)| w.clone()).collect();
                self.events.send(BridgeEvent::SpellCheckResult {
                    request_id: req_id,
                    misspellings: words,
                    spell_issues,
                });
            }
            BridgeCommand::GrammarCheckDocument => {
                let checker = tw_spell::GrammarChecker::english();
                let text = document_plain_text(&self.session.document);
                let issues = checker.check_text(&text);
                let messages: Vec<String> = issues.into_iter().map(|i| i.message).collect();
                self.events.send(BridgeEvent::GrammarCheckResult {
                    request_id: req_id,
                    issues: messages,
                });
            }
            BridgeCommand::SetReadOnly { enabled } => {
                self.session.document.settings.read_only = enabled;
                self.rebuild(Relayout::Full)
                    .expect("full rebuild always produces a layout");
                self.events.send(BridgeEvent::DisplayListReady {
                    request_id: req_id,
                    page: self.current_page,
                    version: self.version,
                });
            }
            BridgeCommand::SetEncryptionPassword { password } => {
                self.format_ctx.encryption_password = password.filter(|p| !p.is_empty());
                self.events.send(BridgeEvent::DisplayListReady {
                    request_id: req_id,
                    page: self.current_page,
                    version: self.version,
                });
            }
            BridgeCommand::ToggleTrackChanges { enabled } => {
                self.session.document.settings.track_changes_enabled = enabled;
            }
            BridgeCommand::PasteHtml {
                run_id,
                offset,
                html,
            } => match tw_html::import(&html) {
                Ok(doc) => {
                    if paste::paste_fragment_at(&mut self.session, run_id, offset, &doc).is_ok() {
                        self.format_ctx.mark_document_modified();
                        self.rebuild(Relayout::Full)
                            .expect("full rebuild always produces a layout");
                        self.events.send(BridgeEvent::DisplayListReady {
                            request_id: req_id,
                            page: self.current_page,
                            version: self.version,
                        });
                    } else {
                        self.events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: "paste HTML failed".into(),
                        });
                    }
                }
                Err(err) => {
                    self.events.send(BridgeEvent::Error {
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
                    if tw_docx::package_has_vba_parts(&result.package) {
                        if let Err(message) = self
                            .policy
                            .require(PolicyCapability::OpenMacroDocument)
                        {
                            self.events.send(BridgeEvent::Error {
                                request_id: req_id,
                                message,
                            });
                            return Flow::Continue;
                        }
                    }
                    if paste::paste_fragment_at(
                        &mut self.session,
                        run_id,
                        offset,
                        &result.document,
                    )
                    .is_ok()
                    {
                        self.format_ctx.mark_document_modified();
                        self.rebuild(Relayout::Full)
                            .expect("full rebuild always produces a layout");
                        self.events.send(BridgeEvent::DisplayListReady {
                            request_id: req_id,
                            page: self.current_page,
                            version: self.version,
                        });
                    } else {
                        self.events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: "paste DOCX fragment failed".into(),
                        });
                    }
                }
                Err(err) => {
                    self.events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: err.to_string(),
                    });
                }
            },
            BridgeCommand::ExportSelectionDocx { range } => {
                match tw_edit::document_from_range(&self.session.document, &range) {
                    Ok(subset) => {
                        let package = tw_docx::DocxPackage::default();
                        match tw_docx::export(&subset, &package) {
                            Ok(data) => {
                                self.events.send(BridgeEvent::DocumentSaved {
                                    request_id: req_id,
                                    data,
                                });
                            }
                            Err(e) => {
                                self.events.send(BridgeEvent::Error {
                                    request_id: req_id,
                                    message: e.to_string(),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        self.events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: e.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::ApplyEdit { command } => {
                if self.session.document.settings.read_only {
                    self.events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: "document is read-only".into(),
                    });
                    return Flow::Continue;
                }
                let mut commands = vec![command];
                let mut request_ids = vec![req_id];
                while let Some(next) = next_command() {
                    match next {
                        QueuedCommand {
                            request_id: batch_id,
                            inner: BridgeCommand::ApplyEdit { command: c },
                        } => {
                            commands.push(c);
                            request_ids.push(batch_id);
                        }
                        other => {
                            self.pending = Some(other);
                            break;
                        }
                    }
                }

                let mut apply_error = None;
                let mut affected_nodes: Vec<NodeId> = Vec::new();
                let mut tx = self.session.begin_transaction(None);
                let mut force_full_relayout = false;
                for command in commands {
                    let is_split = matches!(&command, Command::SplitParagraphAt { .. });
                    match tx.apply(command) {
                        Ok(result) => {
                            if is_split {
                                force_full_relayout = true;
                            }
                            if let Some(run_id) = result.seed_run_id {
                                self.layout_cache
                                    .write()
                                    .set_last_split_caret(run_id, 0);
                            } else if is_split {
                                if let Some(&run_id) = result.affected_nodes.first() {
                                    self.layout_cache
                                        .write()
                                        .set_last_split_caret(run_id, 0);
                                }
                            }
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
                        self.events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: abort_err.to_string(),
                        });
                        return Flow::Continue;
                    }
                    // Successful abort: document unchanged — do not mark dirty or relayout.
                    for batch_id in &request_ids {
                        self.events.send(BridgeEvent::Error {
                            request_id: *batch_id,
                            message: e.to_string(),
                        });
                    }
                } else {
                    tx.commit();
                    self.format_ctx.mark_document_modified();
                    let outcome = self
                        .rebuild(if force_full_relayout || affected_nodes.is_empty() {
                            Relayout::Full
                        } else {
                            Relayout::Nodes(affected_nodes.as_slice())
                        })
                        .expect("edit rebuild always produces a layout");
                    let version = outcome.version;
                    let ready_page = outcome.ready_page;
                    for batch_id in request_ids {
                        self.events.send(BridgeEvent::DisplayListReady {
                            request_id: batch_id,
                            page: ready_page,
                            version,
                        });
                    }
                }
            }
            BridgeCommand::ExportPdf => {
                let exporter = tw_pdf::DisplayListPdfExporter;
                match exporter.export(&self.session.document, &tw_pdf::PdfExportOptions::default())
                {
                    Ok(data) => {
                        self.events.send(BridgeEvent::DocumentSaved {
                            request_id: req_id,
                            data,
                        });
                    }
                    Err(e) => {
                        self.events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: e.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::ExportPdfForPrint { layout, selection } => {
                let result = match selection {
                    Some(range) => tw_pdf::prepare_print_pdf_selection(
                        &self.session.document,
                        &range,
                        &layout,
                    ),
                    None => tw_pdf::prepare_print_pdf(&self.session.document, &layout),
                };
                match result {
                    Ok(data) => {
                        self.events.send(BridgeEvent::DocumentSaved {
                            request_id: req_id,
                            data,
                        });
                    }
                    Err(e) => {
                        self.events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: e.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::SetCurrentPage { page } => {
                self.current_page = page;
                self.snapshot.set_current_page(page);
                let snap = self.snapshot.read();
                self.events.send(BridgeEvent::DisplayListReady {
                    request_id: req_id,
                    page: snap.page_index,
                    version: snap.version,
                });
            }
            BridgeCommand::Undo => match self.session.undo() {
                Ok(Some(_)) => {
                    self.rebuild(Relayout::Full)
                        .expect("full rebuild always produces a layout");
                    self.events.send(BridgeEvent::DisplayListReady {
                        request_id: req_id,
                        page: self.current_page,
                        version: self.version,
                    });
                }
                Ok(None) => {
                    self.events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: "nothing to undo".into(),
                    });
                }
                Err(e) => {
                    self.events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: e.to_string(),
                    });
                }
            },
            BridgeCommand::Redo => match self.session.redo() {
                Ok(Some(_)) => {
                    self.rebuild(Relayout::Full)
                        .expect("full rebuild always produces a layout");
                    self.events.send(BridgeEvent::DisplayListReady {
                        request_id: req_id,
                        page: self.current_page,
                        version: self.version,
                    });
                }
                Ok(None) => {
                    self.events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: "nothing to redo".into(),
                    });
                }
                Err(e) => {
                    self.events.send(BridgeEvent::Error {
                        request_id: req_id,
                        message: e.to_string(),
                    });
                }
            },
            BridgeCommand::Shutdown => return Flow::Shutdown,
        }
        Flow::Continue
    }
}
