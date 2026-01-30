//! UI widgets for social features.

use makepad_widgets::*;

pub mod event_card;
pub mod friend_list;
pub mod profile_page;

pub use event_card::*;
pub use friend_list::*;
pub use profile_page::*;

/// Register all social widget designs.
pub fn live_design(cx: &mut Cx) {
    event_card::live_design(cx);
    friend_list::live_design(cx);
    profile_page::live_design(cx);
}
