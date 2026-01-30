//! UI widgets for social features.

use makepad_widgets::*;

pub mod profile_page;

pub use profile_page::*;

/// Register all social widget designs.
pub fn live_design(cx: &mut Cx) {
    profile_page::live_design(cx);
}
