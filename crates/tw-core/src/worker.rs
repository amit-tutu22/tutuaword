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
use tw_model::{Block, NodeId, NumberingRef};
use tw_pdf::PdfExporter;
use tw_render::DisplayListBuilder;

/// Reserved for the worker's initial document-ready event (not tied to a caller command).
pub const STARTUP_REQUEST_ID: u64 = 0;

/// Bounded command queue depth (session blocks on `send` when full).
pub const COMMAND_QUEUE_CAPACITY: usize = 512;

/// Bounded worker → session event channel; overflow uses drop-oldest backpressure.
pub const EVENT_CHANNEL_CAPACITY: usize = 256;

/// Outbound event queue with per-page `DisplayListReady` coalescing and drop-oldest.
struct EventPublisher {
    tx: Sender<BridgeEvent>,
    buffer: VecDeque<BridgeEvent>,
}

impl EventPublisher {
    fn new(tx: Sender<BridgeEvent>) -> Self {
        Self {
            tx,
            buffer: VecDeque::new(),
        }
    }

    fn send(&mut self, event: BridgeEvent) {
        if let BridgeEvent::DisplayListReady { page, .. } = &event {
            self.coalesce_display_ready(*page);
        }
        while self.buffer.len() >= EVENT_CHANNEL_CAPACITY {
            self.buffer.pop_front();
        }
        self.buffer.push_back(event);
        self.flush();
    }

    fn coalesce_display_ready(&mut self, page: u32) {
        self.buffer.retain(|event| {
            !matches!(
                event,
                BridgeEvent::DisplayListReady { page: p, .. } if *p == page
            )
        });
    }

    fn flush(&mut self) {
        while let Some(event) = self.buffer.front() {
            match self.tx.try_send(event.clone()) {
                Ok(()) => {
                    self.buffer.pop_front();
                }
                Err(TrySendError::Full(_)) => break,
                Err(TrySendError::Disconnected(_)) => {
                    self.buffer.clear();
                    break;
                }
            }
        }
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
    ApplyHeading1 {
        caret_run_id: Option<NodeId>,
    },
    ApplyNormalStyle {
        caret_run_id: Option<NodeId>,
    },
    ApplyBulletList {
        caret_run_id: Option<NodeId>,
    },
    ApplyNumberedList {
        caret_run_id: Option<NodeId>,
    },
    InsertTable { rows: u32, cols: u32 },
    InsertImage { width: f32, height: f32 },
    InsertPageBreak {
        caret_run_id: Option<NodeId>,
    },
    Undo,
    Redo,
    ExportPdf,
    AcceptAllRevisions,
    RejectAllRevisions,
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
    let mut cached_pages: Vec<SinglePageSnapshot> = Vec::new();

    let rebuild = |session: &EditSession,
                   layout: &mut LayoutEngine,
                   version: &mut u64,
                   current_page: u32,
                   affected_nodes: Option<&[NodeId]>,
                   cached_pages: &mut Vec<SinglePageSnapshot>| {
        match affected_nodes {
            Some(nodes) if !nodes.is_empty() => {
                layout.invalidate_nodes(&session.document, nodes);
            }
            _ => layout.invalidate_all(),
        }
        let doc_layout = layout.layout_document(&session.document);
        *version += 1;
        let text = document_plain_text(&session.document);
        let read_only = session.document.settings.read_only;
        let relayout_start = layout.relayout_start_page() as usize;
        let relayout_count = layout.last_relayout_pages();
        let is_incremental = matches!(affected_nodes, Some(nodes) if !nodes.is_empty())
            && relayout_count < doc_layout.pages.len();

        let pages: Vec<SinglePageSnapshot> = if is_incremental
            && cached_pages.len() == doc_layout.pages.len()
        {
            let mut display_targets: Vec<usize> = (relayout_start
                ..relayout_start + relayout_count)
                .filter(|&idx| idx == current_page as usize)
                .collect();
            if display_targets.is_empty() {
                display_targets.push(relayout_start);
            }
            for idx in display_targets {
                let page = &doc_layout.pages[idx];
                let list =
                    DisplayListBuilder::from_page_without_atlas(page, *version);
                cached_pages[idx] = SinglePageSnapshot {
                    bytes: Arc::new(DisplayListBuilder::to_page_bytes(&list)),
                    page_width: page.width,
                    page_height: page.height,
                };
            }
            cached_pages.clone()
        } else {
            doc_layout
                .pages
                .iter()
                .map(|page| {
                    let list =
                        DisplayListBuilder::from_page_without_atlas(page, *version);
                    SinglePageSnapshot {
                        bytes: Arc::new(DisplayListBuilder::to_page_bytes(&list)),
                        page_width: page.width,
                        page_height: page.height,
                    }
                })
                .collect()
        };

        *cached_pages = pages.clone();

        let atlas = layout.atlas();
        let atlas_generation = atlas.generation;
        let atlas_width = atlas.width;
        let atlas_height = atlas.height;
        let atlas_bytes = Arc::new(DisplayListBuilder::atlas_to_bytes(atlas, atlas_generation));

        let page_count = pages.len().max(1) as u32;
        let props_json = document_properties_json(&session.document, page_count);
        let page_index = current_page.min(page_count.saturating_sub(1));
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
        let mut cache = layout_cache.write();
        if is_incremental {
            cache.update_from_session_incremental(
                layout,
                &session.document,
                &session.buffer,
                *version,
                relayout_start as u32,
                page_count,
            );
        } else {
            cache.update_from_session(layout, &session.document, &session.buffer);
        }
        *version
    };

    version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
    let page_count = layout.page_count() as u32;
    events.send(BridgeEvent::DocumentOpened {
        request_id: STARTUP_REQUEST_ID,
        page_count,
    });

    let mut pending: Option<QueuedCommand> = None;

    loop {
        let QueuedCommand {
            request_id: req_id,
            inner: cmd,
        } = match pending.take() {
            Some(queued) => queued,
            None => match cmd_rx.recv() {
                Ok(queued) => queued,
                Err(_) => break,
            },
        };

        match cmd {
            BridgeCommand::NewDocument => {
                session = EditSession::new();
                format_ctx = FormatContext::new_document();
                current_page = 0;
                version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
                let page_count = layout.page_count() as u32;
                events.send(BridgeEvent::DocumentOpened { request_id: req_id, page_count });
            }
            BridgeCommand::OpenDocument { data, path_hint } => {
                match import_document_bundle(&data, path_hint.as_deref()) {
                    Ok(bundle) => {
                        format_ctx = FormatContext::from_bundle(bundle.clone(), path_hint);
                        session = EditSession::from_document(bundle.document);
                        current_page = 0;
                        version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
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
            BridgeCommand::AcceptAllRevisions => {
                match session.apply(Command::AcceptAllRevisions) {
                    Ok(_) => {
                        format_ctx.mark_document_modified();
                        version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: req_id,
                            page: current_page,
                            version,
                        });
                    }
                    Err(err) => {
                        events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: err.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::RejectAllRevisions => {
                match session.apply(Command::RejectAllRevisions) {
                    Ok(_) => {
                        format_ctx.mark_document_modified();
                        version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: req_id,
                            page: current_page,
                            version,
                        });
                    }
                    Err(err) => {
                        events.send(BridgeEvent::Error {
                            request_id: req_id,
                            message: err.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::PasteHtml {
                run_id,
                offset,
                html,
            } => match tw_html::import(&html) {
                Ok(doc) => {
                    if paste::paste_fragment_at(&mut session, run_id, offset, &doc).is_ok() {
                        format_ctx.mark_document_modified();
                        version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
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
                        version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
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
                let mut applied = 0usize;
                let mut affected_nodes: Vec<NodeId> = Vec::new();
                for command in commands {
                    match session.apply(command) {
                        Ok(result) => {
                            affected_nodes.extend(result.affected_nodes);
                            applied += 1;
                        }
                        Err(e) => {
                            for _ in 0..applied {
                                let _ = session.undo();
                            }
                            apply_error = Some(e);
                            break;
                        }
                    }
                }

                if let Some(e) = apply_error {
                    format_ctx.mark_document_modified();
                    version = rebuild(
                        &session,
                        &mut layout,
                        &mut version,
                        current_page,
                        if affected_nodes.is_empty() {
                            None
                        } else {
                            Some(affected_nodes.as_slice())
                        },
                        &mut cached_pages,
                    );
                    for batch_id in &request_ids {
                        events.send(BridgeEvent::Error {
                            request_id: *batch_id,
                            message: e.to_string(),
                        });
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: *batch_id,
                            page: current_page,
                            version,
                        });
                    }
                } else {
                    format_ctx.mark_document_modified();
                    version = rebuild(
                        &session,
                        &mut layout,
                        &mut version,
                        current_page,
                        if affected_nodes.is_empty() {
                            None
                        } else {
                            Some(affected_nodes.as_slice())
                        },
                        &mut cached_pages,
                    );
                    for batch_id in request_ids {
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: batch_id,
                            page: current_page,
                            version,
                        });
                    }
                }
            }
            BridgeCommand::ApplyHeading1 { caret_run_id } => {
                let para_id = paragraph_id_from_caret(&session.document, caret_run_id);
                if let Some(para_id) = para_id {
                    if let Some(cmd) = heading1_command_for(para_id) {
                        let _ = session.apply(cmd);
                        version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: req_id,
                            page: current_page,
                            version,
                        });
                    }
                }
            }
            BridgeCommand::ApplyNormalStyle { caret_run_id } => {
                let para_id = paragraph_id_from_caret(&session.document, caret_run_id);
                if let Some(para_id) = para_id {
                    if let Some(cmd) = normal_style_command_for(para_id) {
                        let _ = session.apply(cmd);
                        version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: req_id,
                            page: current_page,
                            version,
                        });
                    }
                }
            }
            BridgeCommand::ApplyBulletList { caret_run_id } => {
                let para_id = paragraph_id_from_caret(&session.document, caret_run_id);
                if let Some(para_id) = para_id {
                    if let Some(cmd) = bullet_list_command_for(para_id) {
                        let _ = session.apply(cmd);
                        version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: req_id,
                            page: current_page,
                            version,
                        });
                    }
                }
            }
            BridgeCommand::ApplyNumberedList { caret_run_id } => {
                let para_id = paragraph_id_from_caret(&session.document, caret_run_id);
                if let Some(para_id) = para_id {
                    if let Some(cmd) = numbered_list_command_for(para_id) {
                        let _ = session.apply(cmd);
                        version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
                        events.send(BridgeEvent::DisplayListReady {
                            request_id: req_id,
                            page: current_page,
                            version,
                        });
                    }
                }
            }
            BridgeCommand::InsertTable { rows, cols } => {
                if let Some(cmd) = insert_table_command(&session.document, rows, cols) {
                    let _ = session.apply(cmd);
                    version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
                    events.send(BridgeEvent::DisplayListReady {
                        request_id: req_id,
                        page: current_page,
                        version,
                    });
                }
            }
            BridgeCommand::InsertImage { width, height } => {
                if let Some(cmd) = insert_image_command(&session.document, width, height) {
                    let _ = session.apply(cmd);
                    version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
                    events.send(BridgeEvent::DisplayListReady {
                        request_id: req_id,
                        page: current_page,
                        version,
                    });
                }
            }
            BridgeCommand::InsertPageBreak { caret_run_id } => {
                if let Some(cmd) = insert_page_break_command_for(&session.document, caret_run_id)
                {
                    let _ = session.apply(cmd);
                    version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
                    events.send(BridgeEvent::DisplayListReady {
                        request_id: req_id,
                        page: current_page,
                        version,
                    });
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
                    version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
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
                    version = rebuild(&session, &mut layout, &mut version, current_page, None, &mut cached_pages);
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

fn paragraph_id_from_caret(
    doc: &tw_model::Document,
    caret_run_id: Option<NodeId>,
) -> Option<NodeId> {
    caret_run_id
        .and_then(|run_id| tw_edit::paragraph_id_for_run(doc, run_id).ok())
        .or_else(|| first_paragraph_id(doc))
}

pub fn first_paragraph_id(doc: &tw_model::Document) -> Option<NodeId> {
    doc.sections.first()?.blocks.iter().find_map(|b| match b {
        Block::Paragraph(p) => Some(p.id),
        _ => None,
    })
}

pub fn last_block_id(doc: &tw_model::Document) -> Option<NodeId> {
    doc.sections.first()?.blocks.last().map(|b| match b {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
    })
}

pub fn numbered_list_command_for(paragraph_id: NodeId) -> Option<Command> {
    Some(Command::SetNumbering {
        paragraph_id,
        numbering: Some(NumberingRef {
            numbering_id: 2,
            level: 0,
        }),
    })
}

pub fn bullet_list_command_for(paragraph_id: NodeId) -> Option<Command> {
    Some(Command::SetNumbering {
        paragraph_id,
        numbering: Some(NumberingRef {
            numbering_id: 1,
            level: 0,
        }),
    })
}

pub fn heading1_command_for(paragraph_id: NodeId) -> Option<Command> {
    Some(Command::ApplyParagraphStyle {
        paragraph_id,
        style_name: "Heading 1".into(),
    })
}

pub fn normal_style_command_for(paragraph_id: NodeId) -> Option<Command> {
    Some(Command::ApplyParagraphStyle {
        paragraph_id,
        style_name: "Normal".into(),
    })
}

pub fn numbered_list_command(doc: &tw_model::Document) -> Option<Command> {
    let para_id = first_paragraph_id(doc)?;
    numbered_list_command_for(para_id)
}

pub fn bullet_list_command(doc: &tw_model::Document) -> Option<Command> {
    let para_id = first_paragraph_id(doc)?;
    bullet_list_command_for(para_id)
}

pub fn heading1_command(doc: &tw_model::Document) -> Option<Command> {
    let para_id = first_paragraph_id(doc)?;
    heading1_command_for(para_id)
}

pub fn insert_table_command(doc: &tw_model::Document, rows: u32, cols: u32) -> Option<Command> {
    let after = last_block_id(doc)?;
    Some(Command::InsertTable {
        after_block_id: after,
        rows,
        cols,
    })
}

pub fn insert_image_command(doc: &tw_model::Document, width: f32, height: f32) -> Option<Command> {
    let after = last_block_id(doc)?;
    Some(Command::InsertImage {
        after_block_id: after,
        width,
        height,
    })
}

pub fn insert_page_break_command_for(
    doc: &tw_model::Document,
    caret_run_id: Option<NodeId>,
) -> Option<Command> {
    let after = paragraph_id_from_caret(doc, caret_run_id)?;
    Some(Command::InsertPageBreak {
        after_block_id: after,
    })
}
