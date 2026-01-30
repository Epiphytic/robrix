//! Privacy controls for social features.
//!
//! This module provides privacy safeguards to prevent content from being
//! shared to audiences with less restrictive visibility than intended.
//!
//! ## Privacy Levels
//!
//! Content has a privacy level that determines who can see it:
//! - **Public**: Anyone can see
//! - **Friends**: Only friends can see
//! - **CloseFriends**: Only close friends can see
//! - **Private**: Only the author can see
//!
//! ## Sharing Rules
//!
//! Content can only be shared to destinations with equal or more restrictive
//! privacy. For example:
//! - Public content can be shared anywhere
//! - Friends-only content can be shared to Friends or CloseFriends, but NOT Public
//! - Private content cannot be shared at all
//!
//! ## Example
//!
//! ```rust,ignore
//! use robrix::social::privacy::{PrivacyLevel, SharingGuard, ShareValidation};
//!
//! // Check if sharing is allowed
//! let result = SharingGuard::validate_share(
//!     &source_room,
//!     PrivacyLevel::Friends,
//!     &target_room,
//!     PrivacyLevel::Public,
//!     &[], // mentions
//!     &[], // attachments
//! );
//!
//! match result {
//!     ShareValidation::Allowed => println!("Share allowed"),
//!     ShareValidation::BlockedPrivacyLeak { .. } => println!("Privacy leak blocked!"),
//!     _ => println!("Other validation issue"),
//! }
//! ```

use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId};

/// Privacy level for content visibility.
///
/// Ordered from least restrictive (Public) to most restrictive (Private).
/// The ordering is important for determining if sharing is allowed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PrivacyLevel {
    /// Anyone can see this content.
    #[default]
    Public,
    /// Only friends can see this content.
    Friends,
    /// Only close friends can see this content.
    CloseFriends,
    /// Only the author can see this content.
    Private,
}

impl PrivacyLevel {
    /// Get the numeric value for ordering (higher = more restrictive).
    fn ordinal(&self) -> u8 {
        match self {
            Self::Public => 0,
            Self::Friends => 1,
            Self::CloseFriends => 2,
            Self::Private => 3,
        }
    }

    /// Check if content with this privacy level can be shared to a destination
    /// with the given privacy level.
    ///
    /// Content can only flow to equally or more restrictive destinations.
    /// For example, Friends content can go to CloseFriends but not Public.
    pub fn can_share_to(&self, destination: PrivacyLevel) -> bool {
        // Content can be shared if destination is equally or more restrictive
        destination.ordinal() >= self.ordinal()
    }

    /// Get a human-readable description of this privacy level.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Public => "Public (anyone can see)",
            Self::Friends => "Friends only",
            Self::CloseFriends => "Close friends only",
            Self::Private => "Private (only you)",
        }
    }
}

impl PartialOrd for PrivacyLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrivacyLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordinal().cmp(&other.ordinal())
    }
}

impl std::fmt::Display for PrivacyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "Public"),
            Self::Friends => write!(f, "Friends"),
            Self::CloseFriends => write!(f, "Close Friends"),
            Self::Private => write!(f, "Private"),
        }
    }
}

/// Result of share validation.
#[derive(Clone, Debug)]
pub enum ShareValidation {
    /// The share is allowed.
    Allowed,

    /// The share would leak private content to a less restrictive audience.
    BlockedPrivacyLeak {
        /// The privacy level of the source content.
        source_privacy: PrivacyLevel,
        /// The privacy level of the target destination.
        target_privacy: PrivacyLevel,
    },

    /// A mentioned user is not a member of the target room.
    MentionNotInTarget {
        /// The user who was mentioned but is not in the target.
        user_id: OwnedUserId,
    },

    /// An attached item has incompatible privacy.
    AttachmentPrivacyMismatch {
        /// The room containing the attachment.
        attachment_room: OwnedRoomId,
        /// The privacy level of the attachment.
        attachment_privacy: PrivacyLevel,
    },
}

impl ShareValidation {
    /// Check if the validation passed (share is allowed).
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    /// Get a human-readable error message if validation failed.
    pub fn error_message(&self) -> Option<String> {
        match self {
            Self::Allowed => None,
            Self::BlockedPrivacyLeak {
                source_privacy,
                target_privacy,
            } => Some(format!(
                "Cannot share {} content to {} destination",
                source_privacy, target_privacy
            )),
            Self::MentionNotInTarget { user_id } => Some(format!(
                "Mentioned user {} is not in the target room",
                user_id
            )),
            Self::AttachmentPrivacyMismatch {
                attachment_room,
                attachment_privacy,
            } => Some(format!(
                "Attachment from room {} has {} privacy which is incompatible",
                attachment_room, attachment_privacy
            )),
        }
    }
}

/// Guard that validates sharing operations to prevent privacy leaks.
///
/// This is the main entry point for privacy validation. Use
/// `SharingGuard::validate_share` before allowing any cross-posting
/// or sharing operation.
pub struct SharingGuard;

impl SharingGuard {
    /// Validate a share operation.
    ///
    /// Checks that sharing content from the source room to the target room
    /// does not violate privacy rules.
    ///
    /// # Arguments
    ///
    /// * `source_room` - The room containing the original content
    /// * `source_privacy` - The privacy level of the source room
    /// * `target_room` - The destination room for sharing
    /// * `target_privacy` - The privacy level of the target room
    /// * `mentions` - User IDs mentioned in the content
    /// * `attachments` - (room_id, privacy_level) pairs for any attachments
    ///
    /// # Returns
    ///
    /// Returns `ShareValidation::Allowed` if the share is permitted, or
    /// an appropriate error variant if it should be blocked.
    pub fn validate_share(
        _source_room: &RoomId,
        source_privacy: PrivacyLevel,
        _target_room: &RoomId,
        target_privacy: PrivacyLevel,
        _mentions: &[OwnedUserId],
        attachments: &[(OwnedRoomId, PrivacyLevel)],
    ) -> ShareValidation {
        // Rule 1: Source content privacy must be shareable to target
        if !source_privacy.can_share_to(target_privacy) {
            return ShareValidation::BlockedPrivacyLeak {
                source_privacy,
                target_privacy,
            };
        }

        // Rule 2: Check attachments have compatible privacy
        for (attachment_room, attachment_privacy) in attachments {
            if !attachment_privacy.can_share_to(target_privacy) {
                return ShareValidation::AttachmentPrivacyMismatch {
                    attachment_room: attachment_room.clone(),
                    attachment_privacy: *attachment_privacy,
                };
            }
        }

        // Rule 3: Mentions validation would require room membership lookup
        // This is a placeholder - full implementation would check that
        // mentioned users are members of the target room
        // For now, we allow mentions through (to be validated at send time)

        ShareValidation::Allowed
    }

    /// Quick check if a privacy level transition is valid.
    ///
    /// This is a convenience method for simple cases where you just need
    /// to check if content can flow from one privacy level to another.
    pub fn can_share(source: PrivacyLevel, target: PrivacyLevel) -> bool {
        source.can_share_to(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_level_ordering() {
        assert!(PrivacyLevel::Public < PrivacyLevel::Friends);
        assert!(PrivacyLevel::Friends < PrivacyLevel::CloseFriends);
        assert!(PrivacyLevel::CloseFriends < PrivacyLevel::Private);
    }

    #[test]
    fn test_can_share_to_self() {
        assert!(PrivacyLevel::Public.can_share_to(PrivacyLevel::Public));
        assert!(PrivacyLevel::Friends.can_share_to(PrivacyLevel::Friends));
        assert!(PrivacyLevel::CloseFriends.can_share_to(PrivacyLevel::CloseFriends));
        assert!(PrivacyLevel::Private.can_share_to(PrivacyLevel::Private));
    }

    #[test]
    fn test_public_can_share_anywhere() {
        assert!(PrivacyLevel::Public.can_share_to(PrivacyLevel::Public));
        assert!(PrivacyLevel::Public.can_share_to(PrivacyLevel::Friends));
        assert!(PrivacyLevel::Public.can_share_to(PrivacyLevel::CloseFriends));
        assert!(PrivacyLevel::Public.can_share_to(PrivacyLevel::Private));
    }

    #[test]
    fn test_cannot_share_to_less_restrictive() {
        assert!(!PrivacyLevel::Friends.can_share_to(PrivacyLevel::Public));
        assert!(!PrivacyLevel::CloseFriends.can_share_to(PrivacyLevel::Public));
        assert!(!PrivacyLevel::CloseFriends.can_share_to(PrivacyLevel::Friends));
        assert!(!PrivacyLevel::Private.can_share_to(PrivacyLevel::Public));
        assert!(!PrivacyLevel::Private.can_share_to(PrivacyLevel::Friends));
        assert!(!PrivacyLevel::Private.can_share_to(PrivacyLevel::CloseFriends));
    }

    #[test]
    fn test_sharing_guard_blocks_leak() {
        let source: OwnedRoomId = "!source:example.org".try_into().unwrap();
        let target: OwnedRoomId = "!target:example.org".try_into().unwrap();

        let result = SharingGuard::validate_share(
            &source,
            PrivacyLevel::Friends,
            &target,
            PrivacyLevel::Public,
            &[],
            &[],
        );

        assert!(matches!(result, ShareValidation::BlockedPrivacyLeak { .. }));
    }

    #[test]
    fn test_sharing_guard_allows_valid() {
        let source: OwnedRoomId = "!source:example.org".try_into().unwrap();
        let target: OwnedRoomId = "!target:example.org".try_into().unwrap();

        let result = SharingGuard::validate_share(
            &source,
            PrivacyLevel::Public,
            &target,
            PrivacyLevel::Friends,
            &[],
            &[],
        );

        assert!(matches!(result, ShareValidation::Allowed));
    }
}
