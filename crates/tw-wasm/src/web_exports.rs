//! Extended WASM bindings for browser hosts (Flutter web).

use std::time::Duration;
use tw_core::{BridgeEvent, DetectedFormat, WaitOutcome};
use tw_edit::command_from_json;
use tw_layout::FontFaceSpec;

use crate::{OpenError, WasmSession};

const TW_EVENT_DISPLAY_LIST_READY: u32 = 1;
const TW_EVENT_DOCUMENT_OPENED: u32 = 2;
const TW_EVENT_DOCUMENT_SAVED: u32 = 3;
const TW_EVENT_SPELL_CHECK_RESULT: u32 = 4;
const TW_EVENT_ERROR: u32 = 5;

fn bridge_event_type(event: &BridgeEvent) -> u32 {
    match event {
        BridgeEvent::DisplayListReady { .. } => TW_EVENT_DISPLAY_LIST_READY,
        BridgeEvent::DocumentOpened { .. } => TW_EVENT_DOCUMENT_OPENED,
        BridgeEvent::DocumentSaved { .. } => TW_EVENT_DOCUMENT_SAVED,
        BridgeEvent::SpellCheckResult { .. } => TW_EVENT_SPELL_CHECK_RESULT,
        BridgeEvent::Error { .. } => TW_EVENT_ERROR,
    }
}

fn parse_run_id(run_id: Option<&str>) -> Option<tw_model::NodeId> {
    run_id
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .map(tw_model::NodeId::from_uuid)
}

fn format_from_extension(ext: &str) -> DetectedFormat {
    tw_core::format_from_extension(ext).unwrap_or(DetectedFormat::Unknown)
}

fn wait_for_bytes(session: &tw_core::Session, request_id: u64) -> Result<Vec<u8>, String> {
    match session.wait_for_response(request_id, Duration::from_secs(30)) {
        WaitOutcome::Matched(event) => match event {
            BridgeEvent::DocumentSaved { data, .. } => Ok(data),
            BridgeEvent::SpellCheckResult { misspellings, .. } => {
                Ok(misspellings.join("\n").into_bytes())
            }
            BridgeEvent::Error { message, .. } => Err(message.clone()),
            _ => Err("unexpected response".into()),
        },
        WaitOutcome::Timeout => Err("timed out".into()),
    }
}

fn wait_for_edit(session: &tw_core::Session, request_id: u64) -> Result<(), String> {
    match session.wait_for_response(request_id, Duration::from_secs(30)) {
        WaitOutcome::Matched(event) => match event {
            BridgeEvent::Error { message, .. } => Err(message.clone()),
            _ => Ok(()),
        },
        WaitOutcome::Timeout => Err("timed out".into()),
    }
}

impl WasmSession {
    pub fn open_bytes_with_path_and_wait(
        &self,
        data: Vec<u8>,
        path_hint: Option<String>,
    ) -> Result<(), OpenError> {
        let session = self.session().expect("WasmSession not initialized");
        let request_id = session
            .open_bytes_with_path(data, path_hint)
            .ok_or(OpenError::EngineShutDown)?;
        match session.wait_for_response(request_id, Duration::from_secs(30)) {
            WaitOutcome::Matched(_) => Ok(()),
            WaitOutcome::Timeout => Err(OpenError::TimedOut),
        }
    }

    pub fn save_and_wait(&self) -> Result<Vec<u8>, String> {
        let session = self.session().expect("WasmSession not initialized");
        let request_id = session.save().ok_or_else(|| "engine shut down".to_string())?;
        wait_for_bytes(session, request_id)
    }

    pub fn save_as_and_wait(&self, extension: &str) -> Result<Vec<u8>, String> {
        let session = self.session().expect("WasmSession not initialized");
        let request_id = session
            .save_as(format_from_extension(extension))
            .ok_or_else(|| "engine shut down".to_string())?;
        wait_for_bytes(session, request_id)
    }

    pub fn export_pdf_and_wait(&self) -> Result<Vec<u8>, String> {
        let session = self.session().expect("WasmSession not initialized");
        let request_id = session
            .export_pdf()
            .ok_or_else(|| "engine shut down".to_string())?;
        wait_for_bytes(session, request_id)
    }

    pub fn spell_check_and_wait(&self) -> Result<Vec<u8>, String> {
        let session = self.session().expect("WasmSession not initialized");
        let request_id = session
            .spell_check()
            .ok_or_else(|| "engine shut down".to_string())?;
        wait_for_bytes(session, request_id)
    }

    pub fn enqueue_edit(&self, request_id: Option<u64>) -> Result<u64, String> {
        request_id.ok_or_else(|| "engine shut down".to_string())
    }

    pub fn wait_edit(&self, request_id: u64) -> Result<(), String> {
        let session = self.session().expect("WasmSession not initialized");
        wait_for_edit(session, request_id)
    }

    pub fn paste_html_enqueue(
        &self,
        run_id: &str,
        offset: u32,
        html: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = parse_run_id(Some(run_id)).ok_or_else(|| "invalid run id".to_string())?;
        self.enqueue_edit(
            session.paste_html_at(id, offset as usize, html.as_bytes().to_vec()),
        )
    }

    pub fn paste_docx_enqueue(
        &self,
        run_id: &str,
        offset: u32,
        bytes: &[u8],
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = parse_run_id(Some(run_id)).ok_or_else(|| "invalid run id".to_string())?;
        self.enqueue_edit(session.paste_docx_at(id, offset as usize, bytes.to_vec()))
    }

    pub fn set_current_page_enqueue(&self, page: u32) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.set_current_page(page))
    }

    pub fn undo_enqueue(&self) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.undo())
    }

    pub fn redo_enqueue(&self) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.redo())
    }

    pub fn apply_heading1_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.apply_heading1_at(caret))
    }

    pub fn apply_normal_style_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.apply_normal_style_at(caret))
    }

    pub fn apply_bullet_list_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.apply_bullet_list_at(caret))
    }

    pub fn apply_numbered_list_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.apply_numbered_list_at(caret))
    }

    pub fn insert_page_break_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.insert_page_break_at(caret))
    }

    pub fn insert_table_enqueue(&self, rows: u32, cols: u32) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.insert_table(rows, cols))
    }

    pub fn insert_image_enqueue(&self, width: f32, height: f32) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.insert_image(width, height))
    }

    pub fn set_track_changes_enqueue(&self, enabled: bool) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.set_track_changes(enabled))
    }

    pub fn accept_all_revisions_enqueue(&self) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.accept_all_revisions())
    }

    pub fn reject_all_revisions_enqueue(&self) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.reject_all_revisions())
    }

    pub fn clear_format_enqueue(
        &self,
        start_run: &str,
        start_offset: u32,
        end_run: &str,
        end_offset: u32,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let start_id = parse_run_id(Some(start_run)).ok_or_else(|| "invalid run id".to_string())?;
        let end_id = parse_run_id(Some(end_run)).ok_or_else(|| "invalid run id".to_string())?;
        let command = tw_edit::Command::ClearCharFormatFields {
            range: tw_edit::DocRange {
                start: tw_edit::DocPosition {
                    run_id: start_id,
                    char_offset: start_offset as usize,
                },
                end: tw_edit::DocPosition {
                    run_id: end_id,
                    char_offset: end_offset as usize,
                },
            },
            clear_color: true,
            clear_highlight: true,
        };
        self.enqueue_edit(session.apply(command))
    }

    pub fn new_document_and_wait(&self) -> Result<(), OpenError> {
        let session = self.session().expect("WasmSession not initialized");
        let request_id = session.new_document().ok_or(OpenError::EngineShutDown)?;
        match session.wait_for_response(request_id, Duration::from_secs(30)) {
            WaitOutcome::Matched(_) => Ok(()),
            WaitOutcome::Timeout => Err(OpenError::TimedOut),
        }
    }

    pub fn dispatch_command(&self, bytes: Vec<u8>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let command = command_from_json(&bytes).map_err(|e| e.to_string())?;
        session
            .apply(command)
            .ok_or_else(|| "engine shut down".to_string())
    }

    pub fn poll_event_json(&self) -> Option<String> {
        let session = self.session().expect("WasmSession not initialized");
        session.poll_event().map(|event| {
            serde_json::json!({
                "event_type": bridge_event_type(&event),
                "request_id": event.request_id(),
            })
            .to_string()
        })
    }

    pub fn display_list(&self) -> (Vec<u8>, u64, f32, f32, u32) {
        let session = self.session().expect("WasmSession not initialized");
        let snap = session.get_display_list_bytes();
        (
            snap.bytes.as_ref().clone(),
            snap.version,
            snap.page_width,
            snap.page_height,
            snap.page_count,
        )
    }

    pub fn page_display_list(&self, page: u32) -> Option<(Vec<u8>, u64, f32, f32)> {
        let session = self.session().expect("WasmSession not initialized");
        session.page_display_list(page).map(|snap| {
            (
                snap.bytes.as_ref().clone(),
                snap.version,
                snap.page_width,
                snap.page_height,
            )
        })
    }

    pub fn atlas(&self) -> (u64, u32, u32, Vec<u8>) {
        let session = self.session().expect("WasmSession not initialized");
        let (gen, w, h, bytes) = session.atlas_resource();
        (gen, w, h, bytes.as_ref().clone())
    }

    pub fn atlas_generation(&self) -> u64 {
        self.session()
            .map(|s| s.atlas_generation())
            .unwrap_or(0)
    }

    pub fn document_properties_json(&self) -> String {
        self.session()
            .map(|s| s.get_display_list_bytes().document_properties_json.clone())
            .unwrap_or_else(|| "{}".to_string())
    }

    pub fn is_read_only(&self) -> bool {
        self.session()
            .map(|s| s.get_display_list_bytes().read_only)
            .unwrap_or(false)
    }

    pub fn is_page_stale(&self, page: u32) -> bool {
        self.session()
            .map(|s| s.is_page_stale(page))
            .unwrap_or(false)
    }

    pub fn hit_test(&self, page: u32, x: f32, y: f32) -> Option<(String, u32)> {
        let session = self.session().expect("WasmSession not initialized");
        if session.is_page_stale(page) {
            return None;
        }
        session
            .hit_test(page, x, y)
            .map(|r| (r.run_id.as_uuid().to_string(), r.char_offset as u32))
    }

    pub fn document_tail_hit(&self, page: u32) -> Option<(String, u32)> {
        let session = self.session().expect("WasmSession not initialized");
        session
            .document_tail_hit(page)
            .map(|r| (r.run_id.as_uuid().to_string(), r.char_offset as u32))
    }

    pub fn caret_geometry(&self, page: u32, x: f32, y: f32) -> Option<(f32, f32, f32)> {
        let session = self.session().expect("WasmSession not initialized");
        session.caret_geometry(page, x, y)
    }

    pub fn caret_at(&self, page: u32, run_id: &str, offset: u32) -> Option<(f32, f32, f32)> {
        let session = self.session().expect("WasmSession not initialized");
        let uuid = uuid::Uuid::parse_str(run_id).ok()?;
        let id = tw_model::NodeId::from_uuid(uuid);
        session.caret_at(page, id, offset as usize)
    }

    pub fn selection_rects(
        &self,
        page: u32,
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
    ) -> Vec<f32> {
        let session = self.session().expect("WasmSession not initialized");
        session.selection_rects(page, start_x, start_y, end_x, end_y)
    }

    pub fn caret_format_json(&self, run_id: &str) -> Option<String> {
        let session = self.session().expect("WasmSession not initialized");
        let uuid = uuid::Uuid::parse_str(run_id).ok()?;
        let id = tw_model::NodeId::from_uuid(uuid);
        session.caret_format_json(id)
    }

    pub fn text_in_range(
        &self,
        start_run: &str,
        start_offset: u32,
        end_run: &str,
        end_offset: u32,
    ) -> Option<String> {
        let session = self.session().expect("WasmSession not initialized");
        let start = uuid::Uuid::parse_str(start_run).ok()?;
        let end = uuid::Uuid::parse_str(end_run).ok()?;
        let start_id = tw_model::NodeId::from_uuid(start);
        let end_id = tw_model::NodeId::from_uuid(end);
        session.text_in_range(start_id, start_offset as usize, end_id, end_offset as usize)
    }
}

#[cfg(feature = "wasm-bindgen")]
pub mod bindgen_exports {
    use super::*;
    use tw_layout::WEIGHT_REGULAR;
    use wasm_bindgen::prelude::*;

    /// Browser engine handle (inline executor). Register fonts before opening documents.
    #[wasm_bindgen]
    pub struct TwEngine {
        session: WasmSession,
        last_request_id: u64,
        last_error: String,
    }

    #[wasm_bindgen]
    impl TwEngine {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                session: WasmSession::new(),
                last_request_id: 0,
                last_error: String::new(),
            }
        }

        fn record_error(&mut self, message: String) {
            self.last_error = message;
        }

        fn record_enqueue(&mut self, id: u64) {
            self.last_request_id = id;
            self.last_error.clear();
        }

        pub fn last_error(&self) -> String {
            self.last_error.clone()
        }

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

        pub fn new_document(&mut self) -> Result<(), JsValue> {
            self.session
                .new_document_and_wait()
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        pub fn open_document(&mut self, data: &[u8]) -> Result<(), JsValue> {
            self.session
                .open_bytes_and_wait(data.to_vec())
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        pub fn open_document_with_path(&mut self, data: &[u8], path: &str) -> Result<(), JsValue> {
            let hint = if path.is_empty() {
                None
            } else {
                Some(path.to_string())
            };
            self.session
                .open_bytes_with_path_and_wait(data.to_vec(), hint)
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        pub fn save_document(&mut self) -> Result<Vec<u8>, JsValue> {
            match self.session.save_and_wait() {
                Ok(bytes) => {
                    self.last_error.clear();
                    Ok(bytes)
                }
                Err(e) => {
                    self.record_error(e.clone());
                    Err(JsValue::from_str(&e))
                }
            }
        }

        pub fn save_document_as(&mut self, extension: &str) -> Result<Vec<u8>, JsValue> {
            match self.session.save_as_and_wait(extension) {
                Ok(bytes) => {
                    self.last_error.clear();
                    Ok(bytes)
                }
                Err(e) => {
                    self.record_error(e.clone());
                    Err(JsValue::from_str(&e))
                }
            }
        }

        pub fn export_pdf(&mut self) -> Result<Vec<u8>, JsValue> {
            match self.session.export_pdf_and_wait() {
                Ok(bytes) => {
                    self.last_error.clear();
                    Ok(bytes)
                }
                Err(e) => {
                    self.record_error(e.clone());
                    Err(JsValue::from_str(&e))
                }
            }
        }

        pub fn spell_check(&mut self) -> Result<String, JsValue> {
            match self.session.spell_check_and_wait() {
                Ok(bytes) => {
                    self.last_error.clear();
                    Ok(String::from_utf8_lossy(&bytes).into_owned())
                }
                Err(e) => {
                    self.record_error(e.clone());
                    Err(JsValue::from_str(&e))
                }
            }
        }

        pub fn dispatch(&mut self, data: &[u8]) -> Result<f64, JsValue> {
            let id = self
                .session
                .dispatch_command(data.to_vec())
                .map_err(|e| JsValue::from_str(&e))?;
            self.record_enqueue(id);
            Ok(id as f64)
        }

        fn enqueue_op(&mut self, result: Result<u64, String>) -> Result<f64, JsValue> {
            match result {
                Ok(id) => {
                    self.record_enqueue(id);
                    Ok(id as f64)
                }
                Err(e) => {
                    self.record_error(e.clone());
                    Err(JsValue::from_str(&e))
                }
            }
        }

        pub fn undo(&mut self) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.undo_enqueue())
        }

        pub fn redo(&mut self) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.redo_enqueue())
        }

        pub fn set_current_page(&mut self, page: u32) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.set_current_page_enqueue(page))
        }

        pub fn apply_heading1(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.apply_heading1_enqueue(caret))
        }

        pub fn apply_normal_style(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.apply_normal_style_enqueue(caret))
        }

        pub fn apply_bullet_list(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.apply_bullet_list_enqueue(caret))
        }

        pub fn apply_numbered_list(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.apply_numbered_list_enqueue(caret))
        }

        pub fn insert_page_break(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.insert_page_break_enqueue(caret))
        }

        pub fn insert_table(&mut self, rows: u32, cols: u32) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_table_enqueue(rows, cols))
        }

        pub fn insert_image(&mut self, width: f32, height: f32) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_image_enqueue(width, height))
        }

        pub fn paste_html(&mut self, run_id: &str, offset: u32, html: &str) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.paste_html_enqueue(run_id, offset, html))
        }

        pub fn paste_docx(&mut self, run_id: &str, offset: u32, data: &[u8]) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.paste_docx_enqueue(run_id, offset, data))
        }

        pub fn clear_format(
            &mut self,
            start_run: &str,
            start_offset: u32,
            end_run: &str,
            end_offset: u32,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(
                self.session
                    .clear_format_enqueue(start_run, start_offset, end_run, end_offset),
            )
        }

        pub fn set_track_changes(&mut self, enabled: bool) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.set_track_changes_enqueue(enabled))
        }

        pub fn accept_all_revisions(&mut self) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.accept_all_revisions_enqueue())
        }

        pub fn reject_all_revisions(&mut self) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.reject_all_revisions_enqueue())
        }

        pub fn pump(&mut self) -> u32 {
            self.session.pump() as u32
        }

        pub fn pop_event(&self) -> Option<String> {
            self.session.poll_event_json()
        }

        pub fn last_request_id(&self) -> f64 {
            self.last_request_id as f64
        }

        pub fn page_count(&self) -> u32 {
            self.session.page_count()
        }

        pub fn text(&self) -> String {
            self.session.document_text()
        }

        pub fn first_line_advance(&self) -> f32 {
            self.session.first_line_width(0)
        }

        pub fn display_list_bytes(&self) -> Vec<u8> {
            self.session.display_list().0
        }

        pub fn display_list_version(&self) -> f64 {
            self.session.display_list().1 as f64
        }

        pub fn display_list_page_width(&self) -> f32 {
            self.session.display_list().2
        }

        pub fn display_list_page_height(&self) -> f32 {
            self.session.display_list().3
        }

        pub fn display_list_page_count(&self) -> u32 {
            self.session.display_list().4
        }

        pub fn page_display_list_bytes(&self, page: u32) -> Option<Vec<u8>> {
            self.session.page_display_list(page).map(|v| v.0)
        }

        pub fn page_display_list_version(&self, page: u32) -> Option<f64> {
            self.session.page_display_list(page).map(|v| v.1 as f64)
        }

        pub fn page_display_list_page_width(&self, page: u32) -> Option<f32> {
            self.session.page_display_list(page).map(|v| v.2)
        }

        pub fn page_display_list_page_height(&self, page: u32) -> Option<f32> {
            self.session.page_display_list(page).map(|v| v.3)
        }

        pub fn atlas_generation(&self) -> f64 {
            self.session.atlas_generation() as f64
        }

        pub fn atlas_bytes(&self) -> Vec<u8> {
            self.session.atlas().3
        }

        pub fn atlas_width(&self) -> u32 {
            self.session.atlas().1
        }

        pub fn atlas_height(&self) -> u32 {
            self.session.atlas().2
        }

        pub fn document_properties_json(&self) -> String {
            self.session.document_properties_json()
        }

        pub fn is_read_only(&self) -> bool {
            self.session.is_read_only()
        }

        pub fn is_page_stale(&self, page: u32) -> bool {
            self.session.is_page_stale(page)
        }

        pub fn hit_test(&self, page: u32, x: f32, y: f32) -> Option<String> {
            self.session
                .hit_test(page, x, y)
                .map(|(run, offset)| {
                    serde_json::json!({ "run_id": run, "offset": offset }).to_string()
                })
        }

        pub fn document_tail_hit(&self, page: u32) -> Option<String> {
            self.session
                .document_tail_hit(page)
                .map(|(run, offset)| {
                    serde_json::json!({ "run_id": run, "offset": offset }).to_string()
                })
        }

        pub fn caret_geometry(&self, page: u32, x: f32, y: f32) -> Option<String> {
            self.session
                .caret_geometry(page, x, y)
                .map(|(cx, cy, h)| {
                    serde_json::json!({ "x": cx, "y": cy, "height": h }).to_string()
                })
        }

        pub fn caret_at(&self, page: u32, run_id: &str, offset: u32) -> Option<String> {
            self.session
                .caret_at(page, run_id, offset)
                .map(|(cx, cy, h)| {
                    serde_json::json!({ "x": cx, "y": cy, "height": h }).to_string()
                })
        }

        pub fn selection_rects(
            &self,
            page: u32,
            start_x: f32,
            start_y: f32,
            end_x: f32,
            end_y: f32,
        ) -> Vec<f32> {
            self.session.selection_rects(page, start_x, start_y, end_x, end_y)
        }

        pub fn caret_format_json(&self, run_id: &str) -> Option<String> {
            self.session.caret_format_json(run_id)
        }

        pub fn text_in_range(
            &self,
            start_run: &str,
            start_offset: u32,
            end_run: &str,
            end_offset: u32,
        ) -> Option<String> {
            self.session
                .text_in_range(start_run, start_offset, end_run, end_offset)
        }
    }
}
