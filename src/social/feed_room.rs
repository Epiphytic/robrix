//! Feed room management for user posts.
//!
//! Each user maintains up to three feed rooms:
//! - Public feed: Anyone can read
//! - Friends feed: Only friends can read (restricted join)
//! - Close friends feed: Invite-only

use matrix_sdk::{
    ruma::{
        api::client::room::create_room::v3::Request as CreateRoomRequest,
        events::room::{
            history_visibility::{HistoryVisibility, RoomHistoryVisibilityEventContent},
            join_rules::{JoinRule, RoomJoinRulesEventContent},
        },
        OwnedRoomId, RoomId, UserId,
    },
    Client,
};

/// Feed privacy level.
///
/// Determines who can read posts in a feed room and how users can join.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FeedPrivacy {
    /// Anyone can read, public room directory.
    #[default]
    Public,
    /// Only members of user's friends space can join.
    Friends,
    /// Invite-only for close friends.
    CloseFriends,
}

impl FeedPrivacy {
    /// Get the Matrix join rule for this privacy level.
    ///
    /// # Arguments
    /// * `_friends_space_id` - The room ID of the user's friends space,
    ///   required for `Friends` privacy to set up restricted access.
    ///   Note: Restricted join rules require additional ruma API setup
    ///   that varies by version; for now Friends falls back to Invite.
    pub fn join_rule(&self, _friends_space_id: Option<&RoomId>) -> JoinRule {
        match self {
            Self::Public => JoinRule::Public,
            // TODO: Implement restricted join rules when ruma API stabilizes
            // For now, Friends and CloseFriends both use Invite
            Self::Friends | Self::CloseFriends => JoinRule::Invite,
        }
    }

    /// Get the history visibility for this privacy level.
    ///
    /// Public feeds are world-readable, while friends and close friends
    /// feeds only show history to members (shared).
    pub fn history_visibility(&self) -> HistoryVisibility {
        match self {
            Self::Public => HistoryVisibility::WorldReadable,
            Self::Friends | Self::CloseFriends => HistoryVisibility::Shared,
        }
    }

    /// Get the feed type name for display and room naming.
    pub fn feed_name(&self) -> &'static str {
        match self {
            Self::Public => "Public Feed",
            Self::Friends => "Friends Feed",
            Self::CloseFriends => "Close Friends Feed",
        }
    }

    /// Get the alias suffix for this feed type.
    pub fn alias_suffix(&self) -> &'static str {
        match self {
            Self::Public => "_public",
            Self::Friends => "_friends",
            Self::CloseFriends => "_close",
        }
    }
}

impl std::fmt::Display for FeedPrivacy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.feed_name())
    }
}

/// Collection of a user's feed rooms.
#[derive(Clone, Debug, Default)]
pub struct UserFeeds {
    /// Public feed room ID, if it exists.
    pub public: Option<OwnedRoomId>,
    /// Friends-only feed room ID, if it exists.
    pub friends: Option<OwnedRoomId>,
    /// Close friends feed room ID, if it exists.
    pub close_friends: Option<OwnedRoomId>,
}

impl UserFeeds {
    /// Check if any feed room exists.
    pub fn has_any(&self) -> bool {
        self.public.is_some() || self.friends.is_some() || self.close_friends.is_some()
    }

    /// Get the feed room ID for a given privacy level.
    pub fn get(&self, privacy: FeedPrivacy) -> Option<&OwnedRoomId> {
        match privacy {
            FeedPrivacy::Public => self.public.as_ref(),
            FeedPrivacy::Friends => self.friends.as_ref(),
            FeedPrivacy::CloseFriends => self.close_friends.as_ref(),
        }
    }

    /// Get all existing feed room IDs.
    pub fn all(&self) -> Vec<&OwnedRoomId> {
        [&self.public, &self.friends, &self.close_friends]
            .into_iter()
            .flatten()
            .collect()
    }
}

/// Service for managing feed rooms.
///
/// This service handles the creation, discovery, and management of user feed rooms.
/// Each user can have up to three feed rooms with different privacy levels.
pub struct FeedRoomService {
    client: Client,
}

impl FeedRoomService {
    /// Create a new FeedRoomService.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Create a feed room with the specified privacy level.
    ///
    /// # Arguments
    /// * `privacy` - The privacy level for the feed room
    /// * `friends_space_id` - The user's friends space ID (required for Friends privacy)
    ///
    /// # Errors
    /// Returns an error if the user is not logged in, if a feed room with
    /// the same privacy level already exists, or if the Matrix API call fails.
    pub async fn create_feed_room(
        &self,
        privacy: FeedPrivacy,
        friends_space_id: Option<&RoomId>,
    ) -> Result<OwnedRoomId, FeedRoomError> {
        let user_id = self.client.user_id().ok_or(FeedRoomError::NotLoggedIn)?;

        // Build room creation request with appropriate settings
        let mut request = CreateRoomRequest::new();
        request.name = Some(format!("{}'s {}", user_id.localpart(), privacy.feed_name()));
        request.topic = Some(format!("Social feed room ({}) for {}", privacy, user_id));

        // Create the room
        let response = self
            .client
            .create_room(request)
            .await
            .map_err(FeedRoomError::MatrixError)?;

        let room_id = response.room_id().to_owned();

        // Get the room and configure it
        if let Some(room) = self.client.get_room(&room_id) {
            // Set join rules based on privacy level
            let join_rules = RoomJoinRulesEventContent::new(privacy.join_rule(friends_space_id));
            room.send_state_event(join_rules)
                .await
                .map_err(FeedRoomError::MatrixError)?;

            // Set history visibility
            let history_visibility =
                RoomHistoryVisibilityEventContent::new(privacy.history_visibility());
            room.send_state_event(history_visibility)
                .await
                .map_err(FeedRoomError::MatrixError)?;
        }

        Ok(room_id)
    }

    /// Get all feed rooms for a user.
    ///
    /// This discovers feed rooms by looking for rooms with specific
    /// naming conventions or tags.
    ///
    /// # Note
    /// This is a placeholder implementation. Full discovery would involve
    /// checking room state events or maintaining a mapping in the user's
    /// profile room.
    pub async fn get_user_feeds(&self, _user_id: &UserId) -> Result<UserFeeds, FeedRoomError> {
        // TODO: Implement feed discovery by checking room state or profile room
        // For now, return empty feeds
        Ok(UserFeeds::default())
    }

    /// Get the current user's feed rooms.
    pub async fn get_own_feeds(&self) -> Result<UserFeeds, FeedRoomError> {
        let user_id = self.client.user_id().ok_or(FeedRoomError::NotLoggedIn)?;
        self.get_user_feeds(user_id).await
    }

    /// Join a user's feed room.
    ///
    /// # Arguments
    /// * `room_id` - The feed room to join
    ///
    /// # Errors
    /// Returns an error if the room doesn't exist or the user lacks permission to join.
    pub async fn join_feed(&self, room_id: &RoomId) -> Result<(), FeedRoomError> {
        self.client
            .join_room_by_id(room_id)
            .await
            .map_err(FeedRoomError::MatrixError)?;
        Ok(())
    }

    /// Leave a feed room.
    ///
    /// # Arguments
    /// * `room_id` - The feed room to leave
    pub async fn leave_feed(&self, room_id: &RoomId) -> Result<(), FeedRoomError> {
        if let Some(room) = self.client.get_room(room_id) {
            room.leave().await.map_err(FeedRoomError::MatrixError)?;
        }
        Ok(())
    }
}

/// Errors that can occur when working with feed rooms.
#[derive(Debug, thiserror::Error)]
pub enum FeedRoomError {
    /// User is not logged in to the Matrix client.
    #[error("Not logged in")]
    NotLoggedIn,

    /// A feed room with this privacy level already exists.
    #[error("Feed room already exists for privacy level: {0}")]
    AlreadyExists(FeedPrivacy),

    /// The requested feed room was not found.
    #[error("Feed room not found")]
    FeedNotFound,

    /// User does not have permission to access this feed.
    #[error("Access denied to feed room")]
    AccessDenied,

    /// Invalid feed room configuration.
    #[error("Invalid feed room configuration: {0}")]
    InvalidConfiguration(String),

    /// An error occurred in the Matrix SDK.
    #[error("Matrix error: {0}")]
    MatrixError(#[from] matrix_sdk::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_privacy_ordering() {
        assert!(FeedPrivacy::Public < FeedPrivacy::Friends);
        assert!(FeedPrivacy::Friends < FeedPrivacy::CloseFriends);
    }

    #[test]
    fn test_feed_privacy_display() {
        assert_eq!(FeedPrivacy::Public.to_string(), "Public Feed");
        assert_eq!(FeedPrivacy::Friends.to_string(), "Friends Feed");
        assert_eq!(FeedPrivacy::CloseFriends.to_string(), "Close Friends Feed");
    }

    #[test]
    fn test_user_feeds_has_any() {
        let empty = UserFeeds::default();
        assert!(!empty.has_any());

        let with_public = UserFeeds {
            public: Some("!room:example.org".try_into().unwrap()),
            ..Default::default()
        };
        assert!(with_public.has_any());
    }
}
