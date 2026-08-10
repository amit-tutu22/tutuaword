use crate::ids::NodeId;
use crate::list::NumberingCatalog;
use crate::nodes::{Block, Paragraph, Run, RunContent, Section};
use crate::properties::DocumentProperties;
use crate::styles::StyleSheet;
use crate::table::Table;
use crate::theme::DocumentTheme;
use crate::bibliography::BibliographySource;
use crate::comments::CommentThread;
use crate::digital_signature::DigitalSignature;
use crate::vocabulary::{BlockZone, Footnote, HeaderFooter, HeaderFooterType, RunLocation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentSettings {
    pub track_changes_enabled: bool,
    pub author_name: String,
    pub default_tab_stop: f32,
    pub numbering: NumberingCatalog,
    pub theme: DocumentTheme,
    pub template_name: Option<String>,
    /// When true, the document cannot be edited (from DOCX protection or app policy).
    #[serde(default)]
    pub read_only: bool,
    /// When true, odd and even pages use distinct header/footer variants (`w:evenAndOddHeaders`).
    #[serde(default)]
    pub even_and_odd_headers: bool,
}

impl DocumentSettings {
    pub fn default_settings() -> Self {
        Self {
            track_changes_enabled: false,
            author_name: "Author".into(),
            default_tab_stop: 36.0,
            numbering: NumberingCatalog::with_defaults(),
            theme: DocumentTheme::default(),
            template_name: None,
            read_only: false,
            even_and_odd_headers: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: NodeId,
    pub styles: StyleSheet,
    pub settings: DocumentSettings,
    #[serde(default)]
    pub properties: DocumentProperties,
    pub sections: Vec<Section>,
    /// Footnote bodies keyed by OOXML `w:id` (F16.S1).
    #[serde(default)]
    pub footnotes: Vec<Footnote>,
    /// Bibliography sources keyed by citation tag (F16.S3).
    #[serde(default)]
    pub bibliography_sources: Vec<BibliographySource>,
    /// Comment threads keyed by OOXML `w:id` (F17.S3).
    #[serde(default)]
    pub comments: Vec<CommentThread>,
    /// Digital signatures over document content (F22.S4).
    #[serde(default)]
    pub signatures: Vec<DigitalSignature>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            styles: StyleSheet::with_defaults(),
            settings: DocumentSettings::default_settings(),
            properties: DocumentProperties::default(),
            sections: vec![Section::new()],
            footnotes: Vec::new(),
            bibliography_sources: Vec::new(),
            comments: Vec::new(),
            signatures: Vec::new(),
        }
    }

    pub fn with_paragraph(text: impl Into<String>) -> Self {
        let mut doc = Self::new();
        if let Some(section) = doc.sections.first_mut() {
            section.blocks = vec![Block::Paragraph(Paragraph::with_text(text))];
        }
        doc
    }

    /// Build a document from plain text, splitting on blank lines or single newlines.
    pub fn from_plain_text(text: &str) -> Self {
        let mut doc = Self::new();
        if let Some(section) = doc.sections.first_mut() {
            let paragraphs: Vec<&str> = text
                .split('\n')
                .map(str::trim_end)
                .collect();
            section.blocks = if paragraphs.is_empty() || (paragraphs.len() == 1 && paragraphs[0].is_empty()) {
                vec![Block::Paragraph(Paragraph::new())]
            } else {
                paragraphs
                    .into_iter()
                    .map(|p| Block::Paragraph(Paragraph::with_text(p)))
                    .collect()
            };
        }
        doc
    }

    pub fn first_section_mut(&mut self) -> Option<&mut Section> {
        self.sections.first_mut()
    }

    pub fn paragraphs_mut(&mut self) -> impl Iterator<Item = &mut Paragraph> {
        self.sections.iter_mut().flat_map(|s| {
            s.blocks.iter_mut().filter_map(|b| match b {
                Block::Paragraph(p) => Some(p),
                Block::Table(_) | Block::ImageBlock(_) | Block::ShapeBlock(_) => None,
            })
        })
    }

    pub fn find_run_location(&self, run_id: NodeId) -> Option<RunLocation> {
        for (si, section) in self.sections.iter().enumerate() {
            if let Some(loc) = Self::find_run_in_blocks(run_id, si, BlockZone::Body, &section.blocks) {
                return Some(loc);
            }
            for (kind, hf) in &section.headers {
                if let Some(loc) =
                    Self::find_run_in_blocks(run_id, si, BlockZone::Header(*kind), &hf.blocks)
                {
                    return Some(loc);
                }
            }
            for (kind, hf) in &section.footers {
                if let Some(loc) =
                    Self::find_run_in_blocks(run_id, si, BlockZone::Footer(*kind), &hf.blocks)
                {
                    return Some(loc);
                }
            }
        }
        None
    }

    fn find_run_in_blocks(
        run_id: NodeId,
        section_index: usize,
        zone: BlockZone,
        blocks: &[Block],
    ) -> Option<RunLocation> {
        for (bi, block) in blocks.iter().enumerate() {
            match block {
                Block::Paragraph(p) => {
                    for (ri, run) in p.runs.iter().enumerate() {
                        if run.id == run_id {
                            return Some(RunLocation {
                                section_index,
                                zone,
                                block_index: bi,
                                run_index: ri,
                                table_cell: None,
                                shape_paragraph: None,
                            });
                        }
                    }
                }
                Block::ShapeBlock(shape) => {
                    for (pi, para) in shape.paragraphs.iter().enumerate() {
                        for (ri, run) in para.runs.iter().enumerate() {
                            if run.id == run_id {
                                return Some(RunLocation {
                                    section_index,
                                    zone,
                                    block_index: bi,
                                    run_index: ri,
                                    table_cell: None,
                                    shape_paragraph: Some(pi),
                                });
                            }
                        }
                    }
                }
                Block::Table(table) => {
                    if let Some(loc) =
                        Self::find_run_in_table(run_id, section_index, zone, bi, table)
                    {
                        return Some(loc);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn find_run_in_table(
        run_id: NodeId,
        section_index: usize,
        zone: BlockZone,
        block_index: usize,
        table: &Table,
    ) -> Option<RunLocation> {
        for (row, table_row) in table.rows.iter().enumerate() {
            for (cell, table_cell) in table_row.cells.iter().enumerate() {
                for (block_in_cell, cell_block) in table_cell.blocks.iter().enumerate() {
                    match cell_block {
                        Block::Paragraph(p) => {
                            for (ri, run) in p.runs.iter().enumerate() {
                                if run.id == run_id {
                                    return Some(RunLocation {
                                        section_index,
                                        zone,
                                        block_index,
                                        run_index: ri,
                                        table_cell: Some((row, cell, block_in_cell)),
                                        shape_paragraph: None,
                                    });
                                }
                            }
                        }
                        Block::Table(nested) => {
                            if let Some(loc) = Self::find_run_in_table(
                                run_id,
                                section_index,
                                zone,
                                block_index,
                                nested,
                            ) {
                                return Some(loc);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    pub fn find_paragraph_location(&self, para_id: NodeId) -> Option<(usize, usize)> {
        self.find_paragraph_run_location(para_id)
            .map(|loc| (loc.section_index, loc.block_index))
    }

    pub fn find_paragraph_run_location(&self, para_id: NodeId) -> Option<RunLocation> {
        for (si, section) in self.sections.iter().enumerate() {
            if let Some(loc) =
                Self::find_paragraph_in_blocks(para_id, si, BlockZone::Body, &section.blocks)
            {
                return Some(loc);
            }
            for (kind, hf) in &section.headers {
                if let Some(loc) =
                    Self::find_paragraph_in_blocks(para_id, si, BlockZone::Header(*kind), &hf.blocks)
                {
                    return Some(loc);
                }
            }
            for (kind, hf) in &section.footers {
                if let Some(loc) =
                    Self::find_paragraph_in_blocks(para_id, si, BlockZone::Footer(*kind), &hf.blocks)
                {
                    return Some(loc);
                }
            }
        }
        None
    }

    fn find_paragraph_in_blocks(
        para_id: NodeId,
        section_index: usize,
        zone: BlockZone,
        blocks: &[Block],
    ) -> Option<RunLocation> {
        for (bi, block) in blocks.iter().enumerate() {
            match block {
                Block::Paragraph(p) if p.id == para_id => {
                    return Some(RunLocation {
                        section_index,
                        zone,
                        block_index: bi,
                        run_index: 0,
                        table_cell: None,
                        shape_paragraph: None,
                    });
                }
                Block::ShapeBlock(shape) => {
                    if let Some(pi) = shape.paragraphs.iter().position(|p| p.id == para_id) {
                        return Some(RunLocation {
                            section_index,
                            zone,
                            block_index: bi,
                            run_index: 0,
                            table_cell: None,
                            shape_paragraph: Some(pi),
                        });
                    }
                }
                Block::Table(table) => {
                    if let Some(loc) =
                        Self::find_paragraph_in_table(para_id, section_index, zone, bi, table)
                    {
                        return Some(loc);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn find_paragraph_in_table(
        para_id: NodeId,
        section_index: usize,
        zone: BlockZone,
        block_index: usize,
        table: &Table,
    ) -> Option<RunLocation> {
        for (row, table_row) in table.rows.iter().enumerate() {
            for (cell, table_cell) in table_row.cells.iter().enumerate() {
                for (block_in_cell, cell_block) in table_cell.blocks.iter().enumerate() {
                    match cell_block {
                        Block::Paragraph(p) if p.id == para_id => {
                            return Some(RunLocation {
                                section_index,
                                zone,
                                block_index,
                                run_index: 0,
                                table_cell: Some((row, cell, block_in_cell)),
                                shape_paragraph: None,
                            });
                        }
                        Block::Table(nested) => {
                            if let Some(loc) = Self::find_paragraph_in_table(
                                para_id,
                                section_index,
                                zone,
                                block_index,
                                nested,
                            ) {
                                return Some(loc);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    pub fn blocks_at_mut(&mut self, loc: RunLocation) -> Option<&mut Vec<Block>> {
        let section = self.sections.get_mut(loc.section_index)?;
        match loc.zone {
            BlockZone::Body => Some(&mut section.blocks),
            BlockZone::Header(kind) => section.headers.get_mut(&kind).map(|hf| &mut hf.blocks),
            BlockZone::Footer(kind) => section.footers.get_mut(&kind).map(|hf| &mut hf.blocks),
        }
    }

    pub fn paragraph_at_loc(&self, loc: RunLocation) -> Option<&Paragraph> {
        if let Some((row, cell, block_in_cell)) = loc.table_cell {
            let table = self.blocks_at(loc)?.get(loc.block_index)?.table()?;
            return table.rows.get(row)?
                .cells.get(cell)?
                .blocks.get(block_in_cell)?
                .paragraph();
        }
        if let Some(para_index) = loc.shape_paragraph {
            return self
                .blocks_at(loc)?
                .get(loc.block_index)?
                .shape()?
                .paragraphs
                .get(para_index);
        }
        self.blocks_at(loc)?.get(loc.block_index)?.paragraph()
    }

    pub fn paragraph_at_loc_mut(&mut self, loc: RunLocation) -> Option<&mut Paragraph> {
        if let Some((row, cell, block_in_cell)) = loc.table_cell {
            let table = self
                .blocks_at_mut(loc)?
                .get_mut(loc.block_index)?
                .table_mut()?;
            return table.rows.get_mut(row)?
                .cells.get_mut(cell)?
                .blocks.get_mut(block_in_cell)?
                .paragraph_mut();
        }
        if let Some(para_index) = loc.shape_paragraph {
            return self
                .blocks_at_mut(loc)?
                .get_mut(loc.block_index)?
                .shape_mut()?
                .paragraphs
                .get_mut(para_index);
        }
        self.blocks_at_mut(loc)?
            .get_mut(loc.block_index)?
            .paragraph_mut()
    }

    pub fn run_at(&self, loc: RunLocation) -> Option<&Run> {
        self.paragraph_at_loc(loc)?
            .runs
            .get(loc.run_index)
    }

    pub fn blocks_at(&self, loc: RunLocation) -> Option<&Vec<Block>> {
        let section = self.sections.get(loc.section_index)?;
        match loc.zone {
            BlockZone::Body => Some(&section.blocks),
            BlockZone::Header(kind) => section.headers.get(&kind).map(|hf| &hf.blocks),
            BlockZone::Footer(kind) => section.footers.get(&kind).map(|hf| &hf.blocks),
        }
    }

    pub fn paragraph_at_mut(&mut self, si: usize, bi: usize) -> Option<&mut Paragraph> {
        self.sections
            .get_mut(si)?
            .blocks
            .get_mut(bi)?
            .paragraph_mut()
    }

    pub fn find_block_location(&self, block_id: NodeId) -> Option<(usize, usize)> {
        for (si, section) in self.sections.iter().enumerate() {
            for (bi, block) in section.blocks.iter().enumerate() {
                let id = match block {
                    Block::Paragraph(p) => p.id,
                    Block::Table(t) => t.id,
                    Block::ImageBlock(i) => i.id,
                    Block::ShapeBlock(s) => s.id,
                };
                if id == block_id {
                    return Some((si, bi));
                }
            }
        }
        None
    }

    pub fn block_at_mut(&mut self, si: usize, bi: usize) -> Option<&mut Block> {
        self.sections.get_mut(si)?.blocks.get_mut(bi)
    }

    pub fn paragraph_at(&self, si: usize, bi: usize) -> Option<&Paragraph> {
        self.sections.get(si)?.blocks.get(bi)?.paragraph()
    }

    pub fn header_footer_seed_run(
        &self,
        section_index: usize,
        is_header: bool,
        hf_type: HeaderFooterType,
    ) -> Option<NodeId> {
        let source = if is_header {
            self.header_source_section(section_index, hf_type)
        } else {
            self.footer_source_section(section_index, hf_type)
        };
        let section = self.sections.get(source)?;
        let hf = if is_header {
            section.headers.get(&hf_type)?
        } else {
            section.footers.get(&hf_type)?
        };
        hf.blocks.first()?.paragraph()?.runs.first().map(|r| r.id)
    }

    /// Walk link chain to the section that owns header content for `section_index`.
    pub fn header_source_section(
        &self,
        section_index: usize,
        hf_type: HeaderFooterType,
    ) -> usize {
        let mut idx = section_index.min(self.sections.len().saturating_sub(1));
        while idx > 0 && self.sections[idx].header_links.is_linked(hf_type) {
            idx -= 1;
        }
        idx
    }

    /// Walk link chain to the section that owns footer content for `section_index`.
    pub fn footer_source_section(
        &self,
        section_index: usize,
        hf_type: HeaderFooterType,
    ) -> usize {
        let mut idx = section_index.min(self.sections.len().saturating_sub(1));
        while idx > 0 && self.sections[idx].footer_links.is_linked(hf_type) {
            idx -= 1;
        }
        idx
    }

    pub fn resolved_header(
        &self,
        section_index: usize,
        hf_type: HeaderFooterType,
    ) -> Option<&HeaderFooter> {
        let source = self.header_source_section(section_index, hf_type);
        self.sections.get(source)?.resolve_header(hf_type)
    }

    pub fn resolved_footer(
        &self,
        section_index: usize,
        hf_type: HeaderFooterType,
    ) -> Option<&HeaderFooter> {
        let source = self.footer_source_section(section_index, hf_type);
        self.sections.get(source)?.resolve_footer(hf_type)
    }

    pub fn header_footer_linked(
        &self,
        section_index: usize,
        is_header: bool,
        hf_type: HeaderFooterType,
    ) -> bool {
        if section_index == 0 {
            return false;
        }
        let Some(section) = self.sections.get(section_index) else {
            return false;
        };
        if is_header {
            section.header_links.is_linked(hf_type)
        } else {
            section.footer_links.is_linked(hf_type)
        }
    }

    /// Locate a run inside a table cell: `(table_id, row_index, cell_index)`.
    pub fn find_table_cell_for_run(&self, run_id: NodeId) -> Option<(NodeId, usize, usize)> {
        for section in &self.sections {
            if let Some(found) = Self::find_run_in_table_blocks(run_id, &section.blocks) {
                return Some(found);
            }
        }
        None
    }

    fn find_run_in_table_blocks(run_id: NodeId, blocks: &[Block]) -> Option<(NodeId, usize, usize)> {
        for block in blocks {
            if let Block::Table(table) = block {
                if let Some(found) = Self::find_run_in_table_rows(run_id, table) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn find_run_in_table_rows(run_id: NodeId, table: &Table) -> Option<(NodeId, usize, usize)> {
        for (ri, row) in table.rows.iter().enumerate() {
            for (ci, cell) in row.cells.iter().enumerate() {
                if let Some(found) = Self::find_run_in_table_cell_blocks(
                    run_id,
                    table.id,
                    ri,
                    ci,
                    &cell.blocks,
                ) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn find_run_in_table_cell_blocks(
        run_id: NodeId,
        parent_table_id: NodeId,
        row: usize,
        col: usize,
        blocks: &[Block],
    ) -> Option<(NodeId, usize, usize)> {
        for block in blocks {
            match block {
                Block::Paragraph(p) => {
                    for run in &p.runs {
                        if run.id == run_id {
                            return Some((parent_table_id, row, col));
                        }
                    }
                }
                Block::Table(nested) => {
                    if let Some(found) = Self::find_run_in_table_rows(run_id, nested) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn run_by_id(&self, run_id: NodeId) -> Option<&Run> {
        self.find_run_location(run_id)
            .and_then(|loc| self.run_at(loc))
    }

    pub fn footnote_by_id(&self, id: i32) -> Option<&Footnote> {
        self.footnotes.iter().find(|note| note.id == id)
    }

    pub fn footnote_by_id_mut(&mut self, id: i32) -> Option<&mut Footnote> {
        self.footnotes.iter_mut().find(|note| note.id == id)
    }

    /// Next available OOXML footnote id (reserved: -1 separator, 0 continuation).
    pub fn next_footnote_id(&self) -> i32 {
        let max_body = self.footnotes.iter().map(|f| f.id).max().unwrap_or(0);
        let max_ref = self
            .sections
            .iter()
            .flat_map(|section| section.blocks.iter())
            .filter_map(|block| block.paragraph())
            .flat_map(|para| para.runs.iter())
            .filter_map(|run| match &run.content {
                RunContent::FootnoteRef(note) => Some(note.note_id),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        max_body.max(max_ref).max(0) + 1
    }

    /// Assigns display numbers 1, 2, 3… in body paragraph order (F16.S1).
    pub fn renumber_footnotes(&mut self) {
        let mut number = 1u32;
        for section in &mut self.sections {
            Self::renumber_footnotes_in_blocks(&mut section.blocks, &mut number);
        }
    }

    fn renumber_footnotes_in_blocks(blocks: &mut [Block], number: &mut u32) {
        for block in blocks {
            match block {
                Block::Paragraph(para) => {
                    for run in &mut para.runs {
                        if let RunContent::FootnoteRef(note) = &mut run.content {
                            note.display_number = Some(*number);
                            *number += 1;
                        }
                    }
                }
                Block::Table(table) => {
                    for row in &mut table.rows {
                        for cell in &mut row.cells {
                            Self::renumber_footnotes_in_blocks(&mut cell.blocks, number);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn comment_thread_by_id(&self, id: i32) -> Option<&CommentThread> {
        self.comments.iter().find(|thread| thread.comment_id == id)
    }

    pub fn comment_thread_by_id_mut(&mut self, id: i32) -> Option<&mut CommentThread> {
        self.comments.iter_mut().find(|thread| thread.comment_id == id)
    }

    /// Next available OOXML comment id.
    pub fn next_comment_id(&self) -> i32 {
        let max_body = self.comments.iter().map(|c| c.comment_id).max().unwrap_or(-1);
        let max_ref = self
            .sections
            .iter()
            .flat_map(|section| section.blocks.iter())
            .filter_map(|block| block.paragraph())
            .flat_map(|para| para.runs.iter())
            .filter_map(|run| match &run.content {
                RunContent::CommentRef(c) => Some(c.comment_id),
                _ => None,
            })
            .max()
            .unwrap_or(-1);
        max_body.max(max_ref) + 1
    }

    /// Assigns display numbers 1, 2, 3… in body paragraph order (F17.S3).
    pub fn renumber_comments(&mut self) {
        let mut number = 1u32;
        for section in &mut self.sections {
            Self::renumber_comments_in_blocks(&mut section.blocks, &mut number);
        }
    }

    fn renumber_comments_in_blocks(blocks: &mut [Block], number: &mut u32) {
        for block in blocks {
            match block {
                Block::Paragraph(para) => {
                    for run in &mut para.runs {
                        if let RunContent::CommentRef(c) = &mut run.content {
                            c.display_number = Some(*number);
                            *number += 1;
                        }
                    }
                }
                Block::Table(table) => {
                    for row in &mut table.rows {
                        for cell in &mut row.cells {
                            Self::renumber_comments_in_blocks(&mut cell.blocks, number);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
