use ruma::events::macros::EventContent;
use ruma::events::EmptyStateKey;
use serde::{Deserialize, Serialize};

/// Custom profile data stored as room state in a user's profile room.
/// Event type: `org.social.profile`
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "org.social.profile", kind = State, state_key_type = EmptyStateKey)]
#[serde(deny_unknown_fields)]
pub struct SocialProfileEventContent {
    /// User's biography/about text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,

    /// User's location (city, country, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// User's website URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<url::Url>,

    /// Cover/banner image MXC URI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<ruma::OwnedMxcUri>,

    /// Additional custom fields (for extensibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<serde_json::Value>,
}
