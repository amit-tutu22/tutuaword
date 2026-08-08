//! Minimal editable chart dataset (F13.S3).

use serde::{Deserialize, Serialize};

/// One data series in a chart.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartSeries {
    pub name: String,
    pub values: Vec<f64>,
}

/// Category labels plus one or more value series.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartData {
    pub categories: Vec<String>,
    pub series: Vec<ChartSeries>,
}

impl ChartData {
    /// Default bar-chart sample inserted by the editor.
    pub fn sample_bar() -> Self {
        Self {
            categories: vec![
                "Q1".into(),
                "Q2".into(),
                "Q3".into(),
                "Q4".into(),
            ],
            series: vec![ChartSeries {
                name: "Series 1".into(),
                values: vec![10.0, 20.0, 15.0, 25.0],
            }],
        }
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
