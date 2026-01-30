//! Aggregated newsfeed for combining multiple feed rooms.
//!
//! This module provides services for aggregating posts from multiple
//! feed rooms into a single unified newsfeed, with sorting and filtering
//! capabilities.

pub mod feed_aggregator;
pub mod feed_filter;

pub use feed_aggregator::{create_feed_sync_filter, FeedAggregator, FeedError, FeedItem, FeedSortOrder};
pub use feed_filter::{ContentFilter, FeedFilterSettings};
