use crate::executor::{EngineExecutor, InlineExecutor};
use crate::import::DetectedFormat;
use crate::layout_cache::{new_shared_layout_cache, SharedLayoutCache};
use crate::snapshot::SnapshotBuffer;
use crate::worker::{BridgeCommand, BridgeEvent, QueuedCommand, STARTUP_REQUEST_ID};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tw_edit::{
    adjust_list_level_command_for_caret, autofit_table_command_for_caret,
    bullet_list_command_for_caret,
    continue_numbering_command_for_caret, delete_table_column_command_for_caret,
    delete_table_row_command_for_caret, ensure_header_footer_command_for,
    heading1_command_for_caret, insert_field_command_for, insert_field_command_with_merge,
    insert_footnote_command_for,
    insert_endnote_command_for,
    insert_comment_command_for,
    reply_to_comment_command_for, resolve_comment_command_for,
    insert_table_of_contents_command_for, insert_table_of_figures_command_for,
    insert_bibliography_command_for, insert_bookmark_command_for,
    insert_hyperlink_command_for,
    insert_cross_reference_command_for, insert_index_command_for,
    insert_citation_command_for, add_bibliography_source_command,
    accept_revision_at_caret, reject_revision_at_caret,
    insert_image_bytes_command_for_caret,
    insert_office_math_command_for, insert_office_math_display_command_for_caret,
    set_office_math_command_for,
    insert_image_command, insert_shape_command, insert_text_box_command,
    insert_word_art_command, replace_image_bytes_command,
    insert_nested_table_command_for_caret, insert_page_break_command_for,
    insert_section_break_command_for, insert_table_command_for_caret,
    insert_table_sum_field_command_for_caret, merge_table_cells_right_command_for_caret,
    numbered_list_command_for_caret, paragraph_style_command_for_caret,
    resize_table_column_command_for_caret, restart_numbering_command_for_caret,
    section_index_for_caret, set_header_footer_link_command_for,
    set_table_border_command_for_caret, set_table_cell_shading_command_for_caret,
    sort_table_rows_command_for_caret, split_table_cell_command_for_caret, Command,
    EditSession,
};
use tw_layout::LayoutEngine;
use tw_model::{BorderSpec, Color, Document, FieldType, HeaderFooterType, NodeId};
use tw_native::NativeFormat;
use tw_render::DisplayListBuilder;

pub type DocId = u32;

pub type EventObserver = Arc<dyn Fn(BridgeEvent) + Send + Sync>;

/// Upper bound on events retained for correlated waits before the oldest are shed.
const MAX_PENDING_EVENTS: usize = 1024;

/// Result of waiting for a correlated worker response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaitOutcome {
    Matched(BridgeEvent),
    Timeout,
}

pub struct Session {
    snapshot: Arc<SnapshotBuffer>,
    layout_cache: SharedLayoutCache,
    executor: Box<dyn EngineExecutor>,
    pending_events: Mutex<VecDeque<BridgeEvent>>,
    event_observer: StdMutex<Option<EventObserver>>,
    next_request_id: AtomicU64,
    /// External [`Session::pump_events`] calls, used only to diagnose a host that
    /// installed an observer and then never pumped.
    pump_calls: AtomicU64,
    /// Only a clock-bounded (threaded) wait can time out without making progress.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    unpumped_warning_emitted: AtomicBool,
    pub doc_id: DocId,
}

impl Session {
    /// Session with the platform-default executor: a worker thread on native
    /// targets, an inline (driven) engine on `wasm32`.
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::new_inline()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::new_threaded()
        }
    }

    /// Session whose engine owns a dedicated OS thread (native only).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_threaded() -> Self {
        Self::with_executor(|snapshot, layout_cache| {
            Box::new(crate::executor::ThreadedExecutor::spawn(
                snapshot,
                layout_cache,
            ))
        })
    }

    /// Session with no worker thread: commands run on the caller's thread when
    /// the host drives the engine.
    ///
    /// Available on every target, not just `wasm32`, so the web execution model
    /// is exercised by native tests. See [`Session::drive`] for how a host makes
    /// such a session progress.
    pub fn new_inline() -> Self {
        Self::with_executor(|snapshot, layout_cache| {
            Box::new(InlineExecutor::new(snapshot, layout_cache))
        })
    }

    /// Session driven by a caller-supplied executor. The closure receives the
    /// snapshot buffer and layout cache the session will read from.
    pub fn with_executor<F>(build_executor: F) -> Self
    where
        F: FnOnce(Arc<SnapshotBuffer>, SharedLayoutCache) -> Box<dyn EngineExecutor>,
    {
        let snapshot = Arc::new(SnapshotBuffer::new());
        let layout_cache = new_shared_layout_cache();
        let executor = build_executor(snapshot.clone(), layout_cache.clone());
        Self {
            snapshot,
            layout_cache,
            executor,
            pending_events: Mutex::new(VecDeque::new()),
            event_observer: StdMutex::new(None),
            next_request_id: AtomicU64::new(1),
            pump_calls: AtomicU64::new(0),
            unpumped_warning_emitted: AtomicBool::new(false),
            doc_id: 1,
        }
    }

    /// Run outstanding engine work on the calling thread; returns the number of
    /// work units performed (commands, event flushes, background reflow chunks).
    ///
    /// No-op returning `0` for a threaded session. For an inline session this is
    /// the *only* thing that makes the engine progress, and `0` means the engine
    /// is idle: no further events can appear until another command is submitted.
    /// [`Session::pump_events`] drives and then delivers, so a host on a frame
    /// timer normally calls that instead.
    pub fn drive(&self) -> usize {
        self.executor.drive()
    }

    /// True when the engine only progresses while the host drives it (inline).
    pub fn requires_drive(&self) -> bool {
        self.executor.requires_drive()
    }

    /// Forward every worker event to the host (or tests) before buffering it for
    /// correlated waits.
    ///
    /// **The observer only fires while this session drains its event channel, and
    /// nothing drains it spontaneously.** Enqueuing a command does not; the worker
    /// pushes into a channel and moves on. A host that installs an observer and
    /// then waits for events without calling [`Session::pump_events`] will observe
    /// no events at all, and every correlated wait will run to its full timeout.
    /// On an inline session the stakes are higher still: nothing *executes* the
    /// command either until the host pumps.
    ///
    /// Call `pump_events` on a timer or frame callback for as long as the observer
    /// is installed. The blocking helpers drain as a side effect of waiting, which
    /// is why an unpumped host can appear to work until the last blocking call is
    /// removed.
    pub fn set_event_observer(&self, observer: EventObserver) {
        *self.event_observer.lock().expect("event observer lock") = Some(observer);
    }

    fn drain_events(&self) -> usize {
        let observer = self
            .event_observer
            .lock()
            .expect("event observer lock")
            .clone();
        let mut pending = self.pending_events.lock();
        let mut forwarded = 0;
        while let Some(event) = self.executor.try_next_event() {
            if let Some(ref forward) = observer {
                forward(event.clone());
            }
            pending.push_back(event);
            forwarded += 1;
        }
        // Callers that never poll (the async FFI path drives everything off the
        // observer) would otherwise grow this correlation buffer without bound.
        while pending.len() > MAX_PENDING_EVENTS {
            pending.pop_front();
        }
        forwarded
    }

    /// Deliver newly received worker events to the observer without consuming the
    /// correlation buffer, so a host that drives the engine purely off events can
    /// make progress without a blocking wait. Returns the number forwarded.
    ///
    /// Required, not optional, for any host that installs an event observer -- see
    /// [`Session::set_event_observer`].
    ///
    /// On an inline session this is also what *runs* the engine: it drives every
    /// queued command (plus one background reflow chunk) before delivering, which
    /// is why the FFI/Dart timer pump is the natural host driver on web.
    pub fn pump_events(&self) -> usize {
        self.pump_calls.fetch_add(1, Ordering::Relaxed);
        let _ = self.executor.drive();
        self.drain_events()
    }

    /// A correlated wait that ran to its timeout while an observer was installed
    /// and nothing ever pumped is almost certainly the unpumped-host mistake, not a
    /// slow worker. Say so once, on stderr, rather than letting the host rediscover
    /// it as an unexplained multi-second hang.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    fn warn_if_never_pumped(&self) {
        if self.pump_calls.load(Ordering::Relaxed) != 0 {
            return;
        }
        let has_observer = self
            .event_observer
            .lock()
            .expect("event observer lock")
            .is_some();
        if !has_observer {
            return;
        }
        if self
            .unpumped_warning_emitted
            .swap(true, Ordering::Relaxed)
        {
            return;
        }
        eprintln!(
            "tw-core: a correlated wait timed out and Session::pump_events has never \
             been called. An installed event observer only fires while the event \
             channel is drained; call pump_events (tw_pump_events over FFI) on a \
             timer or frame callback."
        );
    }

    fn send_command(&self, command: BridgeCommand) -> Option<u64> {
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let queued = QueuedCommand {
            request_id,
            inner: command,
        };
        // The executor applies backpressure on a full queue (blocking on the
        // threaded path, draining inline). None only when the engine has shut down.
        self.executor.submit(queued).then_some(request_id)
    }

    pub fn apply(&self, command: Command) -> Option<u64> {
        self.send_command(BridgeCommand::ApplyEdit { command })
    }

    pub fn new_document(&self) -> Option<u64> {
        self.send_command(BridgeCommand::NewDocument)
    }

    pub fn open_bytes(&self, data: Vec<u8>) -> Option<u64> {
        self.open_bytes_with_path(data, None)
    }

    pub fn open_bytes_with_path(&self, data: Vec<u8>, path_hint: Option<String>) -> Option<u64> {
        self.open_bytes_with_path_and_password(data, path_hint, None)
    }

    pub fn open_bytes_with_path_and_password(
        &self,
        data: Vec<u8>,
        path_hint: Option<String>,
        password: Option<String>,
    ) -> Option<u64> {
        self.send_command(BridgeCommand::OpenDocument {
            data,
            path_hint,
            password,
        })
    }

    pub fn save(&self) -> Option<u64> {
        self.send_command(BridgeCommand::SaveDocument)
    }

    pub fn save_as(&self, format: DetectedFormat) -> Option<u64> {
        self.send_command(BridgeCommand::SaveDocumentAs { format })
    }

    pub fn spell_check(&self) -> Option<u64> {
        self.send_command(BridgeCommand::SpellCheckDocument)
    }

    pub fn grammar_check(&self) -> Option<u64> {
        self.send_command(BridgeCommand::GrammarCheckDocument)
    }

    pub fn set_read_only(&self, enabled: bool) -> Option<u64> {
        self.send_command(BridgeCommand::SetReadOnly { enabled })
    }

    /// Set or clear the password used when saving encrypted DOCX (F22.S2).
    pub fn set_encryption_password(&self, password: Option<String>) -> Option<u64> {
        self.send_command(BridgeCommand::SetEncryptionPassword { password })
    }

    pub fn compare_with_text(&self, other: &str) -> tw_model::DocumentCompareSummary {
        let cache = self.layout_cache.read();
        let doc = cache.document();
        tw_model::compare_text(&crate::snapshot::document_plain_text(doc), other)
    }

    pub fn find_matches(
        &self,
        query: &str,
        match_case: bool,
        use_regex: bool,
        use_wildcards: bool,
        format: Option<&tw_edit::FindFormatFilter>,
    ) -> Vec<tw_edit::FindMatch> {
        let cache = self.layout_cache.read();
        let doc = cache.document();
        let Some(range) = tw_edit::document_body_range(doc) else {
            return Vec::new();
        };
        tw_edit::find_matches(
            doc,
            &range,
            query,
            match_case,
            use_regex,
            use_wildcards,
            format,
        )
        .unwrap_or_default()
    }

    pub fn set_track_changes(&self, enabled: bool) -> Option<u64> {
        self.send_command(BridgeCommand::ToggleTrackChanges { enabled })
    }

    pub fn accept_all_revisions(&self) -> Option<u64> {
        self.apply(Command::AcceptAllRevisions)
    }

    pub fn reject_all_revisions(&self) -> Option<u64> {
        self.apply(Command::RejectAllRevisions)
    }

    /// Document Inspector findings JSON (F22.S3).
    pub fn document_inspect_json(&self) -> Option<String> {
        let findings = tw_model::inspect_document(self.document().as_ref());
        serde_json::to_string(&findings).ok()
    }

    /// Remove selected Document Inspector categories (F22.S3).
    pub fn remove_inspect_findings(
        &self,
        comments: bool,
        metadata: bool,
        hidden_text: bool,
    ) -> Option<u64> {
        self.apply(Command::RemoveInspectFindings {
            comments,
            metadata,
            hidden_text,
        })
    }

    /// JSON list of digital signatures (F22.S4).
    pub fn digital_signatures_json(&self) -> Option<String> {
        serde_json::to_string(&self.document().signatures).ok()
    }

    /// JSON verification results for all signatures (F22.S4).
    pub fn verify_signatures_json(&self) -> Option<String> {
        let results = tw_model::verify_all_signatures(self.document().as_ref());
        serde_json::to_string(&results).ok()
    }

    /// Sign the document with signer identity and attach the signature (F22.S4).
    pub fn sign_document(
        &self,
        name: impl Into<String>,
        email: impl Into<String>,
        organization: Option<String>,
    ) -> Result<u64, String> {
        let signer = tw_model::SignerInfo {
            name: name.into(),
            email: email.into(),
            organization,
        };
        let signature = tw_model::sign_document(self.document().as_ref(), signer)?;
        self.apply(Command::AddDigitalSignature { signature })
            .ok_or_else(|| "failed to apply signature".into())
    }

    /// Remove all digital signatures (F22.S4).
    pub fn clear_digital_signatures(&self) -> Option<u64> {
        self.apply(Command::ClearDigitalSignatures)
    }

    pub fn accept_revision_at(&self, caret_run_id: Option<NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| accept_revision_at_caret(doc, caret_run_id))
    }

    pub fn reject_revision_at(&self, caret_run_id: Option<NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| reject_revision_at_caret(doc, caret_run_id))
    }

    pub fn adjacent_revision_run(
        &self,
        caret_run_id: Option<NodeId>,
        forward: bool,
    ) -> Option<NodeId> {
        let caret = caret_run_id?;
        let cache = self.layout_cache.read();
        let doc = cache.document();
        tw_model::adjacent_revision_run(doc, caret, forward)
    }

    /// Returns the next buffered or freshly received event (any request id).
    ///
    /// Drives an inline engine first, so a host can poll without ever blocking.
    pub fn poll_event(&self) -> Option<BridgeEvent> {
        let _ = self.executor.drive();
        let _ = self.drain_events();
        self.pending_events.lock().pop_front()
    }

    fn take_response(&self, request_id: u64) -> Option<BridgeEvent> {
        let mut pending = self.pending_events.lock();
        let index = pending
            .iter()
            .position(|event| event.request_id() == request_id)?;
        Some(pending.remove(index).expect("event index"))
    }

    /// Wait until an event with `request_id` arrives, buffering unrelated events.
    ///
    /// On a threaded session this waits up to `timeout` for the worker thread. On
    /// an inline session there is no other thread to wait *for*, so the wait
    /// drives the engine instead: it runs queued work until the response appears
    /// or the engine reports itself idle, and `timeout` is ignored. That keeps
    /// the call bounded by work rather than by a clock, which matters because
    /// `Instant::now` is not usable on `wasm32-unknown-unknown`.
    pub fn wait_for_response(&self, request_id: u64, timeout: Duration) -> WaitOutcome {
        if self.executor.requires_drive() {
            return self.wait_for_response_driven(request_id);
        }
        self.wait_for_response_blocking(request_id, timeout)
    }

    /// Correlated wait for an inline engine: drive, check, repeat until idle.
    fn wait_for_response_driven(&self, request_id: u64) -> WaitOutcome {
        loop {
            let _ = self.drain_events();
            if let Some(event) = self.take_response(request_id) {
                return WaitOutcome::Matched(event);
            }
            if self.executor.drive() == 0 {
                // Idle engine: nothing further can arrive without new input.
                let _ = self.drain_events();
                return match self.take_response(request_id) {
                    Some(event) => WaitOutcome::Matched(event),
                    None => WaitOutcome::Timeout,
                };
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn wait_for_response_blocking(&self, request_id: u64, timeout: Duration) -> WaitOutcome {
        let started = std::time::Instant::now();
        let deadline = started + timeout;
        loop {
            let _ = self.drain_events();
            if let Some(event) = self.take_response(request_id) {
                return WaitOutcome::Matched(event);
            }
            if std::time::Instant::now() >= deadline {
                self.warn_if_never_pumped();
                return WaitOutcome::Timeout;
            }
            if started.elapsed() < Duration::from_millis(8) {
                std::hint::spin_loop();
            } else {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }

    /// No threaded executor exists on `wasm32`, so this is unreachable there;
    /// it stays compilable (and sleep-free) by deferring to the driven wait.
    #[cfg(target_arch = "wasm32")]
    fn wait_for_response_blocking(&self, request_id: u64, _timeout: Duration) -> WaitOutcome {
        self.wait_for_response_driven(request_id)
    }

    pub fn get_display_list_bytes(&self) -> Arc<crate::snapshot::PageSnapshot> {
        self.snapshot.read()
    }

    /// Read-only document snapshot from the layout cache (versioned with layout).
    pub fn document(&self) -> Arc<Document> {
        self.layout_cache.read().document_snapshot()
    }

    /// Display list for an arbitrary page without disturbing the current page.
    pub fn page_display_list(&self, page: u32) -> Option<Arc<crate::snapshot::SinglePageSnapshot>> {
        self.snapshot.read().page(page)
    }

    pub fn page_display_version(&self, page: u32) -> Option<u64> {
        self.page_display_list(page).map(|p| p.version)
    }

    /// Atlas generation without touching the pixel buffer, so callers can skip a
    /// fetch when the atlas has not changed.
    pub fn atlas_generation(&self) -> u64 {
        self.snapshot.read().atlas_generation
    }

    pub fn atlas_resource(&self) -> (u64, u32, u32, Arc<Vec<u8>>) {
        let snap = self.snapshot.read();
        (
            snap.atlas_generation,
            snap.atlas_width,
            snap.atlas_height,
            snap.atlas_bytes.clone(),
        )
    }

    /// Plain text of the current document snapshot (for hosts without display-list parsing).
    pub fn document_text(&self) -> String {
        self.get_display_list_bytes().document_text.clone()
    }

    /// Width of the first laid-out line on a page, or `0` when unknown.
    pub fn first_line_width(&self, page: u32) -> f32 {
        self.layout_cache.read().first_line_width(page)
    }

    /// Inject a font face. Works on inline, wasm, and threaded (mobile) sessions.
    pub fn register_face(
        &self,
        spec: &tw_layout::FontFaceSpec,
        data: Vec<u8>,
    ) -> Result<tw_layout::FontId, tw_layout::FontRegistrationError> {
        match self.executor.register_face(spec, data) {
            Some(result) => result,
            None => Err(tw_layout::FontRegistrationError::RegistrationNotSupported),
        }
    }

    pub fn set_current_page(&self, page: u32) -> Option<u64> {
        self.send_command(BridgeCommand::SetCurrentPage { page })
    }

    pub fn undo(&self) -> Option<u64> {
        self.send_command(BridgeCommand::Undo)
    }

    pub fn redo(&self) -> Option<u64> {
        self.send_command(BridgeCommand::Redo)
    }

    fn apply_from_document<F>(&self, build: F) -> Option<u64>
    where
        F: FnOnce(&tw_model::Document) -> Option<Command>,
    {
        let cache = self.layout_cache.read();
        let doc = cache.document();
        build(doc).and_then(|command| self.apply(command))
    }

    pub fn apply_heading1(&self) -> Option<u64> {
        self.apply_heading1_at(None)
    }

    pub fn apply_heading1_at(&self, caret_run_id: Option<tw_model::NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| heading1_command_for_caret(doc, caret_run_id))
    }

    pub fn apply_normal_style(&self) -> Option<u64> {
        self.apply_normal_style_at(None)
    }

    pub fn apply_normal_style_at(&self, caret_run_id: Option<tw_model::NodeId>) -> Option<u64> {
        self.apply_paragraph_style_at(caret_run_id, "Normal")
    }

    pub fn apply_paragraph_style_at(
        &self,
        caret_run_id: Option<tw_model::NodeId>,
        style_name: &str,
    ) -> Option<u64> {
        let name = style_name.to_string();
        self.apply_from_document(|doc| {
            paragraph_style_command_for_caret(doc, caret_run_id, &name)
        })
    }

    pub fn apply_document_theme(&self, theme_name: &str) -> Option<u64> {
        let name = theme_name.to_string();
        self.apply(Command::SetDocumentTheme { theme_name: name })
    }

    pub fn apply_section_format(
        &self,
        section_index: usize,
        format: tw_model::SectionFormat,
    ) -> Option<u64> {
        self.apply(Command::SetSectionFormat {
            section_index,
            format,
        })
    }

    pub fn apply_section_format_at(
        &self,
        caret_run_id: Option<NodeId>,
        format: tw_model::SectionFormat,
    ) -> Option<u64> {
        let cache = self.layout_cache.read();
        let section_index = section_index_for_caret(cache.document(), caret_run_id);
        drop(cache);
        self.apply_section_format(section_index, format)
    }

    pub fn section_format_json(&self) -> Option<String> {
        self.section_format_json_at(None)
    }

    pub fn section_format_json_at(&self, caret_run_id: Option<NodeId>) -> Option<String> {
        let cache = self.layout_cache.read();
        let section_index = section_index_for_caret(cache.document(), caret_run_id);
        let format = cache.document().sections.get(section_index)?.format.clone();
        serde_json::to_string(&format).ok()
    }

    pub fn apply_bullet_list(&self) -> Option<u64> {
        self.apply_bullet_list_at(None)
    }

    pub fn apply_bullet_list_at(&self, caret_run_id: Option<tw_model::NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| bullet_list_command_for_caret(doc, caret_run_id))
    }

    pub fn apply_numbered_list(&self) -> Option<u64> {
        self.apply_numbered_list_at(None)
    }

    pub fn apply_numbered_list_at(&self, caret_run_id: Option<tw_model::NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| numbered_list_command_for_caret(doc, caret_run_id))
    }

    /// Promote (+1) or demote (−1) list level at the caret paragraph.
    ///
    /// Returns `None` when the caret is not in a list. Returns `Some(None)` when
    /// already at the min/max level (no edit enqueued).
    pub fn adjust_list_level_at(
        &self,
        caret_run_id: Option<tw_model::NodeId>,
        delta: i32,
    ) -> Option<Option<u64>> {
        let cache = self.layout_cache.read();
        let doc = cache.document();
        let para_id = tw_edit::paragraph_id_from_caret(doc, caret_run_id)?;
        let (si, bi) = doc.find_paragraph_location(para_id)?;
        if doc.paragraph_at(si, bi)?.format.numbering.is_none() {
            return None;
        }
        drop(cache);
        match self.apply_from_document(|doc| {
            adjust_list_level_command_for_caret(doc, caret_run_id, delta)
        }) {
            Some(request_id) => Some(Some(request_id)),
            None => Some(None),
        }
    }

    pub fn restart_numbering_at(&self, caret_run_id: Option<tw_model::NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| restart_numbering_command_for_caret(doc, caret_run_id))
    }

    pub fn continue_numbering_at(&self, caret_run_id: Option<tw_model::NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| continue_numbering_command_for_caret(doc, caret_run_id))
    }

    pub fn insert_table(&self, rows: u32, cols: u32) -> Option<u64> {
        self.insert_table_at(None, rows, cols)
    }

    pub fn insert_table_at(
        &self,
        caret_run_id: Option<NodeId>,
        rows: u32,
        cols: u32,
    ) -> Option<u64> {
        self.apply_from_document(|doc| {
            insert_table_command_for_caret(doc, caret_run_id, rows, cols)
        })
    }

    pub fn delete_table_row_at(&self, caret_run_id: Option<NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| delete_table_row_command_for_caret(doc, caret_run_id))
    }

    pub fn delete_table_column_at(&self, caret_run_id: Option<NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| delete_table_column_command_for_caret(doc, caret_run_id))
    }

    pub fn merge_table_cells_at(&self, caret_run_id: Option<NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| merge_table_cells_right_command_for_caret(doc, caret_run_id))
    }

    pub fn split_table_cell_at(&self, caret_run_id: Option<NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| split_table_cell_command_for_caret(doc, caret_run_id))
    }

    pub fn set_table_border_at(
        &self,
        caret_run_id: Option<NodeId>,
        border: Option<BorderSpec>,
    ) -> Option<u64> {
        self.apply_from_document(|doc| {
            set_table_border_command_for_caret(doc, caret_run_id, border)
        })
    }

    pub fn set_table_cell_shading_at(
        &self,
        caret_run_id: Option<NodeId>,
        background: Option<Color>,
    ) -> Option<u64> {
        self.apply_from_document(|doc| {
            set_table_cell_shading_command_for_caret(doc, caret_run_id, background)
        })
    }

    pub fn resize_table_column_at(
        &self,
        caret_run_id: Option<NodeId>,
        width: f32,
    ) -> Option<u64> {
        self.apply_from_document(|doc| {
            resize_table_column_command_for_caret(doc, caret_run_id, width)
        })
    }

    pub fn autofit_table_at(&self, caret_run_id: Option<NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| autofit_table_command_for_caret(doc, caret_run_id))
    }

    pub fn sort_table_rows_at(
        &self,
        caret_run_id: Option<NodeId>,
        ascending: bool,
    ) -> Option<u64> {
        self.apply_from_document(|doc| {
            sort_table_rows_command_for_caret(doc, caret_run_id, ascending)
        })
    }

    pub fn insert_nested_table_at(
        &self,
        caret_run_id: Option<NodeId>,
        rows: u32,
        cols: u32,
    ) -> Option<u64> {
        self.apply_from_document(|doc| {
            insert_nested_table_command_for_caret(doc, caret_run_id, rows, cols)
        })
    }

    pub fn insert_table_sum_field_at(&self, caret_run_id: Option<NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| {
            insert_table_sum_field_command_for_caret(doc, caret_run_id)
        })
    }

    pub fn insert_image(&self, width: f32, height: f32) -> Option<u64> {
        self.apply_from_document(|doc| insert_image_command(doc, width, height))
    }

    pub fn insert_shape(&self, shape_type: tw_model::ShapeKind) -> Option<u64> {
        self.apply_from_document(|doc| insert_shape_command(doc, shape_type))
    }

    pub fn insert_text_box(&self) -> Option<u64> {
        self.apply_from_document(insert_text_box_command)
    }

    pub fn insert_word_art(&self, text: String) -> Option<u64> {
        self.apply_from_document(|doc| insert_word_art_command(doc, text))
    }

    pub fn insert_diagram(&self) -> Option<u64> {
        self.insert_diagram_with_kind(tw_model::DiagramKind::Process)
    }

    pub fn insert_diagram_with_kind(&self, kind: tw_model::DiagramKind) -> Option<u64> {
        self.apply_from_document(|doc| {
            tw_edit::command_builders::insert_diagram_command_with_kind(doc, kind)
        })
    }

    pub fn insert_chart(&self) -> Option<u64> {
        self.insert_chart_with_kind(tw_model::ChartKind::Column)
    }

    pub fn insert_chart_with_kind(&self, kind: tw_model::ChartKind) -> Option<u64> {
        self.apply_from_document(|doc| {
            tw_edit::command_builders::insert_chart_command_with_kind(doc, kind)
        })
    }

    /// JSON for the editable dataset on a chart shape, if present.
    pub fn chart_data_json(&self, shape_id: tw_model::NodeId) -> Option<String> {
        let doc = self.document();
        let (si, bi) = doc.find_block_location(shape_id)?;
        let shape = doc.sections.get(si)?.blocks.get(bi)?.shape()?;
        if shape.shape.shape_type != tw_model::ShapeKind::Chart {
            return None;
        }
        let data = shape.chart_data.as_ref()?;
        serde_json::to_string(data).ok()
    }

    /// Most recently inserted chart block id (document order, last wins).
    pub fn latest_chart_id(&self) -> Option<tw_model::NodeId> {
        let doc = self.document();
        for section in doc.sections.iter().rev() {
            for block in section.blocks.iter().rev() {
                if let Some(shape) = block.shape() {
                    if shape.shape.shape_type == tw_model::ShapeKind::Chart {
                        return Some(shape.id);
                    }
                }
            }
        }
        None
    }

    pub fn set_chart_data(
        &self,
        shape_id: tw_model::NodeId,
        chart_data: Option<tw_model::ChartData>,
    ) -> Option<u64> {
        self.apply(Command::SetChartData {
            shape_id,
            chart_data,
        })
    }

    /// Insert inline OMML at the caret — F14.S3.
    pub fn insert_office_math_at(
        &self,
        run_id: tw_model::NodeId,
        offset: usize,
        xml: String,
    ) -> Option<u64> {
        self.apply(insert_office_math_command_for(run_id, offset, xml))
    }

    /// Insert a display equation block after the caret paragraph — F14.S3.
    pub fn insert_office_math_display(
        &self,
        caret_run_id: Option<tw_model::NodeId>,
        xml: String,
    ) -> Option<u64> {
        self.apply_from_document(|doc| {
            insert_office_math_display_command_for_caret(doc, caret_run_id, xml)
        })
    }

    /// Replace OMML on an existing equation run — F14.S3.
    pub fn set_office_math(&self, run_id: tw_model::NodeId, xml: String) -> Option<u64> {
        self.apply(set_office_math_command_for(run_id, xml))
    }

    /// OMML XML stored on [run_id], if it is an equation run.
    pub fn office_math_xml(&self, run_id: tw_model::NodeId) -> Option<String> {
        let doc = self.document();
        let loc = doc.find_run_location(run_id)?;
        let run = doc.run_at(loc)?;
        match &run.content {
            tw_model::RunContent::OfficeMath { xml } => Some(xml.clone()),
            _ => None,
        }
    }

    /// Most recently inserted equation run id (document order, last wins).
    pub fn latest_office_math_run_id(&self) -> Option<tw_model::NodeId> {
        let doc = self.document();
        let mut last = None;
        for section in &doc.sections {
            for block in &section.blocks {
                if let Some(para) = block.paragraph() {
                    for run in &para.runs {
                        if matches!(run.content, tw_model::RunContent::OfficeMath { .. }) {
                            last = Some(run.id);
                        }
                    }
                }
            }
        }
        last
    }

    pub fn delete_block(&self, id: tw_model::NodeId) -> Option<u64> {
        self.apply(Command::DeleteBlock { id })
    }

    pub fn insert_image_bytes(&self, bytes: Vec<u8>, mime_type: String) -> Option<u64> {
        self.insert_image_bytes_at(bytes, mime_type, None)
    }

    pub fn insert_image_bytes_at(
        &self,
        bytes: Vec<u8>,
        mime_type: String,
        caret_run_id: Option<tw_model::NodeId>,
    ) -> Option<u64> {
        self.apply_from_document(|doc| {
            insert_image_bytes_command_for_caret(doc, caret_run_id, bytes, mime_type)
        })
    }

    pub fn set_image_size(&self, image_id: tw_model::NodeId, width: f32, height: f32) -> Option<u64> {
        self.apply(Command::SetImageSize {
            image_id,
            width,
            height,
        })
    }

    pub fn replace_image_bytes(
        &self,
        image_id: tw_model::NodeId,
        bytes: Vec<u8>,
        mime_type: String,
    ) -> Option<u64> {
        self.apply(replace_image_bytes_command(image_id, bytes, mime_type))
    }

    pub fn set_image_wrap(&self, image_id: tw_model::NodeId, wrap: tw_model::TextWrap) -> Option<u64> {
        self.apply(Command::SetImageWrap { image_id, wrap })
    }

    pub fn set_image_anchor(
        &self,
        image_id: tw_model::NodeId,
        anchor: tw_model::ImageAnchor,
    ) -> Option<u64> {
        self.apply(Command::SetImageAnchor { image_id, anchor })
    }

    pub fn set_image_transform(
        &self,
        image_id: tw_model::NodeId,
        transform: tw_model::ImageTransform,
    ) -> Option<u64> {
        self.apply(Command::SetImageTransform {
            image_id,
            transform,
        })
    }

    pub fn insert_image_caption(&self, image_id: tw_model::NodeId) -> Option<u64> {
        self.apply(Command::InsertImageCaption { image_id })
    }

    pub fn set_image_alt_text(
        &self,
        image_id: tw_model::NodeId,
        alt_text: Option<String>,
    ) -> Option<u64> {
        self.apply(Command::SetImageAltText {
            image_id,
            alt_text,
        })
    }

    /// Alternative text for [image_id], or empty string when unset (F21.S3).
    pub fn image_alt_text(&self, image_id: tw_model::NodeId) -> Option<String> {
        let doc = self.document();
        let (si, bi) = doc.find_block_location(image_id)?;
        let image = doc.sections.get(si)?.blocks.get(bi)?.image()?;
        Some(image.alt_text.clone().unwrap_or_default())
    }

    pub fn compress_image(&self, image_id: tw_model::NodeId, quality: u8) -> Option<u64> {
        self.apply(Command::CompressImage { image_id, quality })
    }

    pub fn insert_page_break(&self) -> Option<u64> {
        self.insert_page_break_at(None)
    }

    pub fn insert_page_break_at(&self, caret_run_id: Option<tw_model::NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| insert_page_break_command_for(doc, caret_run_id))
    }

    pub fn insert_section_break(&self) -> Option<u64> {
        self.insert_section_break_at(None)
    }

    pub fn insert_section_break_at(&self, caret_run_id: Option<tw_model::NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| insert_section_break_command_for(doc, caret_run_id))
    }

    pub fn ensure_header_footer_at(
        &self,
        caret_run_id: Option<NodeId>,
        is_header: bool,
        page_index: Option<u32>,
    ) -> Option<u64> {
        self.apply_from_document(|doc| {
            ensure_header_footer_command_for(doc, caret_run_id, is_header, page_index)
        })
    }

    pub fn set_even_and_odd_headers(&self, enabled: bool) -> Option<u64> {
        self.apply(Command::SetEvenAndOddHeaders { enabled })
    }

    pub fn header_footer_seed_run_at(
        &self,
        caret_run_id: Option<NodeId>,
        is_header: bool,
        page_index: Option<u32>,
    ) -> Option<NodeId> {
        let doc = self.document();
        let section_index = section_index_for_caret(&doc, caret_run_id);
        let page_index = page_index.unwrap_or(0);
        let section = doc.sections.get(section_index)?;
        let is_first = section_index == 0 && page_index == 0;
        let hf_type = HeaderFooterType::for_page_layout(
            page_index + 1,
            is_first,
            section.format.different_first_page,
            doc.settings.even_and_odd_headers,
        );
        doc.header_footer_seed_run(section_index, is_header, hf_type)
    }

    pub fn header_footer_linked_at(
        &self,
        caret_run_id: Option<NodeId>,
        is_header: bool,
        page_index: Option<u32>,
    ) -> bool {
        let doc = self.document();
        let section_index = section_index_for_caret(&doc, caret_run_id);
        let page_index = page_index.unwrap_or(0);
        let Some(section) = doc.sections.get(section_index) else {
            return false;
        };
        let is_first = section_index == 0 && page_index == 0;
        let hf_type = HeaderFooterType::for_page_layout(
            page_index + 1,
            is_first,
            section.format.different_first_page,
            doc.settings.even_and_odd_headers,
        );
        doc.header_footer_linked(section_index, is_header, hf_type)
    }

    pub fn set_header_footer_link_at(
        &self,
        caret_run_id: Option<NodeId>,
        is_header: bool,
        page_index: Option<u32>,
        linked: bool,
    ) -> Option<u64> {
        self.apply_from_document(|doc| {
            set_header_footer_link_command_for(doc, caret_run_id, is_header, page_index, linked)
        })
    }

    pub fn insert_field_at(
        &self,
        run_id: NodeId,
        offset: usize,
        field_type: FieldType,
        merge_name: Option<String>,
    ) -> Option<u64> {
        self.apply(insert_field_command_with_merge(
            run_id,
            offset,
            field_type,
            merge_name,
        ))
    }

    pub fn export_selection_docx(&self, range: tw_edit::DocRange) -> Option<u64> {
        self.send_command(BridgeCommand::ExportSelectionDocx { range })
    }

    pub fn reply_to_comment(&self, comment_id: i32, body_text: String) -> Option<u64> {
        self.apply(reply_to_comment_command_for(comment_id, body_text))
    }

    pub fn resolve_comment(&self, comment_id: i32, resolved: bool) -> Option<u64> {
        self.apply(resolve_comment_command_for(comment_id, resolved))
    }

    pub fn insert_form_field_at(
        &self,
        run_id: NodeId,
        offset: usize,
        kind: tw_model::FormFieldKind,
        name: Option<String>,
        initial_value: Option<String>,
    ) -> Option<u64> {
        self.apply(Command::InsertFormField {
            run_id,
            offset,
            kind,
            name,
            initial_value,
        })
    }

    pub fn set_form_field_value_at(
        &self,
        run_id: NodeId,
        value: impl Into<String>,
    ) -> Option<u64> {
        self.apply(Command::SetFormFieldValue {
            run_id,
            value: value.into(),
        })
    }

    pub fn insert_merge_field_at(
        &self,
        run_id: NodeId,
        offset: usize,
        name: impl Into<String>,
    ) -> Option<u64> {
        self.apply(Command::InsertMergeField {
            run_id,
            offset,
            name: name.into(),
        })
    }

    pub fn apply_mail_merge_row_at(
        &self,
        values: std::collections::BTreeMap<String, String>,
    ) -> Option<u64> {
        self.apply(Command::ApplyMailMergeRow { values })
    }

    pub fn insert_footnote_at(&self, run_id: NodeId, offset: usize) -> Option<u64> {
        self.apply(insert_footnote_command_for(run_id, offset))
    }

    pub fn insert_endnote_at(&self, run_id: NodeId, offset: usize) -> Option<u64> {
        self.apply(insert_endnote_command_for(run_id, offset))
    }

    pub fn insert_comment_at(
        &self,
        run_id: NodeId,
        offset: usize,
        body_text: impl Into<String>,
    ) -> Option<u64> {
        self.apply(insert_comment_command_for(run_id, offset, body_text))
    }

    pub fn insert_table_of_contents_at(
        &self,
        caret_run_id: Option<NodeId>,
    ) -> Option<u64> {
        let cache = self.layout_cache.read();
        let doc = cache.document();
        let page_numbers: Vec<u32> = cache
            .document_outline_with_pages()
            .into_iter()
            .map(|(_, page)| page.max(1))
            .collect();
        let command =
            insert_table_of_contents_command_for(doc, caret_run_id, page_numbers)?;
        drop(cache);
        self.apply(command)
    }

    pub fn insert_table_of_figures_at(
        &self,
        caret_run_id: Option<NodeId>,
    ) -> Option<u64> {
        let cache = self.layout_cache.read();
        let doc = cache.document();
        let page_numbers: Vec<u32> = cache
            .document_captions_with_pages()
            .into_iter()
            .map(|(_, page)| page.max(1))
            .collect();
        let command =
            insert_table_of_figures_command_for(doc, caret_run_id, page_numbers)?;
        drop(cache);
        self.apply(command)
    }

    pub fn add_bibliography_source(&self, source: tw_model::BibliographySource) -> Option<u64> {
        self.apply(add_bibliography_source_command(source))
    }

    pub fn insert_citation_at(
        &self,
        run_id: NodeId,
        offset: usize,
        source_key: &str,
    ) -> Option<u64> {
        self.apply(insert_citation_command_for(
            run_id,
            offset,
            source_key.to_string(),
        ))
    }

    pub fn insert_bibliography_at(&self, caret_run_id: Option<NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| insert_bibliography_command_for(doc, caret_run_id))
    }

    pub fn insert_bookmark_at(
        &self,
        run_id: NodeId,
        offset: usize,
        name: &str,
    ) -> Option<u64> {
        self.apply(insert_bookmark_command_for(run_id, offset, name.to_string()))
    }

    pub fn insert_hyperlink_at(
        &self,
        run_id: NodeId,
        offset: usize,
        url: &str,
        text: &str,
        tooltip: Option<&str>,
    ) -> Option<u64> {
        self.apply(insert_hyperlink_command_for(
            run_id,
            offset,
            url.to_string(),
            text.to_string(),
            tooltip.map(str::to_string),
        ))
    }

    pub fn insert_cross_reference_at(
        &self,
        run_id: NodeId,
        offset: usize,
        bookmark_name: &str,
    ) -> Option<u64> {
        self.apply(insert_cross_reference_command_for(
            run_id,
            offset,
            bookmark_name.to_string(),
        ))
    }

    pub fn insert_index_at(&self, caret_run_id: Option<NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| insert_index_command_for(doc, caret_run_id))
    }

    pub fn paste_html_at(&self, run_id: tw_model::NodeId, offset: usize, html: Vec<u8>) -> Option<u64> {
        self.send_command(BridgeCommand::PasteHtml {
            run_id,
            offset,
            html,
        })
    }

    pub fn paste_docx_at(&self, run_id: tw_model::NodeId, offset: usize, bytes: Vec<u8>) -> Option<u64> {
        self.send_command(BridgeCommand::PasteDocx {
            run_id,
            offset,
            bytes,
        })
    }

    pub fn export_pdf(&self) -> Option<u64> {
        self.send_command(BridgeCommand::ExportPdf)
    }

    /// Export a print-ready (VisualMatch with structural fallback) PDF (F25.S1–S3).
    ///
    /// Pass `selection` to print only that body range (F25.S3).
    pub fn export_pdf_for_print(
        &self,
        layout: tw_pdf::PrintLayoutOptions,
        selection: Option<tw_edit::DocRange>,
    ) -> Option<u64> {
        self.send_command(BridgeCommand::ExportPdfForPrint { layout, selection })
    }

    pub fn page_count(&self) -> u32 {
        self.snapshot.read().page_count.max(1)
    }

    pub fn hit_test(&self, page: u32, x: f32, y: f32) -> Option<tw_layout::HitTestResult> {
        self.layout_cache.read().hit_test(page, x, y)
    }

    pub fn document_tail_hit(&self, page: u32) -> Option<tw_layout::HitTestResult> {
        self.layout_cache.read().document_tail_hit(page)
    }

    pub fn caret_geometry(&self, page: u32, x: f32, y: f32) -> Option<(f32, f32, f32)> {
        self.layout_cache.read().caret_geometry(page, x, y)
    }

    pub fn caret_at(&self, page: u32, run_id: tw_model::NodeId, char_offset: usize) -> Option<(f32, f32, f32)> {
        self.layout_cache.read().caret_at(page, run_id, char_offset)
    }

    pub fn selection_rects(&self, page: u32, start_x: f32, start_y: f32, end_x: f32, end_y: f32) -> Vec<f32> {
        self.layout_cache.read().selection_rects(page, start_x, start_y, end_x, end_y)
    }

    pub fn text_in_range(
        &self,
        start_run: tw_model::NodeId,
        start_offset: usize,
        end_run: tw_model::NodeId,
        end_offset: usize,
    ) -> Option<String> {
        self.layout_cache
            .read()
            .text_in_range(start_run, start_offset, end_run, end_offset)
    }

    pub fn caret_format_json(&self, run_id: tw_model::NodeId) -> Option<String> {
        let cache = self.layout_cache.read();
        let (char_format, para_format, style_name) = cache.format_at(run_id)?;
        let inspector_summary =
            tw_model::style_inspector_at(cache.document(), run_id).map(|s| s.display());
        #[derive(serde::Serialize)]
        struct CaretFormatResponse {
            char_format: tw_model::CharFormat,
            para_format: tw_model::ParaFormat,
            style_name: Option<String>,
            inspector_summary: Option<String>,
        }
        serde_json::to_string(&CaretFormatResponse {
            char_format,
            para_format,
            style_name,
            inspector_summary,
        })
        .ok()
    }

    pub fn document_outline_json(&self) -> Option<String> {
        #[derive(serde::Serialize)]
        struct OutlineEntryResponse {
            paragraph_id: tw_model::NodeId,
            level: u8,
            text: String,
            run_id: tw_model::NodeId,
            page: u32,
        }

        let entries = self
            .layout_cache
            .read()
            .document_outline_with_pages()
            .into_iter()
            .map(|(entry, page)| OutlineEntryResponse {
                paragraph_id: entry.paragraph_id,
                level: entry.level,
                text: entry.text,
                run_id: entry.run_id,
                page,
            })
            .collect::<Vec<_>>();
        serde_json::to_string(&entries).ok()
    }

    /// JSON array of bookmarks: `{ name, run_id, paragraph_id, page }` (F19.S4).
    pub fn bookmarks_json(&self) -> Option<String> {
        #[derive(serde::Serialize)]
        struct BookmarkEntryResponse {
            name: String,
            run_id: tw_model::NodeId,
            paragraph_id: tw_model::NodeId,
            page: u32,
        }

        let entries = self
            .layout_cache
            .read()
            .bookmarks_with_pages()
            .into_iter()
            .map(|(entry, page)| BookmarkEntryResponse {
                name: entry.name,
                run_id: entry.run_id,
                paragraph_id: entry.paragraph_id,
                page,
            })
            .collect::<Vec<_>>();
        serde_json::to_string(&entries).ok()
    }

    /// JSON semantic accessibility tree (F21.S1).
    pub fn semantic_tree_json(&self) -> Option<String> {
        let tree = tw_model::semantic_document_tree(self.document().as_ref());
        serde_json::to_string(&tree).ok()
    }

    /// JSON accessibility checker issues (F21.S4).
    pub fn accessibility_issues_json(&self) -> Option<String> {
        let issues = tw_model::check_accessibility(self.document().as_ref());
        serde_json::to_string(&issues).ok()
    }

    pub fn layout_page_count(&self) -> u32 {
        self.layout_cache.read().page_count()
    }

    /// True while `page` still carries pre-edit geometry awaiting background reflow.
    pub fn is_page_stale(&self, page: u32) -> bool {
        self.layout_cache.read().is_page_stale(page)
    }

    /// Wait for any event. Inline sessions drive the engine instead of sleeping,
    /// returning `None` once the engine is idle rather than burning `timeout_ms`.
    pub fn wait_for_event(&self, timeout_ms: u64) -> Option<BridgeEvent> {
        if self.executor.requires_drive() {
            return self.wait_for_event_driven();
        }
        self.wait_for_event_blocking(timeout_ms)
    }

    fn wait_for_event_driven(&self) -> Option<BridgeEvent> {
        loop {
            let _ = self.drain_events();
            if let Some(event) = self.pending_events.lock().pop_front() {
                return Some(event);
            }
            if self.executor.drive() == 0 {
                let _ = self.drain_events();
                return self.pending_events.lock().pop_front();
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn wait_for_event_blocking(&self, timeout_ms: u64) -> Option<BridgeEvent> {
        let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
        while std::time::Instant::now() < deadline {
            if let Some(event) = self.poll_event() {
                return Some(event);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        None
    }

    #[cfg(target_arch = "wasm32")]
    fn wait_for_event_blocking(&self, _timeout_ms: u64) -> Option<BridgeEvent> {
        self.wait_for_event_driven()
    }

    /// Wait for the worker startup `DocumentOpened` event (`STARTUP_REQUEST_ID`).
    pub fn wait_for_startup(&self, timeout: Duration) -> WaitOutcome {
        self.wait_for_response(STARTUP_REQUEST_ID, timeout)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.executor.shutdown();
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Synchronous path for tests without worker thread.
pub struct SyncSession {
    pub edit: EditSession,
    pub layout: LayoutEngine,
    pub version: u64,
}

impl SyncSession {
    pub fn new() -> Self {
        Self {
            edit: EditSession::new(),
            layout: LayoutEngine::new(),
            version: 0,
        }
    }

    pub fn apply(&mut self, command: Command) -> tw_edit::EditResult {
        let result = self.edit.apply(command).unwrap();
        self.relayout(Some(&result.affected_nodes));
        result
    }

    pub fn relayout(&mut self, affected_nodes: Option<&[NodeId]>) {
        match affected_nodes {
            Some(nodes) if !nodes.is_empty() => {
                self.layout.invalidate_nodes(&self.edit.document, nodes);
            }
            _ => self.layout.invalidate_all(),
        }
        let _layout = self.layout.layout_document(&self.edit.document);
        self.version += 1;
    }

    pub fn display_list_bytes(&mut self) -> Vec<u8> {
        self.relayout(None);
        let layout = self.layout.document_layout();
        let page = layout.pages.first().expect("at least one page");
        let list = DisplayListBuilder::from_page(page, self.layout.atlas(), self.version);
        DisplayListBuilder::to_bytes(&list)
    }

    pub fn page_count(&self) -> u32 {
        self.layout.page_count() as u32
    }

    pub fn export(&self) -> Vec<u8> {
        NativeFormat::export(&self.edit.document).unwrap()
    }

    pub fn import(data: &[u8]) -> Document {
        NativeFormat::import(data).unwrap()
    }

    pub fn find_matches(
        &self,
        query: &str,
        match_case: bool,
        use_regex: bool,
        use_wildcards: bool,
        format: Option<&tw_edit::FindFormatFilter>,
    ) -> Vec<tw_edit::FindMatch> {
        let Some(range) = tw_edit::document_body_range(&self.edit.document) else {
            return Vec::new();
        };
        tw_edit::find_matches(
            &self.edit.document,
            &range,
            query,
            match_case,
            use_regex,
            use_wildcards,
            format,
        )
        .unwrap_or_default()
    }
}

impl Default for SyncSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::NodeId;

    #[test]
    fn sync_session_typing_pipeline() {
        let mut session = SyncSession::new();
        let run_id = session.edit.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .runs[0]
            .id;

        session.apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "Hello".into(),
        });

        let bytes = session.display_list_bytes();
        assert!(!bytes.is_empty());
    }
}
