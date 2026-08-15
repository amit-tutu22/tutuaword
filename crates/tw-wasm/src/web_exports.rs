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
        // Same wire type as spell check — payload is newline-joined issues.
        BridgeEvent::GrammarCheckResult { .. } => TW_EVENT_SPELL_CHECK_RESULT,
        BridgeEvent::Error { .. } => TW_EVENT_ERROR,
    }
}

fn parse_run_id(run_id: Option<&str>) -> Option<tw_model::NodeId> {
    run_id
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .map(tw_model::NodeId::from_uuid)
}

fn text_wrap_from_u8(value: u8) -> Option<tw_model::TextWrap> {
    match value {
        0 => Some(tw_model::TextWrap::Inline),
        1 => Some(tw_model::TextWrap::Square),
        2 => Some(tw_model::TextWrap::TopBottom),
        3 => Some(tw_model::TextWrap::Behind),
        4 => Some(tw_model::TextWrap::InFront),
        _ => None,
    }
}

fn anchor_origin_from_u8(value: u8) -> Option<tw_model::AnchorOrigin> {
    match value {
        0 => Some(tw_model::AnchorOrigin::Column),
        1 => Some(tw_model::AnchorOrigin::Page),
        2 => Some(tw_model::AnchorOrigin::Margin),
        3 => Some(tw_model::AnchorOrigin::Paragraph),
        _ => None,
    }
}

fn parse_field_type_name(name: &str) -> Result<tw_model::FieldType, String> {
    match name.to_ascii_lowercase().as_str() {
        "page" => Ok(tw_model::FieldType::Page),
        "numpages" => Ok(tw_model::FieldType::NumPages),
        "date" => Ok(tw_model::FieldType::Date),
        "time" => Ok(tw_model::FieldType::Time),
        "tablesum" | "sum" => Ok(tw_model::FieldType::TableSumAbove),
        "formtext" | "form_text" => Ok(tw_model::FieldType::FormText),
        "formcheckbox" | "form_checkbox" | "checkbox" => Ok(tw_model::FieldType::FormCheckbox),
        "mergefield" | "merge_field" | "merge" => Ok(tw_model::FieldType::MergeField),
        _ => Err(format!("unknown field type: {name}")),
    }
}

fn parse_form_field_kind(kind: &str) -> Result<tw_model::FormFieldKind, String> {
    match kind.to_ascii_lowercase().as_str() {
        "text" | "formtext" | "plain" | "plaintext" => Ok(tw_model::FormFieldKind::PlainText),
        "checkbox" | "formcheckbox" | "check" => Ok(tw_model::FormFieldKind::Checkbox),
        _ => Err(format!("unknown form field kind: {kind}")),
    }
}

fn format_from_extension(ext: &str) -> DetectedFormat {
    tw_core::format_from_extension(ext).unwrap_or(DetectedFormat::Unknown)
}

fn encode_spell_issues(issues: &[(String, usize, usize, Vec<String>)]) -> Vec<u8> {
    #[derive(serde::Serialize)]
    struct Issue<'a> {
        word: &'a str,
        start: usize,
        end: usize,
        suggestions: &'a [String],
    }
    let payload: Vec<Issue<'_>> = issues
        .iter()
        .map(|(word, start, end, suggestions)| Issue {
            word,
            start: *start,
            end: *end,
            suggestions,
        })
        .collect();
    serde_json::to_vec(&payload).unwrap_or_default()
}

fn wait_for_bytes(session: &tw_core::Session, request_id: u64) -> Result<Vec<u8>, String> {
    match session.wait_for_response(request_id, Duration::from_secs(30)) {
        WaitOutcome::Matched(event) => match event {
            BridgeEvent::DocumentSaved { data, .. } => Ok(data),
            BridgeEvent::SpellCheckResult {
                misspellings,
                spell_issues,
                ..
            } => {
                if spell_issues.is_empty() {
                    Ok(misspellings.join("\n").into_bytes())
                } else {
                    Ok(encode_spell_issues(&spell_issues))
                }
            }
            BridgeEvent::GrammarCheckResult { issues, .. } => {
                Ok(issues.join("\n").into_bytes())
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
        self.open_bytes_with_path_and_password_and_wait(data, path_hint, None)
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

    pub fn export_pdf_for_print_and_wait(
        &self,
        layout: tw_pdf::PrintLayoutOptions,
        selection: Option<tw_edit::DocRange>,
    ) -> Result<Vec<u8>, String> {
        let session = self.session().expect("WasmSession not initialized");
        let request_id = session
            .export_pdf_for_print(layout, selection)
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

    pub fn grammar_check_and_wait(&self) -> Result<Vec<u8>, String> {
        let session = self.session().expect("WasmSession not initialized");
        let request_id = session
            .grammar_check()
            .ok_or_else(|| "engine shut down".to_string())?;
        wait_for_bytes(session, request_id)
    }

    pub fn compare_document_text(&self, other: &str) -> String {
        let session = self.session().expect("WasmSession not initialized");
        let summary = session.compare_with_text(other);
        format!(
            "insertions:{} deletions:{}",
            summary.insertion_count, summary.deletion_count
        )
    }

    pub fn find_matches_json(
        &self,
        query: &str,
        match_case: bool,
        use_regex: bool,
        use_wildcards: bool,
        format_json: &str,
    ) -> String {
        let session = self.session().expect("WasmSession not initialized");
        let format = if format_json.is_empty() {
            None
        } else {
            serde_json::from_str::<tw_edit::FindFormatFilter>(format_json).ok()
        };
        let matches = session.find_matches(
            query,
            match_case,
            use_regex,
            use_wildcards,
            format.as_ref(),
        );
        #[derive(serde::Serialize)]
        struct MatchOut {
            start_run_id: String,
            start: usize,
            end_run_id: String,
            end: usize,
        }
        let payload: Vec<MatchOut> = matches
            .iter()
            .map(|m| MatchOut {
                start_run_id: m.start.run_id.to_string(),
                start: m.start.char_offset,
                end_run_id: m.end.run_id.to_string(),
                end: m.end.char_offset,
            })
            .collect();
        serde_json::to_string(&payload).unwrap_or_else(|_| "[]".to_string())
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

    pub fn apply_paragraph_style_enqueue(
        &self,
        caret_run_id: Option<&str>,
        style_name: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.apply_paragraph_style_at(caret, style_name))
    }

    pub fn apply_document_theme_enqueue(&self, theme_name: &str) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.apply_document_theme(theme_name))
    }

    pub fn apply_section_format_enqueue(
        &self,
        format_json: &str,
        caret_run_id: Option<&str>,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let format: tw_model::SectionFormat = serde_json::from_str(format_json)
            .map_err(|e| format!("invalid section format json: {e}"))?;
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.apply_section_format_at(caret, format))
    }

    pub fn section_format_json(&self, caret_run_id: Option<&str>) -> Result<String, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        session
            .section_format_json_at(caret)
            .ok_or_else(|| "section format unavailable".into())
    }

    pub fn insert_section_break_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.insert_section_break_at(caret))
    }

    pub fn ensure_header_footer_enqueue(
        &self,
        caret_run_id: Option<&str>,
        is_header: bool,
        page_index: Option<u32>,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.ensure_header_footer_at(caret, is_header, page_index))
    }

    pub fn set_even_and_odd_headers_enqueue(&self, enabled: bool) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.set_even_and_odd_headers(enabled))
    }

    pub fn header_footer_linked(
        &self,
        caret_run_id: Option<&str>,
        is_header: bool,
        page_index: Option<u32>,
    ) -> bool {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        session.header_footer_linked_at(caret, is_header, page_index)
    }

    pub fn set_header_footer_link_enqueue(
        &self,
        caret_run_id: Option<&str>,
        is_header: bool,
        page_index: Option<u32>,
        linked: bool,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.set_header_footer_link_at(
            caret,
            is_header,
            page_index,
            linked,
        ))
    }

    pub fn even_and_odd_headers_enabled(&self) -> bool {
        self.session()
            .expect("WasmSession not initialized")
            .document()
            .settings
            .even_and_odd_headers
    }

    pub fn header_footer_seed_run(
        &self,
        caret_run_id: Option<&str>,
        is_header: bool,
        page_index: Option<u32>,
    ) -> Result<String, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        session
            .header_footer_seed_run_at(caret, is_header, page_index)
            .map(|id| id.to_string())
            .ok_or_else(|| "header/footer seed run unavailable".into())
    }

    pub fn insert_field_enqueue(
        &self,
        run_id: &str,
        offset: usize,
        field_type: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let run = parse_run_id(Some(run_id)).ok_or_else(|| "run id required".to_string())?;
        let field = parse_field_type_name(field_type)?;
        self.enqueue_edit(session.insert_field_at(run, offset, field, None))
    }

    pub fn insert_form_field_enqueue(
        &self,
        run_id: &str,
        offset: usize,
        kind: &str,
        name: &str,
        initial_value: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let run = parse_run_id(Some(run_id)).ok_or_else(|| "run id required".to_string())?;
        let kind = parse_form_field_kind(kind)?;
        let name = if name.trim().is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        let initial_value = if initial_value.is_empty() {
            None
        } else {
            Some(initial_value.to_string())
        };
        self.enqueue_edit(session.insert_form_field_at(run, offset, kind, name, initial_value))
    }

    pub fn set_form_field_value_enqueue(
        &self,
        run_id: &str,
        value: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let run = parse_run_id(Some(run_id)).ok_or_else(|| "run id required".to_string())?;
        self.enqueue_edit(session.set_form_field_value_at(run, value))
    }

    pub fn insert_merge_field_enqueue(
        &self,
        run_id: &str,
        offset: usize,
        name: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let run = parse_run_id(Some(run_id)).ok_or_else(|| "run id required".to_string())?;
        if name.trim().is_empty() {
            return Err("merge field name required".into());
        }
        self.enqueue_edit(session.insert_merge_field_at(run, offset, name))
    }

    pub fn apply_mail_merge_row_enqueue(&self, values_json: &str) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let map: std::collections::BTreeMap<String, String> = serde_json::from_str(values_json)
            .map_err(|e| format!("invalid mail merge JSON: {e}"))?;
        self.enqueue_edit(session.apply_mail_merge_row_at(map))
    }

    pub fn insert_footnote_enqueue(&self, run_id: &str, offset: usize) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let run = parse_run_id(Some(run_id)).ok_or_else(|| "run id required".to_string())?;
        self.enqueue_edit(session.insert_footnote_at(run, offset))
    }

    pub fn insert_comment_enqueue(
        &self,
        run_id: &str,
        offset: usize,
        body_text: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let run = parse_run_id(Some(run_id)).ok_or_else(|| "run id required".to_string())?;
        self.enqueue_edit(session.insert_comment_at(run, offset, body_text))
    }

    pub fn insert_table_of_contents_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.insert_table_of_contents_at(caret))
    }

    pub fn add_bibliography_source_enqueue(
        &self,
        key: &str,
        author: &str,
        title: &str,
        year: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.add_bibliography_source(tw_model::BibliographySource::new(
            key, author, title, year,
        )))
    }

    pub fn insert_citation_enqueue(
        &self,
        run_id: &str,
        offset: usize,
        source_key: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let run = parse_run_id(Some(run_id)).ok_or_else(|| "run id required".to_string())?;
        self.enqueue_edit(session.insert_citation_at(run, offset, source_key))
    }

    pub fn insert_bibliography_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.insert_bibliography_at(caret))
    }

    pub fn insert_bookmark_enqueue(
        &self,
        run_id: &str,
        offset: usize,
        name: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let run = parse_run_id(Some(run_id)).ok_or_else(|| "run id required".to_string())?;
        self.enqueue_edit(session.insert_bookmark_at(run, offset, name))
    }

    pub fn insert_hyperlink_enqueue(
        &self,
        run_id: &str,
        offset: usize,
        url: &str,
        text: &str,
        tooltip: Option<&str>,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let run = parse_run_id(Some(run_id)).ok_or_else(|| "run id required".to_string())?;
        self.enqueue_edit(session.insert_hyperlink_at(run, offset, url, text, tooltip))
    }

    pub fn insert_cross_reference_enqueue(
        &self,
        run_id: &str,
        offset: usize,
        bookmark_name: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let run = parse_run_id(Some(run_id)).ok_or_else(|| "run id required".to_string())?;
        self.enqueue_edit(session.insert_cross_reference_at(run, offset, bookmark_name))
    }

    pub fn insert_index_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.insert_index_at(caret))
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

    pub fn insert_table_enqueue(
        &self,
        rows: u32,
        cols: u32,
        caret_run_id: Option<&str>,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.insert_table_at(caret, rows, cols))
    }

    pub fn delete_table_row_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.delete_table_row_at(caret))
    }

    pub fn delete_table_column_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.delete_table_column_at(caret))
    }

    pub fn merge_table_cells_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.merge_table_cells_at(caret))
    }

    pub fn split_table_cell_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.split_table_cell_at(caret))
    }

    pub fn set_table_border_enqueue(
        &self,
        caret_run_id: Option<&str>,
        width: f32,
        color_r: u8,
        color_g: u8,
        color_b: u8,
        color_a: u8,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        let border = if width > 0.0 {
            Some(tw_model::BorderSpec {
                width,
                color: tw_model::Color {
                    r: color_r,
                    g: color_g,
                    b: color_b,
                    a: color_a,
                },
            })
        } else {
            None
        };
        self.enqueue_edit(session.set_table_border_at(caret, border))
    }

    pub fn set_table_cell_shading_enqueue(
        &self,
        caret_run_id: Option<&str>,
        color_r: i32,
        color_g: u8,
        color_b: u8,
        color_a: u8,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        let background = if color_r < 0 {
            None
        } else {
            Some(tw_model::Color {
                r: color_r as u8,
                g: color_g,
                b: color_b,
                a: color_a,
            })
        };
        self.enqueue_edit(session.set_table_cell_shading_at(caret, background))
    }

    pub fn resize_table_column_enqueue(
        &self,
        caret_run_id: Option<&str>,
        width: f32,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.resize_table_column_at(caret, width))
    }

    pub fn autofit_table_enqueue(&self, caret_run_id: Option<&str>) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.autofit_table_at(caret))
    }

    pub fn sort_table_rows_enqueue(
        &self,
        caret_run_id: Option<&str>,
        ascending: bool,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.sort_table_rows_at(caret, ascending))
    }

    pub fn insert_nested_table_enqueue(
        &self,
        caret_run_id: Option<&str>,
        rows: u32,
        cols: u32,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.insert_nested_table_at(caret, rows, cols))
    }

    pub fn insert_table_sum_field_enqueue(
        &self,
        caret_run_id: Option<&str>,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(caret_run_id);
        self.enqueue_edit(session.insert_table_sum_field_at(caret))
    }

    pub fn insert_image_enqueue(&self, width: f32, height: f32) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.insert_image(width, height))
    }

    pub fn insert_shape_enqueue(&self, shape_type: i32) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let kind = match shape_type {
            0 => tw_model::ShapeKind::Rectangle,
            1 => tw_model::ShapeKind::Line,
            2 => tw_model::ShapeKind::Ellipse,
            3 => tw_model::ShapeKind::TextBox,
            4 => tw_model::ShapeKind::WordArt,
            _ => tw_model::ShapeKind::Other,
        };
        self.enqueue_edit(session.insert_shape(kind))
    }

    pub fn insert_text_box_enqueue(&self) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.insert_text_box())
    }

    pub fn insert_word_art_enqueue(&self, text: String) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.insert_word_art(text))
    }

    pub fn insert_diagram_enqueue(&self, diagram_type: i32) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let kind = tw_model::DiagramKind::from_i32(diagram_type);
        self.enqueue_edit(session.insert_diagram_with_kind(kind))
    }

    pub fn insert_chart_enqueue(&self, chart_type: i32) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let kind = tw_model::ChartKind::from_i32(chart_type);
        self.enqueue_edit(session.insert_chart_with_kind(kind))
    }

    pub fn chart_data_json(&self, shape_id: &str) -> Result<String, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = uuid::Uuid::parse_str(shape_id)
            .map(tw_model::NodeId::from_uuid)
            .map_err(|e| e.to_string())?;
        session
            .chart_data_json(id)
            .ok_or_else(|| "chart data not found".to_string())
    }

    pub fn latest_chart_id(&self) -> Result<String, String> {
        let session = self.session().expect("WasmSession not initialized");
        session
            .latest_chart_id()
            .map(|id| id.to_string())
            .ok_or_else(|| "no chart in document".to_string())
    }

    pub fn set_chart_data_enqueue(&self, shape_id: &str, chart_json: &str) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = uuid::Uuid::parse_str(shape_id)
            .map(tw_model::NodeId::from_uuid)
            .map_err(|e| e.to_string())?;
        let chart_data: tw_model::ChartData =
            serde_json::from_str(chart_json).map_err(|e| e.to_string())?;
        self.enqueue_edit(session.set_chart_data(id, Some(chart_data)))
    }

    pub fn insert_office_math_enqueue(
        &self,
        run_id: &str,
        offset: usize,
        xml: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = uuid::Uuid::parse_str(run_id)
            .map(tw_model::NodeId::from_uuid)
            .map_err(|e| e.to_string())?;
        self.enqueue_edit(session.insert_office_math_at(id, offset, xml.to_string()))
    }

    pub fn insert_office_math_display_enqueue(
        &self,
        caret_run_id: Option<&str>,
        xml: &str,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = caret_run_id
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .map(tw_model::NodeId::from_uuid);
        self.enqueue_edit(session.insert_office_math_display(caret, xml.to_string()))
    }

    pub fn set_office_math_enqueue(&self, run_id: &str, xml: &str) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = uuid::Uuid::parse_str(run_id)
            .map(tw_model::NodeId::from_uuid)
            .map_err(|e| e.to_string())?;
        self.enqueue_edit(session.set_office_math(id, xml.to_string()))
    }

    pub fn office_math_xml(&self, run_id: &str) -> Result<String, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = uuid::Uuid::parse_str(run_id)
            .map(tw_model::NodeId::from_uuid)
            .map_err(|e| e.to_string())?;
        session
            .office_math_xml(id)
            .ok_or_else(|| "not an equation run".to_string())
    }

    pub fn latest_office_math_run_id(&self) -> Result<String, String> {
        let session = self.session().expect("WasmSession not initialized");
        session
            .latest_office_math_run_id()
            .map(|id| id.to_string())
            .ok_or_else(|| "no equation in document".to_string())
    }

    pub fn delete_block_enqueue(&self, block_id: &str) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = uuid::Uuid::parse_str(block_id)
            .map(tw_model::NodeId::from_uuid)
            .map_err(|e| e.to_string())?;
        self.enqueue_edit(session.delete_block(id))
    }

    pub fn insert_image_bytes_enqueue(
        &self,
        bytes: Vec<u8>,
        mime_type: String,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.insert_image_bytes(bytes, mime_type))
    }

    pub fn set_image_size_enqueue(
        &self,
        image_id: &str,
        width: f32,
        height: f32,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = parse_run_id(Some(image_id)).ok_or_else(|| "invalid image id".to_string())?;
        self.enqueue_edit(session.set_image_size(id, width, height))
    }

    pub fn replace_image_bytes_enqueue(
        &self,
        image_id: &str,
        bytes: Vec<u8>,
        mime_type: String,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = parse_run_id(Some(image_id)).ok_or_else(|| "invalid image id".to_string())?;
        self.enqueue_edit(session.replace_image_bytes(id, bytes, mime_type))
    }

    pub fn set_image_wrap_enqueue(&self, image_id: &str, wrap: u8) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = parse_run_id(Some(image_id)).ok_or_else(|| "invalid image id".to_string())?;
        let wrap = text_wrap_from_u8(wrap).ok_or_else(|| "invalid wrap mode".to_string())?;
        self.enqueue_edit(session.set_image_wrap(id, wrap))
    }

    pub fn set_image_anchor_enqueue(
        &self,
        image_id: &str,
        x: f32,
        y: f32,
        origin_x: u8,
        origin_y: u8,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = parse_run_id(Some(image_id)).ok_or_else(|| "invalid image id".to_string())?;
        let origin_x =
            anchor_origin_from_u8(origin_x).ok_or_else(|| "invalid anchor origin_x".to_string())?;
        let origin_y =
            anchor_origin_from_u8(origin_y).ok_or_else(|| "invalid anchor origin_y".to_string())?;
        self.enqueue_edit(session.set_image_anchor(
            id,
            tw_model::ImageAnchor {
                x,
                y,
                origin_x,
                origin_y,
            },
        ))
    }

    pub fn set_image_transform_enqueue(
        &self,
        image_id: &str,
        rotation_deg: f32,
        crop_left: f32,
        crop_top: f32,
        crop_right: f32,
        crop_bottom: f32,
        opacity: f32,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = parse_run_id(Some(image_id)).ok_or_else(|| "invalid image id".to_string())?;
        self.enqueue_edit(session.set_image_transform(
            id,
            tw_model::ImageTransform {
                rotation_deg,
                crop_left,
                crop_top,
                crop_right,
                crop_bottom,
                opacity,
            },
        ))
    }

    pub fn insert_image_caption_enqueue(&self, image_id: &str) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = parse_run_id(Some(image_id)).ok_or_else(|| "invalid image id".to_string())?;
        self.enqueue_edit(session.insert_image_caption(id))
    }

    pub fn set_image_alt_text_enqueue(
        &self,
        image_id: &str,
        alt_text: Option<String>,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = parse_run_id(Some(image_id)).ok_or_else(|| "invalid image id".to_string())?;
        self.enqueue_edit(session.set_image_alt_text(id, alt_text))
    }

    pub fn image_alt_text(&self, image_id: &str) -> Result<String, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = parse_run_id(Some(image_id)).ok_or_else(|| "invalid image id".to_string())?;
        session
            .image_alt_text(id)
            .ok_or_else(|| "image not found".to_string())
    }

    pub fn compress_image_enqueue(&self, image_id: &str, quality: u8) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let id = parse_run_id(Some(image_id)).ok_or_else(|| "invalid image id".to_string())?;
        self.enqueue_edit(session.compress_image(id, quality))
    }

    pub fn set_track_changes_enqueue(&self, enabled: bool) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.set_track_changes(enabled))
    }

    pub fn set_read_only_enqueue(&self, enabled: bool) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.set_read_only(enabled))
    }

    pub fn set_encryption_password_enqueue(
        &self,
        password: Option<String>,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.set_encryption_password(password))
    }

    pub fn accept_all_revisions_enqueue(&self) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.accept_all_revisions())
    }

    pub fn reject_all_revisions_enqueue(&self) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.reject_all_revisions())
    }

    pub fn accept_revision_at_enqueue(&self, caret_run: &str) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(Some(caret_run)).ok_or_else(|| "invalid run id".to_string())?;
        self.enqueue_edit(session.accept_revision_at(Some(caret)))
    }

    pub fn reject_revision_at_enqueue(&self, caret_run: &str) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(Some(caret_run)).ok_or_else(|| "invalid run id".to_string())?;
        self.enqueue_edit(session.reject_revision_at(Some(caret)))
    }

    pub fn adjacent_revision_run(&self, caret_run: &str, forward: bool) -> Result<String, String> {
        let session = self.session().expect("WasmSession not initialized");
        let caret = parse_run_id(Some(caret_run)).ok_or_else(|| "invalid run id".to_string())?;
        session
            .adjacent_revision_run(Some(caret), forward)
            .map(|id| id.to_string())
            .ok_or_else(|| "no revision".to_string())
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

    pub fn document_outline_json(&self) -> String {
        self.session()
            .and_then(|s| s.document_outline_json())
            .unwrap_or_else(|| "[]".to_string())
    }

    pub fn bookmarks_json(&self) -> String {
        self.session()
            .and_then(|s| s.bookmarks_json())
            .unwrap_or_else(|| "[]".to_string())
    }

    pub fn semantic_tree_json(&self) -> String {
        self.session()
            .and_then(|s| s.semantic_tree_json())
            .unwrap_or_else(|| "[]".to_string())
    }

    pub fn accessibility_issues_json(&self) -> String {
        self.session()
            .and_then(|s| s.accessibility_issues_json())
            .unwrap_or_else(|| "[]".to_string())
    }

    pub fn document_inspect_json(&self) -> String {
        self.session()
            .and_then(|s| s.document_inspect_json())
            .unwrap_or_else(|| "[]".to_string())
    }

    pub fn remove_inspect_findings_enqueue(
        &self,
        comments: bool,
        metadata: bool,
        hidden_text: bool,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.remove_inspect_findings(comments, metadata, hidden_text))
    }

    pub fn digital_signatures_json(&self) -> String {
        self.session()
            .and_then(|s| s.digital_signatures_json())
            .unwrap_or_else(|| "[]".to_string())
    }

    pub fn verify_signatures_json(&self) -> String {
        self.session()
            .and_then(|s| s.verify_signatures_json())
            .unwrap_or_else(|| "[]".to_string())
    }

    pub fn sign_document_enqueue(
        &self,
        name: &str,
        email: &str,
        organization: Option<String>,
    ) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        let request_id = session.sign_document(name, email, organization)?;
        Ok(request_id)
    }

    pub fn clear_digital_signatures_enqueue(&self) -> Result<u64, String> {
        let session = self.session().expect("WasmSession not initialized");
        self.enqueue_edit(session.clear_digital_signatures())
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

    pub fn last_split_caret(&self) -> Option<(String, u32)> {
        let session = self.session().expect("WasmSession not initialized");
        session
            .last_split_caret()
            .map(|(run_id, offset)| (run_id.as_uuid().to_string(), offset as u32))
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
            match self.session.open_bytes_and_wait(data.to_vec()) {
                Ok(()) => {
                    self.last_error.clear();
                    Ok(())
                }
                Err(e) => {
                    self.record_error(e.to_string());
                    Err(JsValue::from_str(&e.to_string()))
                }
            }
        }

        pub fn open_document_with_path(&mut self, data: &[u8], path: &str) -> Result<(), JsValue> {
            self.open_document_with_password(data, path, "")
        }

        /// Open document bytes; [password] decrypts encrypted DOCX when non-empty (F22.S1).
        pub fn open_document_with_password(
            &mut self,
            data: &[u8],
            path: &str,
            password: &str,
        ) -> Result<(), JsValue> {
            let hint = if path.is_empty() {
                None
            } else {
                Some(path.to_string())
            };
            let pw = if password.is_empty() {
                None
            } else {
                Some(password.to_string())
            };
            match self
                .session
                .open_bytes_with_path_and_password_and_wait(data.to_vec(), hint, pw)
            {
                Ok(()) => {
                    self.last_error.clear();
                    Ok(())
                }
                Err(e) => {
                    self.record_error(e.to_string());
                    Err(JsValue::from_str(&e.to_string()))
                }
            }
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

        pub fn export_pdf_for_print(
            &mut self,
            scale_mode: i32,
            scale_percent: f32,
            margin_left: f32,
            margin_right: f32,
            margin_top: f32,
            margin_bottom: f32,
            duplex: i32,
            pages_per_sheet: i32,
            booklet: i32,
        ) -> Result<Vec<u8>, JsValue> {
            let layout = tw_pdf::print_layout_from_codes(
                scale_mode,
                scale_percent,
                margin_left,
                margin_right,
                margin_top,
                margin_bottom,
                duplex,
                pages_per_sheet,
                booklet,
            );
            match self.session.export_pdf_for_print_and_wait(layout, None) {
                Ok(bytes) => {
                    self.last_error.clear();
                    Ok(bytes)
                }
                Err(e) => {
                    self.last_error = e.clone();
                    Err(JsValue::from_str(&e))
                }
            }
        }

        pub fn export_pdf_for_print_selection(
            &mut self,
            start_run_id: &str,
            start_offset: u32,
            end_run_id: &str,
            end_offset: u32,
            scale_mode: i32,
            scale_percent: f32,
            margin_left: f32,
            margin_right: f32,
            margin_top: f32,
            margin_bottom: f32,
            duplex: i32,
            pages_per_sheet: i32,
            booklet: i32,
        ) -> Result<Vec<u8>, JsValue> {
            let Ok(start_uuid) = uuid::Uuid::parse_str(start_run_id) else {
                return Err(JsValue::from_str("invalid start_run_id"));
            };
            let Ok(end_uuid) = uuid::Uuid::parse_str(end_run_id) else {
                return Err(JsValue::from_str("invalid end_run_id"));
            };
            let layout = tw_pdf::print_layout_from_codes(
                scale_mode,
                scale_percent,
                margin_left,
                margin_right,
                margin_top,
                margin_bottom,
                duplex,
                pages_per_sheet,
                booklet,
            );
            let range = tw_edit::DocRange {
                start: tw_edit::DocPosition {
                    run_id: tw_model::NodeId::from_uuid(start_uuid),
                    char_offset: start_offset as usize,
                },
                end: tw_edit::DocPosition {
                    run_id: tw_model::NodeId::from_uuid(end_uuid),
                    char_offset: end_offset as usize,
                },
            };
            match self
                .session
                .export_pdf_for_print_and_wait(layout, Some(range))
            {
                Ok(bytes) => {
                    self.last_error.clear();
                    Ok(bytes)
                }
                Err(e) => {
                    self.last_error = e.clone();
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

        pub fn grammar_check(&mut self) -> Result<String, JsValue> {
            match self.session.grammar_check_and_wait() {
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

        pub fn compare_document_text(&self, other: &str) -> String {
            self.session.compare_document_text(other)
        }

        pub fn find_matches(
            &self,
            query: &str,
            match_case: bool,
            use_regex: bool,
            use_wildcards: bool,
            format_json: &str,
        ) -> String {
            self.session
                .find_matches_json(query, match_case, use_regex, use_wildcards, format_json)
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

        pub fn apply_paragraph_style(
            &mut self,
            caret_run_id: &str,
            style_name: &str,
        ) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.apply_paragraph_style_enqueue(caret, style_name))
        }

        pub fn apply_document_theme(&mut self, theme_name: &str) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.apply_document_theme_enqueue(theme_name))
        }

        pub fn apply_section_format(
            &mut self,
            format_json: &str,
            caret_run_id: &str,
        ) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.apply_section_format_enqueue(format_json, caret))
        }

        pub fn get_section_format_json(&self, caret_run_id: &str) -> Result<String, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.session
                .section_format_json(caret)
                .map_err(|e| JsValue::from_str(&e))
        }

        pub fn insert_section_break(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.insert_section_break_enqueue(caret))
        }

        pub fn ensure_header_footer(
            &mut self,
            caret_run_id: &str,
            is_header: bool,
            page_index: i32,
        ) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            let page = if page_index < 0 {
                None
            } else {
                Some(page_index as u32)
            };
            self.enqueue_op(self.session.ensure_header_footer_enqueue(caret, is_header, page))
        }

        pub fn set_even_and_odd_headers(&mut self, enabled: bool) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.set_even_and_odd_headers_enqueue(enabled))
        }

        pub fn even_and_odd_headers_enabled(&self) -> bool {
            self.session.even_and_odd_headers_enabled()
        }

        pub fn header_footer_linked(
            &self,
            caret_run_id: &str,
            is_header: bool,
            page_index: i32,
        ) -> bool {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            let page = if page_index < 0 {
                None
            } else {
                Some(page_index as u32)
            };
            self.session.header_footer_linked(caret, is_header, page)
        }

        pub fn set_header_footer_link(
            &mut self,
            caret_run_id: &str,
            is_header: bool,
            page_index: i32,
            linked: bool,
        ) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            let page = if page_index < 0 {
                None
            } else {
                Some(page_index as u32)
            };
            self.enqueue_op(self.session.set_header_footer_link_enqueue(
                caret,
                is_header,
                page,
                linked,
            ))
        }

        pub fn header_footer_seed_run(
            &self,
            caret_run_id: &str,
            is_header: bool,
            page_index: i32,
        ) -> Result<String, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            let page = if page_index < 0 {
                None
            } else {
                Some(page_index as u32)
            };
            self.session
                .header_footer_seed_run(caret, is_header, page)
                .map_err(|e| JsValue::from_str(&e))
        }

        pub fn insert_field(&mut self, run_id: &str, offset: u32, field_type: &str) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_field_enqueue(run_id, offset as usize, field_type))
        }

        pub fn insert_form_field(
            &mut self,
            run_id: &str,
            offset: u32,
            kind: &str,
            name: &str,
            initial_value: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_form_field_enqueue(
                run_id,
                offset as usize,
                kind,
                name,
                initial_value,
            ))
        }

        pub fn set_form_field_value(
            &mut self,
            run_id: &str,
            value: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.set_form_field_value_enqueue(run_id, value))
        }

        pub fn insert_merge_field(
            &mut self,
            run_id: &str,
            offset: u32,
            name: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(
                self.session
                    .insert_merge_field_enqueue(run_id, offset as usize, name),
            )
        }

        pub fn apply_mail_merge_row(&mut self, values_json: &str) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.apply_mail_merge_row_enqueue(values_json))
        }

        pub fn insert_footnote(&mut self, run_id: &str, offset: u32) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_footnote_enqueue(run_id, offset as usize))
        }

        pub fn insert_comment(
            &mut self,
            run_id: &str,
            offset: u32,
            body_text: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(
                self.session
                    .insert_comment_enqueue(run_id, offset as usize, body_text),
            )
        }

        pub fn insert_table_of_contents(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.insert_table_of_contents_enqueue(caret))
        }

        pub fn add_bibliography_source(
            &mut self,
            key: &str,
            author: &str,
            title: &str,
            year: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.add_bibliography_source_enqueue(key, author, title, year))
        }

        pub fn insert_citation(
            &mut self,
            run_id: &str,
            offset: u32,
            source_key: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(
                self.session
                    .insert_citation_enqueue(run_id, offset as usize, source_key),
            )
        }

        pub fn insert_bibliography(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.insert_bibliography_enqueue(caret))
        }

        pub fn insert_bookmark(
            &mut self,
            run_id: &str,
            offset: u32,
            name: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(
                self.session
                    .insert_bookmark_enqueue(run_id, offset as usize, name),
            )
        }

        pub fn insert_hyperlink(
            &mut self,
            run_id: &str,
            offset: u32,
            url: &str,
            text: &str,
            tooltip: &str,
        ) -> Result<f64, JsValue> {
            let tip = if tooltip.is_empty() {
                None
            } else {
                Some(tooltip)
            };
            self.enqueue_op(self.session.insert_hyperlink_enqueue(
                run_id,
                offset as usize,
                url,
                text,
                tip,
            ))
        }

        pub fn insert_cross_reference(
            &mut self,
            run_id: &str,
            offset: u32,
            bookmark_name: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_cross_reference_enqueue(
                run_id,
                offset as usize,
                bookmark_name,
            ))
        }

        pub fn insert_index(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.insert_index_enqueue(caret))
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

        pub fn insert_table(
            &mut self,
            rows: u32,
            cols: u32,
            caret_run_id: &str,
        ) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.insert_table_enqueue(rows, cols, caret))
        }

        pub fn delete_table_row(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.delete_table_row_enqueue(caret))
        }

        pub fn delete_table_column(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.delete_table_column_enqueue(caret))
        }

        pub fn merge_table_cells(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.merge_table_cells_enqueue(caret))
        }

        pub fn split_table_cell(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.split_table_cell_enqueue(caret))
        }

        pub fn set_table_border(
            &mut self,
            caret_run_id: &str,
            width: f32,
            color_r: u8,
            color_g: u8,
            color_b: u8,
            color_a: u8,
        ) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.set_table_border_enqueue(
                caret, width, color_r, color_g, color_b, color_a,
            ))
        }

        pub fn set_table_cell_shading(
            &mut self,
            caret_run_id: &str,
            color_r: i32,
            color_g: u8,
            color_b: u8,
            color_a: u8,
        ) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.set_table_cell_shading_enqueue(
                caret, color_r, color_g, color_b, color_a,
            ))
        }

        pub fn resize_table_column(
            &mut self,
            caret_run_id: &str,
            width: f32,
        ) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.resize_table_column_enqueue(caret, width))
        }

        pub fn autofit_table(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.autofit_table_enqueue(caret))
        }

        pub fn sort_table_rows(
            &mut self,
            caret_run_id: &str,
            ascending: bool,
        ) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.sort_table_rows_enqueue(caret, ascending))
        }

        pub fn insert_nested_table(
            &mut self,
            caret_run_id: &str,
            rows: u32,
            cols: u32,
        ) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.insert_nested_table_enqueue(caret, rows, cols))
        }

        pub fn insert_table_sum_field(&mut self, caret_run_id: &str) -> Result<f64, JsValue> {
            let caret = if caret_run_id.is_empty() {
                None
            } else {
                Some(caret_run_id)
            };
            self.enqueue_op(self.session.insert_table_sum_field_enqueue(caret))
        }

        pub fn insert_image(&mut self, width: f32, height: f32) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_image_enqueue(width, height))
        }

        pub fn insert_shape(&mut self, shape_type: i32) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_shape_enqueue(shape_type))
        }

        pub fn insert_text_box(&mut self) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_text_box_enqueue())
        }

        pub fn insert_word_art(&mut self, text: &str) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_word_art_enqueue(text.to_string()))
        }

        pub fn insert_diagram(&mut self, diagram_type: i32) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_diagram_enqueue(diagram_type))
        }

        pub fn insert_chart(&mut self, chart_type: i32) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_chart_enqueue(chart_type))
        }

        pub fn get_chart_data_json(&self, shape_id: &str) -> Result<String, JsValue> {
            self.session
                .chart_data_json(shape_id)
                .map_err(|e| JsValue::from_str(&e))
        }

        pub fn latest_chart_id(&self) -> Result<String, JsValue> {
            self.session
                .latest_chart_id()
                .map_err(|e| JsValue::from_str(&e))
        }

        pub fn set_chart_data_json(
            &mut self,
            shape_id: &str,
            chart_json: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.set_chart_data_enqueue(shape_id, chart_json))
        }

        pub fn insert_office_math(
            &mut self,
            run_id: &str,
            offset: u32,
            xml: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_office_math_enqueue(
                run_id,
                offset as usize,
                xml,
            ))
        }

        pub fn insert_office_math_display(
            &mut self,
            caret_run_id: Option<String>,
            xml: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_office_math_display_enqueue(
                caret_run_id.as_deref(),
                xml,
            ))
        }

        pub fn set_office_math_xml(&mut self, run_id: &str, xml: &str) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.set_office_math_enqueue(run_id, xml))
        }

        pub fn get_office_math_xml(&self, run_id: &str) -> Result<String, JsValue> {
            self.session
                .office_math_xml(run_id)
                .map_err(|e| JsValue::from_str(&e))
        }

        pub fn latest_office_math_run_id(&self) -> Result<String, JsValue> {
            self.session
                .latest_office_math_run_id()
                .map_err(|e| JsValue::from_str(&e))
        }

        pub fn delete_block(&mut self, block_id: &str) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.delete_block_enqueue(block_id))
        }

        pub fn insert_image_bytes(&mut self, data: &[u8], mime_type: &str) -> Result<f64, JsValue> {
            self.enqueue_op(
                self.session
                    .insert_image_bytes_enqueue(data.to_vec(), mime_type.to_string()),
            )
        }

        pub fn set_image_size(
            &mut self,
            image_id: &str,
            width: f32,
            height: f32,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.set_image_size_enqueue(image_id, width, height))
        }

        pub fn replace_image_bytes(
            &mut self,
            image_id: &str,
            data: &[u8],
            mime_type: &str,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.replace_image_bytes_enqueue(
                image_id,
                data.to_vec(),
                mime_type.to_string(),
            ))
        }

        pub fn set_image_wrap(&mut self, image_id: &str, wrap: u8) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.set_image_wrap_enqueue(image_id, wrap))
        }

        pub fn set_image_anchor(
            &mut self,
            image_id: &str,
            x: f32,
            y: f32,
            origin_x: u8,
            origin_y: u8,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.set_image_anchor_enqueue(
                image_id, x, y, origin_x, origin_y,
            ))
        }

        pub fn set_image_transform(
            &mut self,
            image_id: &str,
            rotation_deg: f32,
            crop_left: f32,
            crop_top: f32,
            crop_right: f32,
            crop_bottom: f32,
            opacity: f32,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.set_image_transform_enqueue(
                image_id,
                rotation_deg,
                crop_left,
                crop_top,
                crop_right,
                crop_bottom,
                opacity,
            ))
        }

        pub fn insert_image_caption(&mut self, image_id: &str) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.insert_image_caption_enqueue(image_id))
        }

        pub fn set_image_alt_text(
            &mut self,
            image_id: &str,
            alt_text: Option<String>,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(
                self.session
                    .set_image_alt_text_enqueue(image_id, alt_text),
            )
        }

        pub fn image_alt_text(&self, image_id: &str) -> Result<String, JsValue> {
            self.session
                .image_alt_text(image_id)
                .map_err(|e| JsValue::from_str(&e))
        }

        pub fn compress_image(&mut self, image_id: &str, quality: u8) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.compress_image_enqueue(image_id, quality))
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

        pub fn set_read_only(&mut self, enabled: bool) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.set_read_only_enqueue(enabled))
        }

        /// Set or clear DOCX encryption password for subsequent saves (F22.S2).
        /// Empty string clears the password.
        pub fn set_encryption_password(&mut self, password: &str) -> Result<f64, JsValue> {
            let value = if password.is_empty() {
                None
            } else {
                Some(password.to_string())
            };
            self.enqueue_op(self.session.set_encryption_password_enqueue(value))
        }

        pub fn accept_all_revisions(&mut self) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.accept_all_revisions_enqueue())
        }

        pub fn reject_all_revisions(&mut self) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.reject_all_revisions_enqueue())
        }

        pub fn accept_revision_at(&mut self, caret_run: &str) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.accept_revision_at_enqueue(caret_run))
        }

        pub fn reject_revision_at(&mut self, caret_run: &str) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.reject_revision_at_enqueue(caret_run))
        }

        pub fn adjacent_revision_run(&self, caret_run: &str, forward: bool) -> Result<String, JsValue> {
            self.session
                .adjacent_revision_run(caret_run, forward)
                .map_err(|e| JsValue::from_str(&e))
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

        pub fn document_outline_json(&self) -> String {
            self.session.document_outline_json()
        }

        pub fn bookmarks_json(&self) -> String {
            self.session.bookmarks_json()
        }

        pub fn semantic_tree_json(&self) -> String {
            self.session.semantic_tree_json()
        }

        pub fn accessibility_issues_json(&self) -> String {
            self.session.accessibility_issues_json()
        }

        pub fn document_inspect_json(&self) -> String {
            self.session.document_inspect_json()
        }

        pub fn remove_inspect_findings(
            &mut self,
            comments: bool,
            metadata: bool,
            hidden_text: bool,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.remove_inspect_findings_enqueue(
                comments,
                metadata,
                hidden_text,
            ))
        }

        pub fn digital_signatures_json(&self) -> String {
            self.session.digital_signatures_json()
        }

        pub fn verify_signatures_json(&self) -> String {
            self.session.verify_signatures_json()
        }

        pub fn sign_document(
            &mut self,
            name: &str,
            email: &str,
            organization: Option<String>,
        ) -> Result<f64, JsValue> {
            self.enqueue_op(
                self.session
                    .sign_document_enqueue(name, email, organization),
            )
        }

        pub fn clear_digital_signatures(&mut self) -> Result<f64, JsValue> {
            self.enqueue_op(self.session.clear_digital_signatures_enqueue())
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

        pub fn last_split_caret(&self) -> Option<String> {
            self.session.last_split_caret().map(|(run, offset)| {
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
