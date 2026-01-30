//! UI widgets for social features.
//!
//! This module contains all the Makepad widgets used to build the
//! social media UI, including profile pages, post cards, feed views,
//! and composer widgets.

use makepad_widgets::*;

pub mod feed_view;
pub mod post_card;
pub mod post_composer;
pub mod profile_page;

pub use feed_view::*;
pub use post_card::*;
pub use post_composer::*;
pub use profile_page::*;

/// Register all social widget designs with the Makepad live system.
pub fn live_design(cx: &mut Cx) {
    profile_page::live_design(cx);
    post_composer::live_design(cx);
    post_card::live_design(cx);
    feed_view::live_design(cx);
}
