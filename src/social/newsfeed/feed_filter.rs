//! Feed filtering for the aggregated newsfeed.
//!
//! Provides filtering capabilities to refine what content appears in a user's
//! newsfeed based on content type, author, and other criteria.

use matrix_sdk::ruma::{OwnedUserId, UserId};
use std::collections::HashSet;

use super::feed_aggregator::FeedItem;

/// Content type filter for feed items.
///
/// Allows filtering the feed to show only certain types of content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ContentFilter {
    /// Show all content types.
    #[default]
    All,
    /// Show only text posts.
    TextOnly,
    /// Show only media posts (images and videos).
    MediaOnly,
    /// Show only posts with links.
    LinksOnly,
}

impl ContentFilter {
    /// Check if a feed item matches this filter.
    pub fn matches(&self, item: &FeedItem) -> bool {
        use crate::social::post::PostContent;

        match self {
            Self::All => true,
            Self::TextOnly => matches!(item.content, PostContent::Text { .. }),
            Self::MediaOnly => {
                matches!(
                    item.content,
                    PostContent::Image { .. } | PostContent::Video { .. }
                )
            }
            Self::LinksOnly => matches!(item.content, PostContent::Link { .. }),
        }
    }
}

/// Settings for filtering the newsfeed.
///
/// Combines multiple filter criteria that can be applied to feed items.
#[derive(Clone, Debug, Default)]
pub struct FeedFilterSettings {
    /// Filter by content type.
    pub content_filter: ContentFilter,
    /// Show only posts from these users (empty = show all).
    pub authors: HashSet<OwnedUserId>,
    /// Hide posts from these users.
    pub muted_authors: HashSet<OwnedUserId>,
    /// Minimum engagement threshold (0 = no minimum).
    pub min_engagement: u32,
    /// Only show posts newer than this many seconds (0 = no limit).
    pub max_age_seconds: u64,
}

impl FeedFilterSettings {
    /// Create a new filter with default settings (show all).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the content type filter.
    pub fn with_content_filter(mut self, filter: ContentFilter) -> Self {
        self.content_filter = filter;
        self
    }

    /// Filter to show only posts from specific authors.
    pub fn with_authors(mut self, authors: impl IntoIterator<Item = OwnedUserId>) -> Self {
        self.authors = authors.into_iter().collect();
        self
    }

    /// Add an author to the allowed list.
    pub fn add_author(&mut self, author: OwnedUserId) {
        self.authors.insert(author);
    }

    /// Mute posts from specific users.
    pub fn with_muted_authors(mut self, authors: impl IntoIterator<Item = OwnedUserId>) -> Self {
        self.muted_authors = authors.into_iter().collect();
        self
    }

    /// Mute a specific user.
    pub fn mute_author(&mut self, author: OwnedUserId) {
        self.muted_authors.insert(author);
    }

    /// Unmute a specific user.
    pub fn unmute_author(&mut self, author: &UserId) {
        self.muted_authors.retain(|a| a != author);
    }

    /// Set minimum engagement threshold.
    pub fn with_min_engagement(mut self, min: u32) -> Self {
        self.min_engagement = min;
        self
    }

    /// Set maximum post age in seconds.
    pub fn with_max_age(mut self, seconds: u64) -> Self {
        self.max_age_seconds = seconds;
        self
    }

    /// Check if a feed item passes all filters.
    pub fn matches(&self, item: &FeedItem) -> bool {
        // Check content type filter
        if !self.content_filter.matches(item) {
            return false;
        }

        // Check author filter (if non-empty, only show listed authors)
        if !self.authors.is_empty() && !self.authors.contains(&item.sender) {
            return false;
        }

        // Check muted authors
        if self.muted_authors.contains(&item.sender) {
            return false;
        }

        // Check minimum engagement
        if self.min_engagement > 0 && item.engagement() < self.min_engagement {
            return false;
        }

        // Note: max_age_seconds check would require current time comparison
        // which is deferred to the caller or a method with time parameter

        true
    }

    /// Apply this filter to a list of feed items.
    ///
    /// Returns a new vector containing only items that match all filter criteria.
    pub fn apply(&self, items: Vec<FeedItem>) -> Vec<FeedItem> {
        items
            .into_iter()
            .filter(|item| self.matches(item))
            .collect()
    }

    /// Check if any filters are active.
    ///
    /// Returns true if the settings are not the default "show all" configuration.
    pub fn has_active_filters(&self) -> bool {
        self.content_filter != ContentFilter::All
            || !self.authors.is_empty()
            || !self.muted_authors.is_empty()
            || self.min_engagement > 0
            || self.max_age_seconds > 0
    }

    /// Reset all filters to default.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::post::PostContent;
    use matrix_sdk::ruma::MilliSecondsSinceUnixEpoch;
    use std::collections::BTreeMap;

    fn make_text_item(sender: &str, engagement: u32) -> FeedItem {
        FeedItem {
            room_id: "!room:example.org".try_into().unwrap(),
            event_id: "$event:example.org".try_into().unwrap(),
            sender: sender.try_into().unwrap(),
            origin_server_ts: MilliSecondsSinceUnixEpoch(0u64.try_into().unwrap()),
            content: PostContent::Text {
                body: "Test".to_string(),
                formatted_body: None,
                mentions: std::collections::BTreeSet::new(),
            },
            reactions: {
                let mut r = BTreeMap::new();
                if engagement > 0 {
                    r.insert("👍".to_string(), engagement);
                }
                r
            },
            comment_count: 0,
        }
    }

    #[test]
    fn test_content_filter_all() {
        let filter = ContentFilter::All;
        let item = make_text_item("@user:example.org", 0);
        assert!(filter.matches(&item));
    }

    #[test]
    fn test_content_filter_text_only() {
        let filter = ContentFilter::TextOnly;
        let item = make_text_item("@user:example.org", 0);
        assert!(filter.matches(&item));
    }

    #[test]
    fn test_filter_settings_muted_author() {
        let muted_user: OwnedUserId = "@muted:example.org".try_into().unwrap();
        let settings = FeedFilterSettings::new().with_muted_authors([muted_user.clone()]);

        let muted_item = make_text_item("@muted:example.org", 0);
        let normal_item = make_text_item("@user:example.org", 0);

        assert!(!settings.matches(&muted_item));
        assert!(settings.matches(&normal_item));
    }

    #[test]
    fn test_filter_settings_min_engagement() {
        let settings = FeedFilterSettings::new().with_min_engagement(5);

        let low_engagement = make_text_item("@user:example.org", 2);
        let high_engagement = make_text_item("@user:example.org", 10);

        assert!(!settings.matches(&low_engagement));
        assert!(settings.matches(&high_engagement));
    }

    #[test]
    fn test_filter_settings_apply() {
        let muted: OwnedUserId = "@muted:example.org".try_into().unwrap();
        let settings = FeedFilterSettings::new().with_muted_authors([muted]);

        let items = vec![
            make_text_item("@user1:example.org", 0),
            make_text_item("@muted:example.org", 0),
            make_text_item("@user2:example.org", 0),
        ];

        let filtered = settings.apply(items);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_has_active_filters() {
        let default = FeedFilterSettings::new();
        assert!(!default.has_active_filters());

        let with_filter = FeedFilterSettings::new().with_min_engagement(1);
        assert!(with_filter.has_active_filters());
    }
}
