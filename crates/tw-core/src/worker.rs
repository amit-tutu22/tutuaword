use crate::bundle::{export_document, import_document_bundle, FormatContext};
use crate::import::DetectedFormat;
use crate::snapshot::{
    document_plain_text, snapshot_from_pages, SinglePageSnapshot, SnapshotBuffer,
};
use crossbeam_channel::{Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tw_edit::{Command, EditSession};
use tw_layout::LayoutEngine;
use tw_model::{Block, NodeId, NumberingRef};
use tw_pdf::PdfExporter;
use tw_render::DisplayListBuilder;

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
    ApplyHeading1,
    ApplyBulletList,
    InsertTable { rows: u32, cols: u32 },
    InsertImage { width: f32, height: f32 },
    Undo,
    Redo,
    ExportPdf,
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum BridgeEvent {
    DisplayListReady { page: u32, version: u64 },
    DocumentOpened { page_count: u32 },
    DocumentSaved { data: Vec<u8> },
    SpellCheckResult { misspellings: Vec<String> },
    Error { message: String },
}

pub struct WorkerHandle {
    pub cmd_tx: Sender<BridgeCommand>,
    pub event_rx: Receiver<BridgeEvent>,
    join: Option<JoinHandle<()>>,
}

impl WorkerHandle {
    pub fn spawn(snapshot: Arc<SnapshotBuffer>, layout_cache: crate::SharedLayoutCache) -> Self {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

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
        let _ = self.cmd_tx.send(BridgeCommand::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn worker_loop(
    cmd_rx: Receiver<BridgeCommand>,
    event_tx: Sender<BridgeEvent>,
    snapshot: Arc<SnapshotBuffer>,
    layout_cache: crate::SharedLayoutCache,
) {
    let mut session = EditSession::new();
    let mut format_ctx = FormatContext::new_document();
    let mut layout = LayoutEngine::new();
    let mut version: u64 = 0;
    let mut current_page: u32 = 0;

    let rebuild = |session: &EditSession,
                   layout: &mut LayoutEngine,
                   version: &mut u64,
                   current_page: u32| {
        layout.invalidate_all();
        let doc_layout = layout.layout_document(&session.document);
        *version += 1;
        let text = document_plain_text(&session.document);
        let pages: Vec<SinglePageSnapshot> = doc_layout
            .pages
            .iter()
            .map(|page| {
                let list = DisplayListBuilder::from_page(page, layout.atlas(), *version);
                SinglePageSnapshot {
                    bytes: DisplayListBuilder::to_bytes(&list),
                    page_width: page.width,
                    page_height: page.height,
                }
            })
            .collect();

        let page_count = pages.len().max(1) as u32;
        let page_index = current_page.min(page_count.saturating_sub(1));
        snapshot.publish(snapshot_from_pages(pages, page_index, *version, text));
        layout_cache.write().update_from_engine(layout);
        *version
    };

    version = rebuild(&session, &mut layout, &mut version, current_page);
    let page_count = layout.page_count() as u32;
    let _ = event_tx.send(BridgeEvent::DocumentOpened { page_count });

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            BridgeCommand::NewDocument => {
                session = EditSession::new();
                format_ctx = FormatContext::new_document();
                current_page = 0;
                version = rebuild(&session, &mut layout, &mut version, current_page);
                let page_count = layout.page_count() as u32;
                let _ = event_tx.send(BridgeEvent::DocumentOpened { page_count });
            }
            BridgeCommand::OpenDocument { data, path_hint } => {
                match import_document_bundle(&data, path_hint.as_deref()) {
                    Ok(bundle) => {
                        format_ctx = FormatContext::from_bundle(bundle.clone(), path_hint);
                        session = EditSession::from_document(bundle.document);
                        current_page = 0;
                        version = rebuild(&session, &mut layout, &mut version, current_page);
                        let page_count = layout.page_count() as u32;
                        let _ = event_tx.send(BridgeEvent::DocumentOpened { page_count });
                    }
                    Err(e) => {
                        let _ = event_tx.send(BridgeEvent::Error {
                            message: e.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::SaveDocument => match export_document(&session.document, &format_ctx) {
                Ok(data) => {
                    let _ = event_tx.send(BridgeEvent::DocumentSaved { data });
                }
                Err(e) => {
                    let _ = event_tx.send(BridgeEvent::Error {
                        message: e.to_string(),
                    });
                }
            },
            BridgeCommand::SaveDocumentAs { format } => {
                format_ctx.save_format = format;
                match export_document(&session.document, &format_ctx) {
                    Ok(data) => {
                        let _ = event_tx.send(BridgeEvent::DocumentSaved { data });
                    }
                    Err(e) => {
                        let _ = event_tx.send(BridgeEvent::Error {
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
                let _ = event_tx.send(BridgeEvent::SpellCheckResult {
                    misspellings: words,
                });
            },
            BridgeCommand::ToggleTrackChanges { enabled } => {
                session.document.settings.track_changes_enabled = enabled;
            },
            BridgeCommand::ApplyEdit { command } => {
                if session.apply(command).is_ok() {
                    format_ctx.mark_document_modified();
                    version = rebuild(&session, &mut layout, &mut version, current_page);
                    let _ = event_tx.send(BridgeEvent::DisplayListReady {
                        page: current_page,
                        version,
                    });
                }
            }
            BridgeCommand::ApplyHeading1 => {
                if let Some(cmd) = heading1_command(&session.document) {
                    let _ = session.apply(cmd);
                    version = rebuild(&session, &mut layout, &mut version, current_page);
                    let _ = event_tx.send(BridgeEvent::DisplayListReady {
                        page: current_page,
                        version,
                    });
                }
            }
            BridgeCommand::ApplyBulletList => {
                if let Some(cmd) = bullet_list_command(&session.document) {
                    let _ = session.apply(cmd);
                    version = rebuild(&session, &mut layout, &mut version, current_page);
                    let _ = event_tx.send(BridgeEvent::DisplayListReady {
                        page: current_page,
                        version,
                    });
                }
            }
            BridgeCommand::InsertTable { rows, cols } => {
                if let Some(cmd) = insert_table_command(&session.document, rows, cols) {
                    let _ = session.apply(cmd);
                    version = rebuild(&session, &mut layout, &mut version, current_page);
                    let _ = event_tx.send(BridgeEvent::DisplayListReady {
                        page: current_page,
                        version,
                    });
                }
            }
            BridgeCommand::InsertImage { width, height } => {
                if let Some(cmd) = insert_image_command(&session.document, width, height) {
                    let _ = session.apply(cmd);
                    version = rebuild(&session, &mut layout, &mut version, current_page);
                    let _ = event_tx.send(BridgeEvent::DisplayListReady {
                        page: current_page,
                        version,
                    });
                }
            }
            BridgeCommand::ExportPdf => {
                let exporter = tw_pdf::DisplayListPdfExporter;
                match exporter.export(&session.document, &tw_pdf::PdfExportOptions::default()) {
                    Ok(data) => {
                        let _ = event_tx.send(BridgeEvent::DocumentSaved { data });
                    }
                    Err(e) => {
                        let _ = event_tx.send(BridgeEvent::Error {
                            message: e.to_string(),
                        });
                    }
                }
            }
            BridgeCommand::SetCurrentPage { page } => {
                current_page = page;
                snapshot.set_current_page(page);
                let snap = snapshot.read();
                let _ = event_tx.send(BridgeEvent::DisplayListReady {
                    page: snap.page_index,
                    version: snap.version,
                });
            }
            BridgeCommand::Undo => {
                if session.undo().is_ok() {
                    version = rebuild(&session, &mut layout, &mut version, current_page);
                    let _ = event_tx.send(BridgeEvent::DisplayListReady {
                        page: current_page,
                        version,
                    });
                }
            }
            BridgeCommand::Redo => {
                if session.redo().is_ok() {
                    version = rebuild(&session, &mut layout, &mut version, current_page);
                    let _ = event_tx.send(BridgeEvent::DisplayListReady {
                        page: current_page,
                        version,
                    });
                }
            }
            BridgeCommand::Shutdown => break,
        }
    }
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

pub fn bullet_list_command(doc: &tw_model::Document) -> Option<Command> {
    let para_id = first_paragraph_id(doc)?;
    Some(Command::SetNumbering {
        paragraph_id: para_id,
        numbering: Some(NumberingRef {
            numbering_id: 1,
            level: 0,
        }),
    })
}

pub fn heading1_command(doc: &tw_model::Document) -> Option<Command> {
    let para_id = first_paragraph_id(doc)?;
    Some(Command::ApplyParagraphStyle {
        paragraph_id: para_id,
        style_name: "Heading 1".into(),
    })
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
