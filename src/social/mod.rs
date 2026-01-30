//! Social media features for Robrix.
//!
//! This module implements the Matrix-based social media architecture,
//! providing profile pages, feeds, friend networks, and events.

use makepad_widgets::*;

pub mod discovery;
pub mod events;
pub mod feed_room;
pub mod friends;
pub mod newsfeed;
pub mod post;
pub mod privacy;
pub mod profile_room;
pub mod reactions;
pub mod widgets;

mod actions;
mod requests;

// Note: actions and requests modules are placeholders for future use.
// Re-exports will be added when the modules have public items.

// Re-export core types from profile_room (Phase 2)
pub use profile_room::{ProfileRoomConfig, ProfileRoomError, ProfileRoomService};

// Re-export profile page widgets (Phase 2)
pub use widgets::profile_page::{LoadedProfile, SocialProfileAction, SocialProfilePage};

// Re-export feed room types (Phase 3)
pub use feed_room::{FeedPrivacy, FeedRoomError, FeedRoomService, UserFeeds};

// Re-export post types (Phase 3)
pub use post::{FeedPost, Post, PostContent, PostError, PostMetadata};

// Re-export reactions types (Phase 3)
pub use reactions::{common_emojis, reactions_for_display, ReactionDisplay, ReactionSummary};

// Re-export widget types (Phase 3)
pub use widgets::feed_view::{FeedState, SocialFeedView, SocialFeedViewAction};
pub use widgets::post_card::{LinkPreviewData, PostCardData, SocialPostCard, SocialPostCardAction};
pub use widgets::post_composer::{AttachedMedia, SocialPostComposer, SocialPostComposerAction};

// Re-export newsfeed types (Phase 4)
pub use newsfeed::{
    create_feed_sync_filter, ContentFilter, FeedAggregator, FeedError, FeedFilterSettings,
    FeedItem, FeedSortOrder,
};

// Re-export privacy types (Phase 7)
pub use privacy::{PrivacyLevel, ShareValidation, SharingGuard};

/// Register all social feature UI components.
pub fn live_design(cx: &mut Cx) {
    // Register all widget designs
    widgets::live_design(cx);
}
