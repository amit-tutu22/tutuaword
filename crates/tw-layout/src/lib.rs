mod engine;
mod line;
mod tables;
mod types;

pub use engine::*;
pub use line::*;
pub use tables::*;
pub use types::*;

/// Re-exported so a host can register fonts through `LayoutEngine` without
/// depending on `tw-shape` directly.
pub use tw_shape::{
    FontDatabase, FontFaceSpec, FontId, FontRegistrationError, TextShaper, WEIGHT_BOLD,
    WEIGHT_REGULAR,
};
