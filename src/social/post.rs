//! Post creation and management.
//!
//! Posts are standard Matrix messages with optional social extensions.
//! This module provides types for creating posts with various content types
//! (text, images, videos, links) and converting them to Matrix message events.

use matrix_sdk::ruma::{
    events::room::message::{
        ImageMessageEventContent, MessageType, RoomMessageEventContent, VideoMessageEventContent,
    },
    MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedMxcUri, OwnedRoomId, OwnedUserId,
};
use robrix_social_events::link_preview::LinkPreview;
use std::collections::BTreeSet;

use crate::social::feed_room::FeedPrivacy;

/// A social media post ready to be sent to feed rooms.
#[derive(Clone, Debug)]
pub struct Post {
    /// The message content of the post.
    pub content: PostContent,
    /// Target feed rooms to post to.
    pub targets: Vec<OwnedRoomId>,
    /// Privacy levels this post is intended for.
    pub privacy_levels: Vec<FeedPrivacy>,
}

impl Post {
    /// Create a new text post.
    pub fn text(body: impl Into<String>) -> Self {
        Self {
            content: PostContent::Text {
                body: body.into(),
                formatted_body: None,
                mentions: BTreeSet::new(),
            },
            targets: Vec::new(),
            privacy_levels: vec![FeedPrivacy::Public],
        }
    }

    /// Create a new post with an image.
    pub fn image(mxc_uri: OwnedMxcUri, width: u32, height: u32) -> Self {
        Self {
            content: PostContent::Image {
                mxc_uri,
                caption: None,
                thumbnail_uri: None,
                width,
                height,
            },
            targets: Vec::new(),
            privacy_levels: vec![FeedPrivacy::Public],
        }
    }

    /// Create a new post with a video.
    pub fn video(mxc_uri: OwnedMxcUri) -> Self {
        Self {
            content: PostContent::Video {
                mxc_uri,
                caption: None,
                thumbnail_uri: None,
                duration_ms: None,
            },
            targets: Vec::new(),
            privacy_levels: vec![FeedPrivacy::Public],
        }
    }

    /// Create a new link post.
    pub fn link(url: url::Url) -> Self {
        Self {
            content: PostContent::Link {
                url,
                comment: None,
                preview: Box::new(None),
            },
            targets: Vec::new(),
            privacy_levels: vec![FeedPrivacy::Public],
        }
    }

    /// Set the target room IDs for this post.
    pub fn with_targets(mut self, targets: Vec<OwnedRoomId>) -> Self {
        self.targets = targets;
        self
    }

    /// Set the privacy levels for this post.
    pub fn with_privacy(mut self, privacy_levels: Vec<FeedPrivacy>) -> Self {
        self.privacy_levels = privacy_levels;
        self
    }

    /// Add a caption to image or video content.
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        let caption_str = caption.into();
        match &mut self.content {
            PostContent::Image { caption, .. } => {
                *caption = Some(caption_str);
            }
            PostContent::Video { caption, .. } => {
                *caption = Some(caption_str);
            }
            PostContent::Link { comment, .. } => {
                *comment = Some(caption_str);
            }
            PostContent::Text { .. } => {
                // Text posts don't have captions, ignore
            }
        }
        self
    }

    /// Convert the post content to a Matrix message.
    pub fn into_room_message(&self) -> RoomMessageEventContent {
        self.content.into_room_message()
    }
}

/// Post content types.
///
/// Different types of content that can be included in a social post.
#[derive(Clone, Debug)]
pub enum PostContent {
    /// Text-only post, optionally with HTML formatting and mentions.
    Text {
        /// Plain text body of the post.
        body: String,
        /// Optional HTML-formatted body.
        formatted_body: Option<String>,
        /// User IDs mentioned in the post.
        mentions: BTreeSet<OwnedUserId>,
    },
    /// Photo/image post with optional caption.
    Image {
        /// MXC URI of the uploaded image.
        mxc_uri: OwnedMxcUri,
        /// Optional caption for the image.
        caption: Option<String>,
        /// Optional thumbnail MXC URI.
        thumbnail_uri: Option<OwnedMxcUri>,
        /// Image width in pixels.
        width: u32,
        /// Image height in pixels.
        height: u32,
    },
    /// Video post with optional caption.
    Video {
        /// MXC URI of the uploaded video.
        mxc_uri: OwnedMxcUri,
        /// Optional caption for the video.
        caption: Option<String>,
        /// Optional thumbnail MXC URI.
        thumbnail_uri: Option<OwnedMxcUri>,
        /// Duration in milliseconds.
        duration_ms: Option<u64>,
    },
    /// Link share with optional preview.
    Link {
        /// The URL being shared.
        url: url::Url,
        /// Optional comment about the link.
        comment: Option<String>,
        /// Optional rich link preview data (boxed to reduce enum size).
        preview: Box<Option<LinkPreview>>,
    },
}

impl PostContent {
    /// Convert this post content to a Matrix room message.
    pub fn into_room_message(&self) -> RoomMessageEventContent {
        match self {
            Self::Text {
                body,
                formatted_body,
                mentions,
            } => {
                let mut content = if let Some(html) = formatted_body {
                    RoomMessageEventContent::text_html(body, html)
                } else {
                    RoomMessageEventContent::text_plain(body)
                };

                // Add mentions if present
                if !mentions.is_empty() {
                    content = content.add_mentions(
                        matrix_sdk::ruma::events::Mentions::with_user_ids(mentions.iter().cloned()),
                    );
                }

                content
            }
            Self::Image {
                mxc_uri,
                caption,
                thumbnail_uri: _,
                width: _,
                height: _,
            } => {
                let body = caption.clone().unwrap_or_else(|| "Image".to_string());
                let content = ImageMessageEventContent::plain(body, mxc_uri.clone());
                // Note: Thumbnail info and dimensions could be added here
                // using ImageMessageEventContent methods if needed
                RoomMessageEventContent::new(MessageType::Image(content))
            }
            Self::Video {
                mxc_uri,
                caption,
                thumbnail_uri: _,
                duration_ms: _,
            } => {
                let body = caption.clone().unwrap_or_else(|| "Video".to_string());
                let content = VideoMessageEventContent::plain(body, mxc_uri.clone());
                // Note: Thumbnail info and duration could be added here
                RoomMessageEventContent::new(MessageType::Video(content))
            }
            Self::Link {
                url,
                comment,
                preview,
            } => {
                // Create a text message with the URL and optional comment
                let body = if let Some(comment) = comment {
                    format!("{}\n\n{}", comment, url)
                } else {
                    url.to_string()
                };

                // Generate HTML with link preview if available
                let html = if let Some(preview) = &**preview {
                    let mut html = String::new();
                    if let Some(comment) = comment {
                        html.push_str(&format!("<p>{}</p>", htmlize::escape_text(comment)));
                    }
                    html.push_str("<blockquote>");
                    if let Some(title) = &preview.title {
                        html.push_str(&format!(
                            "<strong><a href=\"{}\">{}</a></strong>",
                            url,
                            htmlize::escape_text(title)
                        ));
                    } else {
                        html.push_str(&format!("<a href=\"{}\">{}</a>", url, url));
                    }
                    if let Some(description) = &preview.description {
                        html.push_str(&format!(
                            "<br/><em>{}</em>",
                            htmlize::escape_text(description)
                        ));
                    }
                    if let Some(site_name) = &preview.site_name {
                        html.push_str(&format!(
                            "<br/><small>{}</small>",
                            htmlize::escape_text(site_name)
                        ));
                    }
                    html.push_str("</blockquote>");
                    Some(html)
                } else if comment.is_some() {
                    Some(format!(
                        "<p>{}</p><p><a href=\"{}\">{}</a></p>",
                        htmlize::escape_text(comment.as_ref().unwrap()),
                        url,
                        url
                    ))
                } else {
                    Some(format!("<a href=\"{}\">{}</a>", url, url))
                };

                if let Some(html) = html {
                    RoomMessageEventContent::text_html(body, html)
                } else {
                    RoomMessageEventContent::text_plain(body)
                }
            }
        }
    }
}

/// Metadata about a post.
///
/// Contains timestamps and edit history for tracking post lifecycle.
#[derive(Clone, Debug)]
pub struct PostMetadata {
    /// Event ID of the post.
    pub event_id: OwnedEventId,
    /// Room ID where the post was sent.
    pub room_id: OwnedRoomId,
    /// User ID of the post author.
    pub sender: OwnedUserId,
    /// Original post timestamp from the server.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
    /// Edit history timestamps (most recent first).
    pub edit_timestamps: Vec<MilliSecondsSinceUnixEpoch>,
    /// Whether the post has been edited.
    pub is_edited: bool,
}

impl PostMetadata {
    /// Create new metadata for a post.
    pub fn new(
        event_id: OwnedEventId,
        room_id: OwnedRoomId,
        sender: OwnedUserId,
        origin_server_ts: MilliSecondsSinceUnixEpoch,
    ) -> Self {
        Self {
            event_id,
            room_id,
            sender,
            origin_server_ts,
            edit_timestamps: Vec::new(),
            is_edited: false,
        }
    }

    /// Mark the post as edited with the given timestamp.
    pub fn add_edit(&mut self, timestamp: MilliSecondsSinceUnixEpoch) {
        self.edit_timestamps.insert(0, timestamp);
        self.is_edited = true;
    }

    /// Get the most recent timestamp (edit time or original time).
    pub fn last_modified(&self) -> MilliSecondsSinceUnixEpoch {
        self.edit_timestamps
            .first()
            .copied()
            .unwrap_or(self.origin_server_ts)
    }
}

/// A complete post with content and metadata.
///
/// This represents a post as it exists in a feed room, including
/// all associated metadata from the Matrix event.
#[derive(Clone, Debug)]
pub struct FeedPost {
    /// The post content.
    pub content: PostContent,
    /// Post metadata (timestamps, author, etc.).
    pub metadata: PostMetadata,
}

impl FeedPost {
    /// Create a new feed post from content and metadata.
    pub fn new(content: PostContent, metadata: PostMetadata) -> Self {
        Self { content, metadata }
    }

    /// Get the event ID of this post.
    pub fn event_id(&self) -> &OwnedEventId {
        &self.metadata.event_id
    }

    /// Get the author's user ID.
    pub fn sender(&self) -> &OwnedUserId {
        &self.metadata.sender
    }

    /// Get the room ID where this post lives.
    pub fn room_id(&self) -> &OwnedRoomId {
        &self.metadata.room_id
    }

    /// Get the original timestamp.
    pub fn timestamp(&self) -> MilliSecondsSinceUnixEpoch {
        self.metadata.origin_server_ts
    }

    /// Check if this post has been edited.
    pub fn is_edited(&self) -> bool {
        self.metadata.is_edited
    }
}

/// Errors that can occur when working with posts.
#[derive(Debug, thiserror::Error)]
pub enum PostError {
    /// The specified room was not found.
    #[error("Room not found: {0}")]
    RoomNotFound(OwnedRoomId),

    /// User is not logged in.
    #[error("Not logged in")]
    NotLoggedIn,

    /// User does not have permission to post.
    #[error("Permission denied to post in room")]
    PermissionDenied,

    /// Failed to upload media.
    #[error("Failed to upload media: {0}")]
    MediaUploadFailed(String),

    /// An error occurred in the Matrix SDK.
    #[error("Matrix error: {0}")]
    MatrixError(#[from] matrix_sdk::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_post_creation() {
        let post = Post::text("Hello, world!");
        assert!(matches!(post.content, PostContent::Text { body, .. } if body == "Hello, world!"));
        assert_eq!(post.privacy_levels, vec![FeedPrivacy::Public]);
    }

    #[test]
    fn test_post_with_privacy() {
        let post = Post::text("Test").with_privacy(vec![FeedPrivacy::Friends]);
        assert_eq!(post.privacy_levels, vec![FeedPrivacy::Friends]);
    }

    #[test]
    fn test_image_post_with_caption() {
        let mxc: OwnedMxcUri = "mxc://example.org/abc123".into();
        let post = Post::image(mxc, 800, 600).with_caption("A nice photo");
        assert!(matches!(
            post.content,
            PostContent::Image { caption: Some(c), .. } if c == "A nice photo"
        ));
    }

    #[test]
    fn test_text_to_room_message() {
        let post = Post::text("Hello");
        let msg = post.into_room_message();
        assert!(matches!(msg.msgtype, MessageType::Text(_)));
    }
}
