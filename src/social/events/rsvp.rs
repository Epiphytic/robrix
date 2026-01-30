//! RSVP system for events.
//!
//! SECURITY: This module includes critical validation to prevent
//! RSVP spoofing attacks.

use matrix_sdk::{
    ruma::{events::AnySyncStateEvent, OwnedEventId, OwnedUserId, RoomId, UserId},
    Client,
};
use robrix_social_events::rsvp::{RsvpStatus, SocialRsvpEventContent};

/// RSVP validation result.
#[derive(Debug)]
pub enum RsvpValidation {
    /// The RSVP is valid.
    Valid,
    /// state_key doesn't match sender - potential spoofing attempt.
    SenderMismatch {
        /// The user ID claimed in the state_key.
        claimed: OwnedUserId,
        /// The actual sender of the event.
        actual: OwnedUserId,
    },
    /// Invalid RSVP content.
    InvalidContent(String),
}

/// Validate an RSVP event for security.
///
/// CRITICAL: Matrix does NOT enforce that state_key matches sender.
/// Clients MUST perform this validation to prevent impersonation.
///
/// # Arguments
/// * `event` - The sync state event to validate
/// * `sender` - The actual sender of the event
///
/// # Returns
/// Returns `RsvpValidation::Valid` if the event is valid, or an error variant
/// describing the validation failure.
pub fn validate_rsvp_event(event: &AnySyncStateEvent, sender: &UserId) -> RsvpValidation {
    // For org.social.rsvp events, state_key must equal sender
    if event.event_type().to_string() == "org.social.rsvp" {
        let state_key = event.state_key();

        // Parse state_key as user ID
        match OwnedUserId::try_from(state_key.to_string()) {
            Ok(claimed_user) => {
                if claimed_user.as_str() != sender.as_str() {
                    return RsvpValidation::SenderMismatch {
                        claimed: claimed_user,
                        actual: sender.to_owned(),
                    };
                }
            }
            Err(_) => {
                return RsvpValidation::InvalidContent(format!("Invalid state_key: {}", state_key));
            }
        }
    }

    RsvpValidation::Valid
}

/// Service for managing RSVPs.
pub struct RsvpService {
    client: Client,
}

impl RsvpService {
    /// Create a new RsvpService.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Set the current user's RSVP for an event.
    ///
    /// # Arguments
    /// * `room_id` - The event room ID
    /// * `status` - The RSVP status (Going, Interested, NotGoing)
    /// * `guests` - Number of guests including the user
    /// * `note` - Optional note (e.g., "Bringing potato salad!")
    ///
    /// # Errors
    /// Returns an error if the user is not logged in, room not found, or API fails.
    pub async fn set_rsvp(
        &self,
        room_id: &RoomId,
        status: RsvpStatus,
        guests: u32,
        note: Option<String>,
    ) -> Result<OwnedEventId, RsvpError> {
        let user_id = self.client.user_id().ok_or(RsvpError::NotLoggedIn)?;

        let room = self
            .client
            .get_room(room_id)
            .ok_or(RsvpError::RoomNotFound)?;

        let content = SocialRsvpEventContent {
            status,
            guests,
            note,
        };

        // Send state event with our user ID as state_key
        let response = room
            .send_state_event_for_key(user_id, content)
            .await
            .map_err(RsvpError::MatrixError)?;

        Ok(response.event_id)
    }

    /// Submit an RSVP (alias for set_rsvp for API compatibility).
    pub async fn submit_rsvp(
        &self,
        room_id: &RoomId,
        status: RsvpStatus,
        guests: u32,
        note: Option<String>,
    ) -> Result<OwnedEventId, RsvpError> {
        self.set_rsvp(room_id, status, guests, note).await
    }

    /// Get all RSVPs for an event.
    ///
    /// Returns a list of validated RSVPs. Invalid RSVPs (e.g., spoofed) are filtered out.
    ///
    /// # Errors
    /// Returns an error if the room is not found.
    pub async fn get_rsvps(&self, room_id: &RoomId) -> Result<Vec<ValidatedRsvp>, RsvpError> {
        let _room = self
            .client
            .get_room(room_id)
            .ok_or(RsvpError::RoomNotFound)?;

        // TODO: Implement RSVP retrieval with validation
        // This would involve:
        // 1. Fetching all org.social.rsvp state events from the room
        // 2. Validating each event (state_key == sender)
        // 3. Filtering out invalid events
        // 4. Converting valid events to ValidatedRsvp structs
        //
        // For now, return empty list until state event retrieval is implemented
        Ok(Vec::new())
    }

    /// Get aggregated RSVP counts.
    ///
    /// # Errors
    /// Returns an error if RSVP retrieval fails.
    pub async fn get_rsvp_counts(&self, room_id: &RoomId) -> Result<RsvpCounts, RsvpError> {
        let rsvps = self.get_rsvps(room_id).await?;

        let mut counts = RsvpCounts::default();
        for rsvp in rsvps {
            match rsvp.status {
                RsvpStatus::Going => {
                    counts.going += 1;
                    counts.total_guests += rsvp.guests;
                }
                RsvpStatus::Interested => counts.interested += 1,
                RsvpStatus::NotGoing => counts.not_going += 1,
            }
        }

        Ok(counts)
    }
}

/// A validated RSVP (passed security checks).
#[derive(Clone, Debug)]
pub struct ValidatedRsvp {
    /// The user who submitted the RSVP.
    pub user_id: OwnedUserId,
    /// The RSVP status.
    pub status: RsvpStatus,
    /// Number of guests (including the user).
    pub guests: u32,
    /// Optional note.
    pub note: Option<String>,
}

/// Aggregated RSVP counts.
#[derive(Clone, Debug, Default)]
pub struct RsvpCounts {
    /// Number of users going.
    pub going: u32,
    /// Number of users interested.
    pub interested: u32,
    /// Number of users not going.
    pub not_going: u32,
    /// Total guests (including +1s).
    pub total_guests: u32,
}

/// Errors that can occur when working with RSVPs.
#[derive(Debug, thiserror::Error)]
pub enum RsvpError {
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
