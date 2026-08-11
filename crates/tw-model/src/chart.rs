//! Minimal editable chart dataset (F13.S3).

use serde::{Deserialize, Serialize};

/// Chart geometry style (Word Insert Chart gallery).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChartKind {
    #[default]
    Column,
    Bar,
    Line,
    Pie,
}

impl ChartKind {
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => Self::Bar,
            2 => Self::Line,
            3 => Self::Pie,
            _ => Self::Column,
        }
    }

    pub fn as_i32(self) -> i32 {
        match self {
            Self::Column => 0,
            Self::Bar => 1,
            Self::Line => 2,
            Self::Pie => 3,
        }
    }
}

/// One data series in a chart.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartSeries {
    pub name: String,
    pub values: Vec<f64>,
}

/// Category labels plus one or more value series.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartData {
    #[serde(default)]
    pub kind: ChartKind,
    pub categories: Vec<String>,
    pub series: Vec<ChartSeries>,
}

impl ChartData {
    /// Default sample matching Word's Insert Chart starter data.
    pub fn sample(kind: ChartKind) -> Self {
        Self {
            kind,
            categories: vec![
                "Category 1".into(),
                "Category 2".into(),
                "Category 3".into(),
                "Category 4".into(),
            ],
            series: vec![
                ChartSeries {
                    name: "Series 1".into(),
                    values: vec![4.3, 2.5, 3.5, 4.5],
                },
                ChartSeries {
                    name: "Series 2".into(),
                    values: vec![2.4, 4.4, 1.8, 2.8],
                },
            ],
        }
    }

    /// Default column-chart sample inserted by the editor.
    pub fn sample_bar() -> Self {
        Self::sample(ChartKind::Column)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.categories.is_empty() {
            return Err("chart categories must not be empty");
        }
        if self.series.is_empty() {
            return Err("chart must have at least one series");
        }
        for series in &self.series {
            if series.values.len() != self.categories.len() {
                return Err("series values must match category count");
            }
        }
        Ok(())
    }
}
