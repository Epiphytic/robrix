//! UI widgets for social features.

use makepad_widgets::*;

pub mod friend_list;
pub mod profile_page;

pub use friend_list::*;
pub use profile_page::*;

/// Register all social widget designs.
pub fn live_design(cx: &mut Cx) {
    friend_list::live_design(cx);
    profile_page::live_design(cx);
}
