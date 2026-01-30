use ruma::events::macros::EventContent;
use ruma::events::EmptyStateKey;
use serde::{Deserialize, Serialize};

/// Event/gathering details stored as room state.
/// Event type: `org.social.event`
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "org.social.event", kind = State, state_key_type = EmptyStateKey)]
#[serde(deny_unknown_fields)]
pub struct SocialEventEventContent {
    /// Event title
    pub title: String,

    /// Event description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Start time (Unix timestamp in milliseconds)
    pub start_time: u64,

    /// End time (Unix timestamp in milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,

    /// Event location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<EventLocation>,

    /// Cover image MXC URI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<ruma::OwnedMxcUri>,

    /// Visibility level
    pub visibility: EventVisibility,

    /// RSVP deadline (Unix timestamp in milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rsvp_deadline: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EventLocation {
    /// Human-readable location name
    pub name: String,

    /// Full address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Geo URI (e.g., "geo:40.7829,-73.9654")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EventVisibility {
    Public,
    Private,
}
