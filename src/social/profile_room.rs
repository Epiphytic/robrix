//! Profile room management for social profiles.
//!
//! Each user has a dedicated "profile room" that stores their extended
//! profile information as state events. This is the core service that
//! manages the creation, discovery, and updating of user profile rooms.

use matrix_sdk::{
    ruma::{
        api::client::room::create_room::v3::Request as CreateRoomRequest,
        events::room::{
            join_rules::{JoinRule, RoomJoinRulesEventContent},
            history_visibility::{HistoryVisibility, RoomHistoryVisibilityEventContent},
        },
        OwnedRoomAliasId, OwnedRoomId, RoomId, UserId,
    },
    Client,
};
use robrix_social_events::profile::SocialProfileEventContent;

/// Profile room configuration
pub struct ProfileRoomConfig {
    /// Room alias format: #profile_{localpart}:{server}
    pub alias_prefix: &'static str,
    /// Default join rules for profile rooms
    pub default_join_rule: JoinRule,
    /// Default history visibility
    pub default_history_visibility: HistoryVisibility,
}

impl Default for ProfileRoomConfig {
    fn default() -> Self {
        Self {
            alias_prefix: "profile_",
            default_join_rule: JoinRule::Public,
            default_history_visibility: HistoryVisibility::WorldReadable,
        }
    }
}

/// Service for managing user profile rooms
pub struct ProfileRoomService {
    client: Client,
    config: ProfileRoomConfig,
}

impl ProfileRoomService {
    /// Create a new ProfileRoomService with default configuration.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            config: ProfileRoomConfig::default(),
        }
    }

    /// Create a new ProfileRoomService with custom configuration.
    pub fn with_config(client: Client, config: ProfileRoomConfig) -> Self {
        Self { client, config }
    }

    /// Create a profile room for the current user.
    ///
    /// This creates a new Matrix room configured as a profile room,
    /// with the initial profile content set as a state event.
    ///
    /// # Errors
    /// Returns an error if the user is not logged in, if a profile room
    /// already exists, or if the Matrix API call fails.
    pub async fn create_profile_room(
        &self,
        initial_profile: SocialProfileEventContent,
    ) -> Result<OwnedRoomId, ProfileRoomError> {
        let user_id = self.client.user_id().ok_or(ProfileRoomError::NotLoggedIn)?;

        let _alias = self.profile_alias_for_user(user_id)?;

        // Check if room already exists
        if let Some(room_id) = self.find_profile_room(user_id).await? {
            return Err(ProfileRoomError::AlreadyExists(room_id));
        }

        // Create room request with profile room configuration
        let mut request = CreateRoomRequest::new();
        request.name = Some(format!("{}'s Profile", user_id.localpart()));
        request.topic = Some("Social profile room".to_string());

        // Set initial state events for join rules and history visibility
        let join_rules = RoomJoinRulesEventContent::new(self.config.default_join_rule.clone());
        let history_visibility =
            RoomHistoryVisibilityEventContent::new(self.config.default_history_visibility.clone());

        // Note: Setting initial state events and room alias requires
        // building the full request with proper event serialization.
        // For now, we create a basic room and update it.
        let response = self
            .client
            .create_room(request)
            .await
            .map_err(ProfileRoomError::MatrixError)?;

        let room_id = response.room_id().to_owned();

        // Get the room and set the initial profile state
        if let Some(room) = self.client.get_room(&room_id) {
            // Send join rules state event
            room.send_state_event(join_rules)
                .await
                .map_err(ProfileRoomError::MatrixError)?;

            // Send history visibility state event
            room.send_state_event(history_visibility)
                .await
                .map_err(ProfileRoomError::MatrixError)?;

            // Send initial profile state event
            room.send_state_event(initial_profile)
                .await
                .map_err(ProfileRoomError::MatrixError)?;
        }

        Ok(room_id)
    }

    /// Find a user's profile room by alias.
    ///
    /// Attempts to resolve the profile room alias for the given user.
    /// Returns `None` if the room doesn't exist.
    pub async fn find_profile_room(
        &self,
        user_id: &UserId,
    ) -> Result<Option<OwnedRoomId>, ProfileRoomError> {
        let alias = self.profile_alias_for_user(user_id)?;

        match self.client.resolve_room_alias(&alias).await {
            Ok(response) => Ok(Some(response.room_id)),
            Err(e) => {
                // Check if the error indicates the room wasn't found
                let error_str = e.to_string();
                if error_str.contains("not found") || error_str.contains("M_NOT_FOUND") {
                    Ok(None)
                } else {
                    Err(ProfileRoomError::MatrixError(e.into()))
                }
            }
        }
    }

    /// Update the profile in an existing profile room.
    ///
    /// Sends a new profile state event to the room, replacing the previous one.
    pub async fn update_profile(
        &self,
        room_id: &RoomId,
        profile: SocialProfileEventContent,
    ) -> Result<(), ProfileRoomError> {
        let room = self
            .client
            .get_room(room_id)
            .ok_or(ProfileRoomError::RoomNotFound)?;

        room.send_state_event(profile)
            .await
            .map_err(ProfileRoomError::MatrixError)?;

        Ok(())
    }

    /// Get the profile from a profile room.
    ///
    /// Retrieves the current profile state event from the given room.
    /// Note: This is a placeholder implementation for Phase 2.
    /// Full state event retrieval will be implemented in a later phase.
    pub async fn get_profile(
        &self,
        room_id: &RoomId,
    ) -> Result<Option<SocialProfileEventContent>, ProfileRoomError> {
        let _room = self
            .client
            .get_room(room_id)
            .ok_or(ProfileRoomError::RoomNotFound)?;

        // TODO: Implement state event retrieval once we have the proper
        // ruma event types wired up. For now, return None.
        // The full implementation would use:
        // room.get_state_event_static::<SocialProfileEventContent>()
        Ok(None)
    }

    /// Get profile alias for a user.
    ///
    /// Constructs the canonical room alias for a user's profile room
    /// in the format: #profile_{localpart}:{server}
    fn profile_alias_for_user(
        &self,
        user_id: &UserId,
    ) -> Result<OwnedRoomAliasId, ProfileRoomError> {
        let localpart = user_id.localpart();
        let server = user_id.server_name();
        let alias = format!("#{}{}:{}", self.config.alias_prefix, localpart, server);
        alias.try_into().map_err(|_| ProfileRoomError::InvalidAlias)
    }

    /// Get the current user's profile room, creating it if it doesn't exist.
    ///
    /// This is a convenience method that combines `find_profile_room` and
    /// `create_profile_room`.
    pub async fn get_or_create_profile_room(
        &self,
        initial_profile: SocialProfileEventContent,
    ) -> Result<OwnedRoomId, ProfileRoomError> {
        let user_id = self.client.user_id().ok_or(ProfileRoomError::NotLoggedIn)?;

        if let Some(room_id) = self.find_profile_room(user_id).await? {
            Ok(room_id)
        } else {
            self.create_profile_room(initial_profile).await
        }
    }
}

/// Errors that can occur when working with profile rooms.
#[derive(Debug, thiserror::Error)]
pub enum ProfileRoomError {
    /// User is not logged in to the Matrix client.
    #[error("Not logged in")]
    NotLoggedIn,

    /// A profile room already exists for this user.
    #[error("Profile room already exists: {0}")]
    AlreadyExists(OwnedRoomId),

    /// The requested room was not found.
    #[error("Room not found")]
    RoomNotFound,

    /// The constructed room alias was invalid.
    #[error("Invalid room alias")]
    InvalidAlias,

    /// An error occurred in the Matrix SDK.
    #[error("Matrix error: {0}")]
    MatrixError(#[from] matrix_sdk::Error),
}
