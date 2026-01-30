//! Social media features for Robrix.
//!
//! This module implements the Matrix-based social media architecture,
//! providing profile pages, feeds, friend networks, and events.

use makepad_widgets::*;

pub mod profile_room;
pub mod feed_room;
pub mod post;
pub mod reactions;
pub mod events;
pub mod friends;
pub mod newsfeed;
pub mod discovery;
pub mod privacy;
pub mod widgets;

mod actions;
mod requests;

pub use actions::*;
pub use requests::*;
pub use profile_room::{ProfileRoomConfig, ProfileRoomError, ProfileRoomService};
pub use widgets::profile_page::{LoadedProfile, SocialProfileAction, SocialProfilePage};

/// Register all social feature UI components.
pub fn live_design(cx: &mut Cx) {
    // Register all widget designs
    widgets::live_design(cx);
}
