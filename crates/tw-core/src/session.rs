use crate::import::DetectedFormat;
use crate::layout_cache::{new_shared_layout_cache, SharedLayoutCache};
use crate::snapshot::SnapshotBuffer;
use crate::worker::{BridgeCommand, BridgeEvent, QueuedCommand, WorkerHandle, STARTUP_REQUEST_ID};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};
use tw_edit::{
    bullet_list_command_for_caret, heading1_command_for_caret, insert_image_command,
    insert_page_break_command_for, insert_table_command, normal_style_command_for_caret,
    numbered_list_command_for_caret, Command, EditSession,
};
use tw_layout::LayoutEngine;
use tw_model::{Document, NodeId};
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
    worker: WorkerHandle,
    pending_events: Mutex<VecDeque<BridgeEvent>>,
    event_observer: StdMutex<Option<EventObserver>>,
    next_request_id: AtomicU64,
    /// External [`Session::pump_events`] calls, used only to diagnose a host that
    /// installed an observer and then never pumped.
    pump_calls: AtomicU64,
    unpumped_warning_emitted: AtomicBool,
    pub doc_id: DocId,
}

impl Session {
    pub fn new() -> Self {
        let snapshot = Arc::new(SnapshotBuffer::new());
        let layout_cache = new_shared_layout_cache();
        let worker = WorkerHandle::spawn(snapshot.clone(), layout_cache.clone());
        Self {
            snapshot,
            layout_cache,
            worker,
            pending_events: Mutex::new(VecDeque::new()),
            event_observer: StdMutex::new(None),
            next_request_id: AtomicU64::new(1),
            pump_calls: AtomicU64::new(0),
            unpumped_warning_emitted: AtomicBool::new(false),
            doc_id: 1,
        }
    }

    /// Forward every worker event to the host (or tests) before buffering it for
    /// correlated waits.
    ///
    /// **The observer only fires while this session drains its event channel, and
    /// nothing drains it spontaneously.** Enqueuing a command does not; the worker
    /// pushes into a channel and moves on. A host that installs an observer and
    /// then waits for events without calling [`Session::pump_events`] will observe
    /// no events at all, and every correlated wait will run to its full timeout.
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
        while let Ok(event) = self.worker.event_rx.try_recv() {
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
    pub fn pump_events(&self) -> usize {
        self.pump_calls.fetch_add(1, Ordering::Relaxed);
        self.drain_events()
    }

    /// A correlated wait that ran to its timeout while an observer was installed
    /// and nothing ever pumped is almost certainly the unpumped-host mistake, not a
    /// slow worker. Say so once, on stderr, rather than letting the host rediscover
    /// it as an unexplained multi-second hang.
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
        // Block until the worker accepts the command (bounded queue backpressure).
        // Returns None only if the worker channel is disconnected (shutdown).
        self.worker.cmd_tx.send(queued).ok().map(|_| request_id)
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
        self.send_command(BridgeCommand::OpenDocument { data, path_hint })
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

    pub fn set_track_changes(&self, enabled: bool) -> Option<u64> {
        self.send_command(BridgeCommand::ToggleTrackChanges { enabled })
    }

    pub fn accept_all_revisions(&self) -> Option<u64> {
        self.apply(Command::AcceptAllRevisions)
    }

    pub fn reject_all_revisions(&self) -> Option<u64> {
        self.apply(Command::RejectAllRevisions)
    }

    /// Returns the next buffered or freshly received event (any request id).
    pub fn poll_event(&self) -> Option<BridgeEvent> {
        let _ = self.drain_events();
        self.pending_events.lock().pop_front()
    }

    /// Wait until an event with `request_id` arrives, buffering unrelated events.
    pub fn wait_for_response(&self, request_id: u64, timeout: Duration) -> WaitOutcome {
        let deadline = Instant::now() + timeout;
        let started = Instant::now();
        loop {
            let _ = self.drain_events();
            {
                let mut pending = self.pending_events.lock();
                if let Some(index) = pending
                    .iter()
                    .position(|event| event.request_id() == request_id)
                {
                    return WaitOutcome::Matched(pending.remove(index).expect("event index"));
                }
            }
            if Instant::now() >= deadline {
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
        self.apply_from_document(|doc| normal_style_command_for_caret(doc, caret_run_id))
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

    pub fn insert_table(&self, rows: u32, cols: u32) -> Option<u64> {
        self.apply_from_document(|doc| insert_table_command(doc, rows, cols))
    }

    pub fn insert_image(&self, width: f32, height: f32) -> Option<u64> {
        self.apply_from_document(|doc| insert_image_command(doc, width, height))
    }

    pub fn insert_page_break(&self) -> Option<u64> {
        self.insert_page_break_at(None)
    }

    pub fn insert_page_break_at(&self, caret_run_id: Option<tw_model::NodeId>) -> Option<u64> {
        self.apply_from_document(|doc| insert_page_break_command_for(doc, caret_run_id))
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
        let (char_format, para_format, style_name) =
            self.layout_cache.read().format_at(run_id)?;
        #[derive(serde::Serialize)]
        struct CaretFormatResponse {
            char_format: tw_model::CharFormat,
            para_format: tw_model::ParaFormat,
            style_name: Option<String>,
        }
        serde_json::to_string(&CaretFormatResponse {
            char_format,
            para_format,
            style_name,
        })
        .ok()
    }

    pub fn layout_page_count(&self) -> u32 {
        self.layout_cache.read().page_count()
    }

    /// True while `page` still carries pre-edit geometry awaiting background reflow.
    pub fn is_page_stale(&self, page: u32) -> bool {
        self.layout_cache.read().is_page_stale(page)
    }

    pub fn wait_for_event(&self, timeout_ms: u64) -> Option<BridgeEvent> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        while Instant::now() < deadline {
            if let Some(event) = self.poll_event() {
                return Some(event);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        None
    }

    /// Wait for the worker startup `DocumentOpened` event (`STARTUP_REQUEST_ID`).
    pub fn wait_for_startup(&self, timeout: Duration) -> WaitOutcome {
        self.wait_for_response(STARTUP_REQUEST_ID, timeout)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.worker.shutdown();
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
