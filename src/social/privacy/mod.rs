//! Privacy and security module.

pub mod sharing_guard;

mod validation;

pub use sharing_guard::*;

/// Maximum allowed sizes for various content types
pub mod limits {
    /// Maximum bio length in characters
    pub const MAX_BIO_LENGTH: usize = 500;
    /// Maximum post text length
    pub const MAX_POST_LENGTH: usize = 10_000;
    /// Maximum event description length
    pub const MAX_EVENT_DESCRIPTION: usize = 5_000;
    /// Maximum RSVP note length
    pub const MAX_RSVP_NOTE_LENGTH: usize = 200;
    /// Maximum number of mentions per post
    pub const MAX_MENTIONS_PER_POST: usize = 50;
    /// Maximum link preview description length
    pub const MAX_LINK_PREVIEW_DESCRIPTION: usize = 500;
}

/// Sanitize user input for safety
pub fn sanitize_user_input(input: &str, max_length: usize) -> String {
    // Trim whitespace
    let trimmed = input.trim();

    // Truncate to max length (at char boundary)
    let truncated: String = trimmed.chars().take(max_length).collect();

    // Basic HTML entity encoding for display safety
    truncated
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Validate MXC URI format
pub fn validate_mxc_uri(uri: &str) -> Result<(), ValidationError> {
    if !uri.starts_with("mxc://") {
        return Err(ValidationError::InvalidMxcUri(
            "Must start with mxc://".into(),
        ));
    }

    // Basic format check: mxc://server/media_id
    let parts: Vec<_> = uri.strip_prefix("mxc://").unwrap().split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(ValidationError::InvalidMxcUri("Invalid format".into()));
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid MXC URI: {0}")]
    InvalidMxcUri(String),
    #[error("Content too long: {field} exceeds {max} characters")]
    ContentTooLong { field: String, max: usize },
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}
