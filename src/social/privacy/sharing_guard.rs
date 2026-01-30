//! Privacy safeguards for cross-posting and sharing.
//!
//! This module prevents accidental privacy leaks when sharing
//! content from private rooms to public rooms.

use matrix_sdk::ruma::{OwnedUserId, RoomId};

/// Privacy level of content
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrivacyLevel {
    /// Publicly visible (world_readable)
    Public = 0,
    /// Friends only (restricted join)
    Friends = 1,
    /// Close friends (invite only)
    CloseFriends = 2,
    /// Private/DM
    Private = 3,
}

impl PrivacyLevel {
    /// Check if content at this level can be shared to target level
    pub fn can_share_to(&self, target: PrivacyLevel) -> bool {
        // Can only share to equal or more private levels
        target >= *self
    }
}

/// Result of share validation
#[derive(Debug)]
pub enum ShareValidation {
    /// Sharing is allowed
    Allowed,
    /// Sharing is blocked due to privacy leak
    BlockedPrivacyLeak {
        source: PrivacyLevel,
        target: PrivacyLevel,
        message: String,
    },
    /// Sharing requires user confirmation
    RequiresConfirmation { warning: String },
    /// Mentioned users not in target room
    MissingMentions { missing_users: Vec<OwnedUserId> },
}

/// Service for validating share actions
pub struct SharingGuard;

impl SharingGuard {
    /// Validate a share/cross-post action
    #[allow(clippy::too_many_arguments)]
    pub fn validate_share(
        _source_room: &RoomId,
        source_privacy: PrivacyLevel,
        _target_room: &RoomId,
        target_privacy: PrivacyLevel,
        mentioned_users: &[OwnedUserId],
        target_members: &[OwnedUserId],
    ) -> ShareValidation {
        // Warn when sharing from semi-private to public
        // Check this BEFORE general privacy levels, as Friends > Public would otherwise be blocked
        if source_privacy == PrivacyLevel::Friends && target_privacy == PrivacyLevel::Public {
            return ShareValidation::RequiresConfirmation {
                warning: "You are about to share friends-only content publicly. \
                         The original author may not have intended this content \
                         to be shared publicly."
                    .to_string(),
            };
        }

        // Check privacy levels
        if !source_privacy.can_share_to(target_privacy) {
            return ShareValidation::BlockedPrivacyLeak {
                source: source_privacy,
                target: target_privacy,
                message: format!(
                    "Cannot share {} content to {} audience",
                    privacy_level_name(source_privacy),
                    privacy_level_name(target_privacy),
                ),
            };
        }

        // Check if mentioned users are in target room
        let missing: Vec<_> = mentioned_users
            .iter()
            .filter(|u| !target_members.contains(u))
            .cloned()
            .collect();

        if !missing.is_empty() {
            return ShareValidation::MissingMentions {
                missing_users: missing,
            };
        }

        ShareValidation::Allowed
    }

    /// Check if a quote/reply leaks private content
    pub fn validate_quote(
        original_room_privacy: PrivacyLevel,
        reply_room_privacy: PrivacyLevel,
    ) -> ShareValidation {
        if original_room_privacy > reply_room_privacy {
            ShareValidation::RequiresConfirmation {
                warning: "Your reply quotes content from a more private room. \
                         This may expose private information."
                    .to_string(),
            }
        } else {
            ShareValidation::Allowed
        }
    }
}

fn privacy_level_name(level: PrivacyLevel) -> &'static str {
    match level {
        PrivacyLevel::Public => "public",
        PrivacyLevel::Friends => "friends-only",
        PrivacyLevel::CloseFriends => "close friends",
        PrivacyLevel::Private => "private",
    }
}
