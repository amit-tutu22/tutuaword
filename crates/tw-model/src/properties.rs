use serde::{Deserialize, Serialize};

/// Office core/app document metadata (from `docProps/core.xml`, `docProps/app.xml`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentProperties {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_count: Option<u32>,
}

impl DocumentProperties {
    pub fn merge(&mut self, other: DocumentProperties) {
        if self.title.is_none() {
            self.title = other.title;
        }
        if self.author.is_none() {
            self.author = other.author;
        }
        if self.page_count.is_none() {
            self.page_count = other.page_count;
        }
    }
}
