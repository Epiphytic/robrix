//! Friends space management for social graph.
//!
//! Each user maintains a private space containing their friends' feed rooms.
//! This enables organizing friends and provides the foundation for the
//! friends-only visibility feed tier.

use matrix_sdk::{
    ruma::{
        api::client::room::create_room::v3::Request as CreateRoomRequest,
        events::{
            room::{
                history_visibility::{HistoryVisibility, RoomHistoryVisibilityEventContent},
                join_rules::{JoinRule, RoomJoinRulesEventContent},
            },
            space::child::SpaceChildEventContent,
        },
        OwnedRoomId, RoomId, UserId,
    },
    Client,
};

/// Service for managing the friends space
pub struct FriendsSpaceService {
    client: Client,
    /// The user's friends space room ID (cached after discovery)
    space_id: Option<OwnedRoomId>,
}

impl FriendsSpaceService {
    /// Create a new FriendsSpaceService.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            space_id: None,
        }
    }

    /// Create or get the user's friends space.
    ///
    /// This method first checks the cached space ID, then searches for an
    /// existing friends space, and finally creates a new one if needed.
    pub async fn get_or_create_friends_space(&mut self) -> Result<OwnedRoomId, FriendsError> {
        // Return cached ID if available
        if let Some(id) = &self.space_id {
            return Ok(id.clone());
        }

        // Try to find existing friends space
        if let Some(id) = self.find_friends_space().await? {
            self.space_id = Some(id.clone());
            return Ok(id);
        }

        // Create new friends space
        let id = self.create_friends_space().await?;
        self.space_id = Some(id.clone());
        Ok(id)
    }

    /// Add a friend's feed room to the space.
    ///
    /// This creates a space child relationship between the friends space
    /// and the friend's feed room, effectively adding them to the friends list.
    pub async fn add_friend(&mut self, friend_feed_room: &RoomId) -> Result<(), FriendsError> {
        let space_id = self.get_or_create_friends_space().await?;
        let space = self
            .client
            .get_room(&space_id)
            .ok_or(FriendsError::SpaceNotFound)?;

        // Add as space child with empty server list (Matrix handles routing)
        let content = SpaceChildEventContent::new(vec![]);
        space
            .send_state_event_for_key(friend_feed_room, content)
            .await
            .map_err(FriendsError::MatrixError)?;

        Ok(())
    }

    /// Remove a friend from the space.
    ///
    /// This removes the space child relationship, effectively unfriending the user.
    /// The friend's feed room itself is not affected.
    pub async fn remove_friend(&mut self, friend_feed_room: &RoomId) -> Result<(), FriendsError> {
        let space_id = self.get_or_create_friends_space().await?;
        let space = self
            .client
            .get_room(&space_id)
            .ok_or(FriendsError::SpaceNotFound)?;

        // Remove space child by sending content with empty via list
        // Setting via to empty effectively removes the relationship
        let mut content = SpaceChildEventContent::new(vec![]);
        // Mark as not suggested to indicate removal intent
        content.suggested = false;

        // Send state event with the room ID as key - empty content removes the child
        space
            .send_state_event_for_key(friend_feed_room, content)
            .await
            .map_err(FriendsError::MatrixError)?;

        Ok(())
    }

    /// Get list of friends (feed room IDs in the space).
    ///
    /// Returns the room IDs of all friend feed rooms in the friends space.
    pub async fn get_friends(&self) -> Result<Vec<OwnedRoomId>, FriendsError> {
        let space_id = self.space_id.as_ref().ok_or(FriendsError::SpaceNotFound)?;

        let _space = self
            .client
            .get_room(space_id)
            .ok_or(FriendsError::SpaceNotFound)?;

        // Get space children from the room's space info
        // For now, return an empty list - full implementation requires
        // iterating over m.space.child state events
        let friends = Vec::new();

        // TODO: Implement full space children retrieval using:
        // space.get_state_events_static::<SpaceChildEventContent>()
        // This would iterate through m.space.child state events
        // and extract the room IDs from the state keys

        Ok(friends)
    }

    /// Check if a user is a friend (bidirectional membership check).
    ///
    /// True friendship requires mutual membership - both users must have
    /// each other in their friends space for the relationship to be considered mutual.
    pub async fn is_mutual_friend(&self, _user_id: &UserId) -> Result<bool, FriendsError> {
        // First check if we have them in our friends space
        let friends = self.get_friends().await?;

        // Look for a room owned by the user in our friends list
        // This is a simplified check - full implementation would verify
        // the bidirectional relationship
        for friend_room_id in friends {
            if let Some(_room) = self.client.get_room(&friend_room_id) {
                // TODO: Check if the room creator matches the user
                // This would require fetching room state and comparing
                // with the target user_id
            }
        }

        // TODO: Implement proper bidirectional friendship check
        // For now, return false - full implementation pending
        Ok(false)
    }

    /// Find an existing friends space for the current user.
    async fn find_friends_space(&self) -> Result<Option<OwnedRoomId>, FriendsError> {
        let user_id = self.client.user_id().ok_or(FriendsError::NotLoggedIn)?;

        // Search through joined rooms for a space with the friends tag
        for room in self.client.joined_rooms() {
            // Check if this is a space
            if !room.is_space() {
                continue;
            }

            // Check if the room name matches our convention
            let room_name = room.name();
            let expected_name = format!("{}'s Friends", user_id.localpart());
            if room_name.as_deref() == Some(&expected_name) {
                return Ok(Some(room.room_id().to_owned()));
            }

            // Alternative: check for the friends space tag
            // This would require fetching room tags
        }

        Ok(None)
    }

    /// Create a new friends space for the current user.
    async fn create_friends_space(&self) -> Result<OwnedRoomId, FriendsError> {
        let user_id = self.client.user_id().ok_or(FriendsError::NotLoggedIn)?;

        // Create private space request
        let mut request = CreateRoomRequest::new();
        request.name = Some(format!("{}'s Friends", user_id.localpart()));
        request.topic = Some("Friends space for organizing connections".to_string());

        // Mark as a space by setting the room type
        // Note: This requires setting creation_content with room_type = "m.space"
        // For now, we create a regular room and configure it

        let response = self
            .client
            .create_room(request)
            .await
            .map_err(FriendsError::MatrixError)?;

        let room_id = response.room_id().to_owned();

        // Configure the room as a private friends space
        if let Some(room) = self.client.get_room(&room_id) {
            // Set join rules to invite-only
            let join_rules = RoomJoinRulesEventContent::new(JoinRule::Invite);
            room.send_state_event(join_rules)
                .await
                .map_err(FriendsError::MatrixError)?;

            // Set history visibility to joined members only
            let history_visibility =
                RoomHistoryVisibilityEventContent::new(HistoryVisibility::Joined);
            room.send_state_event(history_visibility)
                .await
                .map_err(FriendsError::MatrixError)?;
        }

        Ok(room_id)
    }

    /// Get the cached space ID without triggering creation.
    pub fn cached_space_id(&self) -> Option<&OwnedRoomId> {
        self.space_id.as_ref()
    }

    /// Clear the cached space ID, forcing rediscovery on next access.
    pub fn clear_cache(&mut self) {
        self.space_id = None;
    }
}

/// Errors that can occur when working with friend spaces.
#[derive(Debug, thiserror::Error)]
pub enum FriendsError {
    /// User is not logged in to the Matrix client.
    #[error("Not logged in")]
    NotLoggedIn,

    /// The friends space was not found.
    #[error("Friends space not found")]
    SpaceNotFound,

    /// The friend's feed room was not found.
    #[error("Friend's feed room not found")]
    FeedRoomNotFound,

    /// The user is already a friend.
    #[error("User is already a friend")]
    AlreadyFriend,

    /// The user is not a friend.
    #[error("User is not a friend")]
    NotFriend,

    /// An error occurred in the Matrix SDK.
    #[error("Matrix error: {0}")]
    MatrixError(#[from] matrix_sdk::Error),
}
