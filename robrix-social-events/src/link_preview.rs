use serde::{Deserialize, Serialize};

/// Rich link preview data embedded in message content.
/// Field name: `org.social.link_preview`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LinkPreview {
    /// Original URL
    pub url: url::Url,

    /// Page title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Page description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Preview image MXC URI (uploaded to homeserver)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ruma::OwnedMxcUri>,

    /// Site name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
}
