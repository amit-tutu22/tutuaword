use crate::line::{layout_paragraph, ParagraphFrame};
use crate::types::{TableCellLayout, TableLayout};
use tw_model::{sum_numeric_above, FieldEvalContext, Table};
use tw_shape::{GlyphAtlas, TextShaper};

const CELL_PADDING: f32 = 4.0;
pub const MIN_ROW_HEIGHT: f32 = 18.0;
/// Maximum nested table depth before layout stops recursing (crash guard).
pub const MAX_TABLE_NEST_DEPTH: u32 = 8;

/// A run of table rows placed on one page.
pub struct TableSlice {
    pub layout: TableLayout,
    /// Rows placed by this call, starting at the requested `start_row`.
    pub rows_placed: usize,
}

pub fn layout_table(
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    table: &Table,
    x: f32,
    y: f32,
    max_width: f32,
    default_color: u32,
    tab_interval: f32,
) -> TableLayout {
    layout_table_slice(
        shaper,
        atlas,
        table,
        0,
        x,
        y,
        max_width,
        f32::INFINITY,
        default_color,
        tab_interval,
        0,
    )
    .layout
}

/// Lays out rows from `start_row` that fit within `max_height`.
///
/// At least one row is always placed so pagination makes progress, even when a
/// single row is taller than the page. Row spans are clamped to the slice.
pub fn layout_table_slice(
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    table: &Table,
    start_row: usize,
    x: f32,
    y: f32,
    max_width: f32,
    max_height: f32,
    default_color: u32,
    tab_interval: f32,
    depth: u32,
) -> TableSlice {
    let col_count = column_count(table);
    let col_widths = fit_column_widths(table, col_count, max_width);
    // Prefix sums so a spanning cell advances by its span exactly once.
    let col_offsets = prefix_offsets(&col_widths);

    let row_count = table.rows.len();
    let mut occupied = vec![vec![false; col_count]; row_count];
    let table_width: f32 = col_widths.iter().sum();
    let mut cells: Vec<TableCellLayout> = Vec::new();
    // Row index and span per cell, so heights can be reconciled after all rows
    // in the slice have been measured.
    let mut cell_spans: Vec<(usize, usize)> = Vec::new();
    let mut row_heights = vec![0.0f32; row_count];
    let mut row_y = y;
    let mut rows_placed = 0usize;

    for ri in start_row..row_count {
        let row = &table.rows[ri];
        let mut row_height = row.height.unwrap_or(0.0).max(MIN_ROW_HEIGHT);
        let mut row_cells: Vec<TableCellLayout> = Vec::new();
        let mut row_spans: Vec<(usize, usize)> = Vec::new();
        let mut ci = 0usize;

        while ci < col_count {
            if occupied[ri][ci] {
                ci += 1;
                continue;
            }
            let Some(cell) = row.cells.get(cell_index(row, ci)) else {
                break;
            };

            let colspan = (cell.format.colspan.max(1) as usize).min(col_count - ci);
            let rowspan = (cell.format.rowspan.max(1) as usize).min(row_count - ri);

            for r in ri..ri + rowspan {
                for c in ci..ci + colspan {
                    occupied[r][c] = true;
                }
            }

            let col_x = x + col_offsets[ci];
            let col_w: f32 = col_widths[ci..ci + colspan].iter().sum();
            let text_width = (col_w - CELL_PADDING * 2.0).max(1.0);

            let sum_above = sum_numeric_above(table, ri, ci);
            let field_ctx = FieldEvalContext::for_page(0, 1).with_table_sum_above(sum_above);

            let mut cell_lines = Vec::new();
            let mut nested_tables = Vec::new();
            let mut cursor_y = row_y + CELL_PADDING;
            for block in &cell.blocks {
                match block {
                    tw_model::Block::Paragraph(para) => {
                        let (lines, height) = layout_paragraph(
                            shaper,
                            atlas,
                            para,
                            ParagraphFrame::new(col_x + CELL_PADDING, cursor_y, text_width)
                                .with_tab_interval(tab_interval)
                                .with_field_context(field_ctx),
                            default_color,
                        );
                        cell_lines.extend(lines);
                        cursor_y += height;
                    }
                    tw_model::Block::Table(nested) => {
                        if depth + 1 >= MAX_TABLE_NEST_DEPTH {
                            continue;
                        }
                        let nested_max = (text_width - CELL_PADDING).max(1.0);
                        let remaining = (y + max_height - cursor_y).max(MIN_ROW_HEIGHT);
                        let nested_slice = layout_table_slice(
                            shaper,
                            atlas,
                            nested,
                            0,
                            col_x + CELL_PADDING,
                            cursor_y,
                            nested_max,
                            remaining,
                            default_color,
                            tab_interval,
                            depth + 1,
                        );
                        cursor_y += nested_slice.layout.height + CELL_PADDING;
                        nested_tables.push(nested_slice.layout);
                    }
                    _ => {}
                }
            }
            let content_height = cursor_y - row_y + CELL_PADDING;

            if rowspan == 1 {
                row_height = row_height.max(content_height);
            }

            row_spans.push((ri, rowspan));
            row_cells.push(TableCellLayout {
                x: col_x,
                y: row_y,
                width: col_w,
                height: content_height,
                cell_id: cell.id,
                background: cell.format.background.map(|c| c.to_argb()),
                lines: cell_lines,
                nested_tables,
            });

            ci += colspan;
        }

        // Defer the row to the next page rather than letting it overflow,
        // unless it is the first row of the slice (placed even if oversized;
        // the page clip hides anything past the bottom margin).
        if rows_placed > 0 && row_y + row_height - y > max_height {
            break;
        }

        row_heights[ri] = row_height;
        row_y += row_height;
        rows_placed += 1;
        cells.append(&mut row_cells);
        cell_spans.append(&mut row_spans);
    }

    let placed_end = start_row + rows_placed;
    // Stretch every cell to the band it occupies so borders align, then clip
    // glyph lines that extend past the cell band (wrapped text can exceed the
    // row height estimate before reconciliation).
    for (cell, &(ri, rowspan)) in cells.iter_mut().zip(cell_spans.iter()) {
        cell.height = row_heights[ri..(ri + rowspan).min(placed_end)]
            .iter()
            .sum::<f32>()
            .max(MIN_ROW_HEIGHT);
        let bottom = cell.y + cell.height;
        cell.lines.retain(|line| line.y < bottom + 0.5);
    }

    let mut grid_lines = Vec::new();
    let mut grid_line_colors = Vec::new();
    let default_border = table.format.border.unwrap_or_default();
    for (cell, &(ri, _)) in cells.iter().zip(cell_spans.iter()) {
        let row = &table.rows[ri];
        let cell_idx = row.cells.iter().position(|c| c.id == cell.cell_id);
        let border = cell_idx
            .and_then(|i| row.cells.get(i))
            .and_then(|c| c.format.border)
            .unwrap_or(default_border);
        push_cell_border(
            &mut grid_lines,
            &mut grid_line_colors,
            cell.x,
            cell.y,
            cell.width,
            cell.height,
            border,
        );
    }

    TableSlice {
        layout: TableLayout {
            x,
            y,
            width: table_width,
            height: row_y - y,
            table_id: table.id,
            cells,
            grid_lines,
            grid_line_colors,
        },
        rows_placed,
    }
}

/// Grid column count, preferring the declared grid (`w:tblGrid`) over the
/// widest row, since a row's spans may not sum to the grid width.
fn column_count(table: &Table) -> usize {
    if !table.format.column_widths.is_empty() {
        return table.format.column_widths.len();
    }
    table
        .rows
        .iter()
        .map(|r| {
            r.cells
                .iter()
                .map(|c| c.format.colspan.max(1) as usize)
                .sum::<usize>()
        })
        .max()
        .unwrap_or(1)
        .max(1)
}

/// Maps a grid column index back to the cell that starts at or before it.
fn cell_index(row: &tw_model::TableRow, grid_col: usize) -> usize {
    let mut consumed = 0usize;
    for (i, cell) in row.cells.iter().enumerate() {
        let span = cell.format.colspan.max(1) as usize;
        if grid_col < consumed + span {
            return i;
        }
        consumed += span;
    }
    row.cells.len().saturating_sub(1)
}

/// Author widths scaled to the available column, so a table can never
/// bleed past the page margin.
fn fit_column_widths(table: &Table, col_count: usize, max_width: f32) -> Vec<f32> {
    let mut widths = table.format.column_widths.clone();
    if widths.len() != col_count || widths.iter().any(|w| *w <= 0.0) {
        widths = vec![max_width / col_count as f32; col_count];
    }

    let total: f32 = widths.iter().sum();
    if total > max_width && total > 0.0 {
        let scale = max_width / total;
        for w in widths.iter_mut() {
            *w *= scale;
        }
    }
    widths
}

fn prefix_offsets(widths: &[f32]) -> Vec<f32> {
    let mut offsets = Vec::with_capacity(widths.len());
    let mut acc = 0.0;
    for w in widths {
        offsets.push(acc);
        acc += w;
    }
    offsets
}

fn push_cell_border(
    lines: &mut Vec<f32>,
    colors: &mut Vec<u32>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    border: tw_model::BorderSpec,
) {
    let color = border.color.to_argb();
    for segment in [
        (x, y, x + w, y),
        (x, y, x, y + h),
        (x + w, y, x + w, y + h),
        (x, y + h, x + w, y + h),
    ] {
        lines.push(segment.0);
        lines.push(segment.1);
        lines.push(segment.2);
        lines.push(segment.3);
        colors.push(color);
    }
}
