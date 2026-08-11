use serde::{Deserialize, Serialize};

/// Declared / granted plugin capabilities (F26.S3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    DocumentRead,
    DocumentEdit,
    DocumentSuggest,
    UiSidebar,
    UiContextMenu,
    UiToolbar,
    UiDialog,
    EventsDocument,
    EventsSelection,
    Network,
    Storage,
    FilesystemRead,
    AiProvider,
}

impl Capability {
    /// Stable integer code for WASM host imports.
    pub fn code(self) -> i32 {
        match self {
            Self::DocumentRead => 0,
            Self::DocumentEdit => 1,
            Self::DocumentSuggest => 2,
            Self::UiSidebar => 3,
            Self::UiContextMenu => 4,
            Self::UiToolbar => 5,
            Self::UiDialog => 6,
            Self::EventsDocument => 7,
            Self::EventsSelection => 8,
            Self::Network => 9,
            Self::Storage => 10,
            Self::FilesystemRead => 11,
            Self::AiProvider => 12,
        }
    }

    pub fn from_code(code: i32) -> Option<Self> {
        Some(match code {
            0 => Self::DocumentRead,
            1 => Self::DocumentEdit,
            2 => Self::DocumentSuggest,
            3 => Self::UiSidebar,
            4 => Self::UiContextMenu,
            5 => Self::UiToolbar,
            6 => Self::UiDialog,
            7 => Self::EventsDocument,
            8 => Self::EventsSelection,
            9 => Self::Network,
            10 => Self::Storage,
            11 => Self::FilesystemRead,
            12 => Self::AiProvider,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DocumentRead => "document.read",
            Self::DocumentEdit => "document.edit",
            Self::DocumentSuggest => "document.suggest",
            Self::UiSidebar => "ui.sidebar",
            Self::UiContextMenu => "ui.context_menu",
            Self::UiToolbar => "ui.toolbar",
            Self::UiDialog => "ui.dialog",
            Self::EventsDocument => "events.document",
            Self::EventsSelection => "events.selection",
            Self::Network => "network",
            Self::Storage => "storage",
            Self::FilesystemRead => "filesystem.read",
            Self::AiProvider => "ai.provider",
        }
    }
}

/// Intersect requested capabilities with what the user granted.
pub fn grant_capabilities(
    requested: &[Capability],
    granted: &[Capability],
) -> Vec<Capability> {
    requested
        .iter()
        .copied()
        .filter(|c| granted.contains(c))
        .collect()
}
