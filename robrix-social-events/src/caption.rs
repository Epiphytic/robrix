use serde::{Deserialize, Serialize};

/// Caption data for media posts.
/// Field name: `org.social.caption`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Caption {
    /// Caption text
    pub text: String,

    /// Formatted caption (HTML)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted_text: Option<String>,
}
