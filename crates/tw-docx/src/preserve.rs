//! Token-preserving paragraph XML retained from import for unedited blocks (R3.3).

use std::collections::HashMap;

use tw_model::NodeId;

/// Raw `w:p` element from the source package, keyed by paragraph [`NodeId`].
#[derive(Debug, Clone)]
pub struct PreservedParagraph {
    pub xml: String,
    pub fingerprint: u64,
}

pub type PreservedParagraphMap = HashMap<NodeId, PreservedParagraph>;

/// Raw imported `w:p` for a block-level drawing shape (F11.S1).
pub type PreservedShapeMap = HashMap<NodeId, PreservedParagraph>;
