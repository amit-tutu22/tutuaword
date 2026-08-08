use chrono::{DateTime, Datelike, Timelike, Utc};

use crate::nodes::{Run, RunContent};
use crate::vocabulary::{FieldData, FieldType};

/// Layout-time context for evaluating dynamic field runs.
#[derive(Debug, Clone, Copy)]
pub struct FieldEvalContext {
    /// One-based page number for the band being laid out.
    pub page_number: u32,
    /// Total pages in the document (best single-pass estimate during layout).
    pub num_pages: u32,
    /// Fixed clock for DATE/TIME fields (tests); `None` uses UTC now.
    pub fixed_datetime: Option<DateTime<Utc>>,
    /// Precomputed `=SUM(ABOVE)` for the active table cell (F09.S5).
    pub table_sum_above: Option<f64>,
}

impl FieldEvalContext {
    pub fn for_page(page_index: u32, num_pages: u32) -> Self {
        Self {
            page_number: page_index.saturating_add(1),
            num_pages: num_pages.max(1),
            fixed_datetime: None,
            table_sum_above: None,
        }
    }

    pub fn with_table_sum_above(mut self, sum: f64) -> Self {
        self.table_sum_above = Some(sum);
        self
    }

    pub fn with_fixed_datetime(mut self, dt: DateTime<Utc>) -> Self {
        self.fixed_datetime = Some(dt);
        self
    }
}

pub fn field_instruction(field_type: &FieldType) -> String {
    match field_type {
        FieldType::Page => " PAGE ".to_string(),
        FieldType::NumPages => " NUMPAGES ".to_string(),
        FieldType::Date => r#" DATE \@ "MMMM d, yyyy" "#.to_string(),
        FieldType::Time => r#" TIME \@ "h:mm am/pm" "#.to_string(),
        FieldType::Filename => " FILENAME ".to_string(),
        FieldType::Author => " AUTHOR ".to_string(),
        FieldType::Title => " TITLE ".to_string(),
        FieldType::CrossRef => " REF ".to_string(),
        FieldType::TableSumAbove => " =SUM(ABOVE) ".to_string(),
        FieldType::Other(instr) => instr.clone(),
    }
}

pub fn evaluate_field(field: &FieldData, ctx: &FieldEvalContext) -> String {
    match field.field_type {
        FieldType::Page => ctx.page_number.to_string(),
        FieldType::NumPages => ctx.num_pages.to_string(),
        FieldType::Date => format_date(ctx),
        FieldType::Time => format_time(ctx),
        FieldType::Filename | FieldType::Author | FieldType::Title |         FieldType::CrossRef => field
            .display_text
            .clone()
            .unwrap_or_else(|| "[field]".to_string()),
        FieldType::TableSumAbove => ctx
            .table_sum_above
            .map(format_sum)
            .unwrap_or_else(|| "0".to_string()),
        FieldType::Other(_) => field
            .display_text
            .clone()
            .unwrap_or_else(|| "[field]".to_string()),
    }
}

pub fn run_layout_text(run: &Run, ctx: Option<&FieldEvalContext>) -> String {
    match &run.content {
        RunContent::Field(field) => ctx
            .map(|c| evaluate_field(field, c))
            .unwrap_or_else(|| {
                field
                    .display_text
                    .clone()
                    .unwrap_or_else(|| "[field]".to_string())
            }),
        _ => run.text().to_string(),
    }
}

pub fn paragraph_layout_text(para: &crate::Paragraph, ctx: Option<&FieldEvalContext>) -> String {
    para.runs
        .iter()
        .filter(|r| r.format.hidden != Some(true))
        .map(|r| run_layout_text(r, ctx))
        .collect()
}

fn format_date(ctx: &FieldEvalContext) -> String {
    let dt = ctx.fixed_datetime.unwrap_or_else(Utc::now);
    let month = dt.format("%B").to_string();
    format!("{} {}, {}", month, dt.day(), dt.year())
}

fn format_time(ctx: &FieldEvalContext) -> String {
    let dt = ctx.fixed_datetime.unwrap_or_else(Utc::now);
    let hour = dt.hour();
    let minute = dt.minute();
    let (h12, ampm) = if hour == 0 {
        (12, "am")
    } else if hour < 12 {
        (hour, "am")
    } else if hour == 12 {
        (12, "pm")
    } else {
        (hour - 12, "pm")
    };
    format!("{h12}:{minute:02} {ampm}")
}

fn format_sum(value: f64) -> String {
    if (value - value.round()).abs() < f64::EPSILON {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.2}")
    }
}
