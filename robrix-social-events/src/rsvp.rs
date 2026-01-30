use ruma::events::macros::EventContent;
use serde::{Deserialize, Serialize};

/// RSVP status for an event.
/// Event type: `org.social.rsvp`
///
/// SECURITY: The state_key MUST equal the sender's user ID.
/// Clients MUST validate this and ignore events where they don't match.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "org.social.rsvp", kind = State, state_key_type = ruma::OwnedUserId)]
#[serde(deny_unknown_fields)]
pub struct SocialRsvpEventContent {
    /// RSVP status
    pub status: RsvpStatus,

    /// Number of guests (including the user)
    #[serde(default = "default_guests")]
    pub guests: u32,

    /// Optional note (e.g., "Bringing potato salad!")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn default_guests() -> u32 {
    1
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RsvpStatus {
    Going,
    Interested,
    NotGoing,
}
