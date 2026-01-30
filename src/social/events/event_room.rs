//! Event/gathering room management.
//!
//! This module provides services for creating and managing event rooms,
//! including power level configuration for different event roles.

use matrix_sdk::{
    room::power_levels::RoomPowerLevelChanges,
    ruma::{
        api::client::room::create_room::v3::Request as CreateRoomRequest,
        events::room::join_rules::{JoinRule, RoomJoinRulesEventContent},
        Int, OwnedRoomId, RoomId, UserId,
    },
    Client,
};
use robrix_social_events::event::{EventVisibility, SocialEventEventContent};

/// Power level roles for events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventRole {
    /// Full control (PL 100)
    Creator,
    /// Can edit event, moderate (PL 50)
    CoHost,
    /// Can chat, RSVP, potentially invite (PL 0)
    Guest,
}

impl EventRole {
    /// Get the power level for this role as Int.
    pub fn power_level(&self) -> Int {
        match self {
            Self::Creator => Int::new(100).unwrap(),
            Self::CoHost => Int::new(50).unwrap(),
            Self::Guest => Int::new(0).unwrap(),
        }
    }

    /// Get the power level for this role as i64.
    pub fn power_level_i64(&self) -> i64 {
        match self {
            Self::Creator => 100,
            Self::CoHost => 50,
            Self::Guest => 0,
        }
    }
}

/// Create power level changes configuration for event rooms.
///
/// Returns power level changes that can be applied to a room.
///
/// # Arguments
/// * `guests_can_invite` - Whether guests can invite other users
pub fn event_room_power_levels(guests_can_invite: bool) -> RoomPowerLevelChanges {
    let mut changes = RoomPowerLevelChanges::new();

    // State events require co-host level
    changes.state_default = Some(EventRole::CoHost.power_level_i64());

    // Chat is open to all
    changes.events_default = Some(EventRole::Guest.power_level_i64());

    // Invite permission depends on event settings
    changes.invite = if guests_can_invite {
        Some(EventRole::Guest.power_level_i64())
    } else {
        Some(EventRole::CoHost.power_level_i64())
    };

    // Moderation requires co-host
    changes.kick = Some(EventRole::CoHost.power_level_i64());
    changes.ban = Some(EventRole::CoHost.power_level_i64());
    changes.redact = Some(EventRole::CoHost.power_level_i64());

    changes
}

/// Service for managing event rooms.
pub struct EventRoomService {
    client: Client,
}

impl EventRoomService {
    /// Create a new EventRoomService.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Create a new event room.
    ///
    /// Creates a Matrix room configured for hosting an event, with appropriate
    /// power levels and initial event state.
    ///
    /// # Arguments
    /// * `event_details` - The event content to set as initial state
    /// * `guests_can_invite` - Whether guests can invite other users
    ///
    /// # Errors
    /// Returns an error if the user is not logged in or if room creation fails.
    pub async fn create_event(
        &self,
        event_details: SocialEventEventContent,
        guests_can_invite: bool,
    ) -> Result<OwnedRoomId, EventRoomError> {
        let _user_id = self.client.user_id().ok_or(EventRoomError::NotLoggedIn)?;

        let join_rule = match event_details.visibility {
            EventVisibility::Public => JoinRule::Public,
            EventVisibility::Private => JoinRule::Invite,
        };

        let power_level_changes = event_room_power_levels(guests_can_invite);

        // Create room request with event room configuration
        let mut request = CreateRoomRequest::new();
        request.name = Some(event_details.title.clone());
        if let Some(ref desc) = event_details.description {
            request.topic = Some(desc.clone());
        }

        // Create the room
        let response = self
            .client
            .create_room(request)
            .await
            .map_err(EventRoomError::MatrixError)?;

        let room_id = response.room_id().to_owned();

        // Get the room and set initial state events
        if let Some(room) = self.client.get_room(&room_id) {
            // Send join rules state event
            let join_rules_content = RoomJoinRulesEventContent::new(join_rule);
            room.send_state_event(join_rules_content)
                .await
                .map_err(EventRoomError::MatrixError)?;

            // Apply power levels changes
            room.apply_power_level_changes(power_level_changes)
                .await
                .map_err(EventRoomError::MatrixError)?;

            // Send event details state event
            room.send_state_event(event_details)
                .await
                .map_err(EventRoomError::MatrixError)?;
        }

        Ok(room_id)
    }

    /// Update event details.
    ///
    /// Sends a new event state event to update the event details.
    ///
    /// # Errors
    /// Returns an error if the room is not found or the Matrix API call fails.
    pub async fn update_event(
        &self,
        room_id: &RoomId,
        event_details: SocialEventEventContent,
    ) -> Result<(), EventRoomError> {
        let room = self
            .client
            .get_room(room_id)
            .ok_or(EventRoomError::RoomNotFound)?;

        room.send_state_event(event_details)
            .await
            .map_err(EventRoomError::MatrixError)?;

        Ok(())
    }

    /// Invite a guest to an event.
    ///
    /// # Errors
    /// Returns an error if the room is not found or the invite fails.
    pub async fn invite_guest(
        &self,
        room_id: &RoomId,
        guest: &UserId,
    ) -> Result<(), EventRoomError> {
        let room = self
            .client
            .get_room(room_id)
            .ok_or(EventRoomError::RoomNotFound)?;

        room.invite_user_by_id(guest)
            .await
            .map_err(EventRoomError::MatrixError)?;

        Ok(())
    }

    /// Add a co-host to an event.
    ///
    /// Promotes a user to co-host power level (50), allowing them to
    /// edit event details and moderate the room.
    ///
    /// # Errors
    /// Returns an error if the room is not found or power level update fails.
    pub async fn add_cohost(
        &self,
        room_id: &RoomId,
        cohost: &UserId,
    ) -> Result<(), EventRoomError> {
        let room = self
            .client
            .get_room(room_id)
            .ok_or(EventRoomError::RoomNotFound)?;

        // Update the user's power level to co-host
        room.update_power_levels(vec![(cohost, EventRole::CoHost.power_level())])
            .await
            .map_err(EventRoomError::MatrixError)?;

        Ok(())
    }
}

/// Errors that can occur when working with event rooms.
#[derive(Debug, thiserror::Error)]
pub enum EventRoomError {
    /// User is not logged in to the Matrix client.
    #[error("Not logged in")]
    NotLoggedIn,

    /// The requested room was not found.
    #[error("Room not found")]
    RoomNotFound,

    /// An error occurred in the Matrix SDK.
    #[error("Matrix error: {0}")]
    MatrixError(#[from] matrix_sdk::Error),
}
