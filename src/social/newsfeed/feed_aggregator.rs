//! Newsfeed aggregation across multiple feed rooms.
//!
//! The newsfeed is the union of all joined feed rooms, sorted
//! chronologically or by engagement.

use matrix_sdk::{
    room::Room,
    ruma::{
        api::client::filter::{FilterDefinition, RoomEventFilter, RoomFilter},
        events::TimelineEventType,
        MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId, RoomId,
    },
    Client,
};
use std::collections::BTreeMap;

use crate::social::post::PostContent;

/// Sync filter optimized for feed rooms.
///
/// Creates a filter that fetches only message events, reactions, and redactions
/// for efficient feed synchronization.
pub fn create_feed_sync_filter() -> FilterDefinition {
    let mut timeline_filter = RoomEventFilter::default();
    timeline_filter.types = Some(vec![
        TimelineEventType::RoomMessage.to_string(),
        TimelineEventType::Reaction.to_string(),
        TimelineEventType::RoomRedaction.to_string(),
    ]);
    timeline_filter.limit = Some(10u32.into());

    // State filter for minimal room info
    let mut state_filter = RoomEventFilter::default();
    state_filter.types = Some(vec!["m.room.name".to_string(), "m.room.avatar".to_string()]);

    let mut room_filter = RoomFilter::default();
    room_filter.timeline = timeline_filter;
    room_filter.state = state_filter;

    let mut filter = FilterDefinition::default();
    filter.room = room_filter;

    filter
}

/// Sort order for the newsfeed.
///
/// Determines how feed items are ordered when displayed to the user.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FeedSortOrder {
    /// Most recent first (Twitter-style).
    #[default]
    Chronological,
    /// By engagement (reactions + comments).
    Engagement,
    /// Grouped by author, then by time within each author.
    GroupedByAuthor,
}

/// An aggregated feed item from any feed room.
///
/// Represents a single post with its metadata and engagement metrics.
#[derive(Clone, Debug)]
pub struct FeedItem {
    /// Source room ID where this post lives.
    pub room_id: OwnedRoomId,
    /// Event ID of this post.
    pub event_id: OwnedEventId,
    /// Author user ID.
    pub sender: OwnedUserId,
    /// Timestamp when the post was created.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
    /// Message content of the post.
    pub content: PostContent,
    /// Reaction counts by emoji.
    pub reactions: BTreeMap<String, u32>,
    /// Number of comments/replies to this post.
    pub comment_count: u32,
}

impl FeedItem {
    /// Calculate the total engagement for this item.
    ///
    /// Engagement is the sum of all reaction counts plus comment count.
    pub fn engagement(&self) -> u32 {
        self.reactions.values().sum::<u32>() + self.comment_count
    }
}

/// Service for aggregating feed items from multiple rooms.
///
/// The FeedAggregator maintains a list of feed rooms to watch and provides
/// methods to fetch a unified, sorted feed from all of them.
pub struct FeedAggregator {
    client: Client,
    /// IDs of feed rooms to aggregate.
    feed_rooms: Vec<OwnedRoomId>,
    /// Current sort order.
    sort_order: FeedSortOrder,
}

impl FeedAggregator {
    /// Create a new FeedAggregator.
    ///
    /// # Arguments
    /// * `client` - The Matrix client to use for fetching room data.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            feed_rooms: Vec::new(),
            sort_order: FeedSortOrder::default(),
        }
    }

    /// Add a feed room to the aggregation.
    ///
    /// If the room is already being aggregated, this is a no-op.
    ///
    /// # Arguments
    /// * `room_id` - The room ID to add to the feed.
    pub fn add_feed_room(&mut self, room_id: OwnedRoomId) {
        if !self.feed_rooms.contains(&room_id) {
            self.feed_rooms.push(room_id);
        }
    }

    /// Remove a feed room from aggregation.
    ///
    /// # Arguments
    /// * `room_id` - The room ID to remove from the feed.
    pub fn remove_feed_room(&mut self, room_id: &RoomId) {
        self.feed_rooms.retain(|id| id != room_id);
    }

    /// Check if a room is being aggregated.
    pub fn contains_room(&self, room_id: &RoomId) -> bool {
        self.feed_rooms.iter().any(|id| id == room_id)
    }

    /// Get the number of feed rooms being aggregated.
    pub fn room_count(&self) -> usize {
        self.feed_rooms.len()
    }

    /// Get the current sort order.
    pub fn sort_order(&self) -> FeedSortOrder {
        self.sort_order
    }

    /// Set the sort order for the feed.
    ///
    /// # Arguments
    /// * `order` - The new sort order to use.
    pub fn set_sort_order(&mut self, order: FeedSortOrder) {
        self.sort_order = order;
    }

    /// Get aggregated feed items from all feed rooms.
    ///
    /// Fetches recent items from all tracked feed rooms, combines them,
    /// sorts them according to the current sort order, and returns up to
    /// `limit` items.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of items to return.
    ///
    /// # Errors
    /// Returns an error if there's a problem fetching room data.
    pub async fn get_aggregated_feed(&self, limit: usize) -> Result<Vec<FeedItem>, FeedError> {
        let mut all_items = Vec::new();

        for room_id in &self.feed_rooms {
            if let Some(room) = self.client.get_room(room_id) {
                // Fetch recent timeline items from this room
                let items = self.fetch_room_items(&room, limit).await?;
                all_items.extend(items);
            }
        }

        // Sort according to current order
        self.sort_items(&mut all_items);

        // Limit total results
        all_items.truncate(limit);

        Ok(all_items)
    }

    /// Fetch items from a single room.
    ///
    /// This is a placeholder implementation that will need to be expanded
    /// to actually parse timeline events into FeedItems.
    async fn fetch_room_items(
        &self,
        _room: &Room,
        _limit: usize,
    ) -> Result<Vec<FeedItem>, FeedError> {
        // TODO: Implement actual timeline fetching
        // This would involve:
        // 1. Getting the room timeline
        // 2. Filtering for message events
        // 3. Collecting reactions for each message
        // 4. Converting to FeedItem format
        Ok(Vec::new())
    }

    /// Sort items according to the current sort order.
    fn sort_items(&self, items: &mut Vec<FeedItem>) {
        match self.sort_order {
            FeedSortOrder::Chronological => {
                items.sort_by(|a, b| b.origin_server_ts.cmp(&a.origin_server_ts));
            }
            FeedSortOrder::Engagement => {
                items.sort_by(|a, b| {
                    let a_engagement = a.engagement();
                    let b_engagement = b.engagement();
                    b_engagement.cmp(&a_engagement)
                });
            }
            FeedSortOrder::GroupedByAuthor => {
                items.sort_by(|a, b| {
                    a.sender
                        .cmp(&b.sender)
                        .then_with(|| b.origin_server_ts.cmp(&a.origin_server_ts))
                });
            }
        }
    }
}

/// Errors that can occur when working with the feed aggregator.
#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    /// No feed rooms are being tracked.
    #[error("No feed rooms configured")]
    NoFeedRooms,

    /// A room was not found.
    #[error("Room not found: {0}")]
    RoomNotFound(OwnedRoomId),

    /// Failed to fetch timeline data.
    #[error("Failed to fetch timeline: {0}")]
    TimelineFetchError(String),

    /// An error occurred in the Matrix SDK.
    #[error("Matrix error: {0}")]
    MatrixError(#[from] matrix_sdk::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_sort_order_default() {
        let order = FeedSortOrder::default();
        assert_eq!(order, FeedSortOrder::Chronological);
    }

    #[test]
    fn test_create_feed_sync_filter() {
        let filter = create_feed_sync_filter();
        assert!(filter.room.timeline.types.is_some());
        let types = filter.room.timeline.types.unwrap();
        assert!(types.contains(&TimelineEventType::RoomMessage.to_string()));
        assert!(types.contains(&TimelineEventType::Reaction.to_string()));
    }

    #[test]
    fn test_feed_item_engagement() {
        let mut reactions = BTreeMap::new();
        reactions.insert("👍".to_string(), 5);
        reactions.insert("❤️".to_string(), 3);

        let item = FeedItem {
            room_id: "!room:example.org".try_into().unwrap(),
            event_id: "$event:example.org".try_into().unwrap(),
            sender: "@user:example.org".try_into().unwrap(),
            origin_server_ts: MilliSecondsSinceUnixEpoch(0u64.try_into().unwrap()),
            content: PostContent::Text {
                body: "Test".to_string(),
                formatted_body: None,
                mentions: std::collections::BTreeSet::new(),
            },
            reactions,
            comment_count: 2,
        };

        assert_eq!(item.engagement(), 10); // 5 + 3 + 2
    }
}
