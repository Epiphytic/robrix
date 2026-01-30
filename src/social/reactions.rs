//! Reactions aggregation for social posts.
//!
//! This module provides types and utilities for aggregating reactions
//! (emoji responses) from Matrix timeline events. Reactions are a key
//! social feature that allows users to express quick responses to posts.

use matrix_sdk::ruma::{OwnedEventId, OwnedUserId};
use std::collections::{BTreeMap, BTreeSet};

/// Summary of reactions on a post.
///
/// Aggregates reaction counts by emoji and tracks which users
/// have reacted with each emoji.
#[derive(Clone, Debug, Default)]
pub struct ReactionSummary {
    /// Count of each reaction emoji.
    counts: BTreeMap<String, u32>,
    /// Users who reacted with each emoji.
    users_by_emoji: BTreeMap<String, BTreeSet<OwnedUserId>>,
    /// Event IDs of reaction events, keyed by (user_id, emoji).
    event_ids: BTreeMap<(OwnedUserId, String), OwnedEventId>,
    /// Total number of reactions.
    total: u32,
}

impl ReactionSummary {
    /// Create a new empty reaction summary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a reaction to the summary.
    ///
    /// # Arguments
    /// * `emoji` - The reaction emoji (e.g., "👍", "❤️")
    /// * `user_id` - The user who reacted
    /// * `event_id` - The event ID of the reaction event
    pub fn add_reaction(
        &mut self,
        emoji: impl Into<String>,
        user_id: OwnedUserId,
        event_id: OwnedEventId,
    ) {
        let emoji = emoji.into();

        // Only add if this user hasn't already reacted with this emoji
        let users = self.users_by_emoji.entry(emoji.clone()).or_default();
        if users.insert(user_id.clone()) {
            *self.counts.entry(emoji.clone()).or_insert(0) += 1;
            self.total += 1;
            self.event_ids.insert((user_id, emoji), event_id);
        }
    }

    /// Remove a reaction from the summary.
    ///
    /// # Arguments
    /// * `emoji` - The reaction emoji to remove
    /// * `user_id` - The user whose reaction to remove
    ///
    /// # Returns
    /// The event ID of the removed reaction, if it existed.
    pub fn remove_reaction(&mut self, emoji: &str, user_id: &OwnedUserId) -> Option<OwnedEventId> {
        if let Some(users) = self.users_by_emoji.get_mut(emoji) {
            if users.remove(user_id) {
                if let Some(count) = self.counts.get_mut(emoji) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        self.counts.remove(emoji);
                    }
                }
                self.total = self.total.saturating_sub(1);

                // Clean up empty user sets
                if users.is_empty() {
                    self.users_by_emoji.remove(emoji);
                }

                return self.event_ids.remove(&(user_id.clone(), emoji.to_string()));
            }
        }
        None
    }

    /// Get the count for a specific emoji.
    pub fn count(&self, emoji: &str) -> u32 {
        self.counts.get(emoji).copied().unwrap_or(0)
    }

    /// Get the total number of reactions.
    pub fn total(&self) -> u32 {
        self.total
    }

    /// Check if a user has reacted with a specific emoji.
    pub fn has_user_reacted(&self, emoji: &str, user_id: &OwnedUserId) -> bool {
        self.users_by_emoji
            .get(emoji)
            .is_some_and(|users| users.contains(user_id))
    }

    /// Get the reaction event ID for a specific user and emoji.
    pub fn get_event_id(&self, user_id: &OwnedUserId, emoji: &str) -> Option<&OwnedEventId> {
        self.event_ids.get(&(user_id.clone(), emoji.to_string()))
    }

    /// Get all reaction counts as a map.
    pub fn counts(&self) -> &BTreeMap<String, u32> {
        &self.counts
    }

    /// Get all unique emojis used in reactions.
    pub fn emojis(&self) -> impl Iterator<Item = &String> {
        self.counts.keys()
    }

    /// Get the users who reacted with a specific emoji.
    pub fn users_for_emoji(&self, emoji: &str) -> Option<&BTreeSet<OwnedUserId>> {
        self.users_by_emoji.get(emoji)
    }

    /// Get the most popular reactions, sorted by count descending.
    pub fn top_reactions(&self, limit: usize) -> Vec<(&String, u32)> {
        let mut reactions: Vec<_> = self.counts.iter().map(|(k, v)| (k, *v)).collect();
        reactions.sort_by(|a, b| b.1.cmp(&a.1));
        reactions.truncate(limit);
        reactions
    }

    /// Check if there are any reactions.
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    /// Clear all reactions.
    pub fn clear(&mut self) {
        self.counts.clear();
        self.users_by_emoji.clear();
        self.event_ids.clear();
        self.total = 0;
    }

    /// Merge another reaction summary into this one.
    ///
    /// Useful for combining reactions from multiple sources.
    pub fn merge(&mut self, other: &ReactionSummary) {
        for (key, event_id) in &other.event_ids {
            let (user_id, emoji) = key;
            self.add_reaction(emoji.clone(), user_id.clone(), event_id.clone());
        }
    }
}

/// Common emoji reactions used in social contexts.
pub mod common_emojis {
    /// Like/thumbs up reaction.
    pub const LIKE: &str = "👍";
    /// Love/heart reaction.
    pub const LOVE: &str = "❤️";
    /// Laugh/funny reaction.
    pub const LAUGH: &str = "😂";
    /// Wow/surprised reaction.
    pub const WOW: &str = "😮";
    /// Sad reaction.
    pub const SAD: &str = "😢";
    /// Angry reaction.
    pub const ANGRY: &str = "😠";
    /// Fire/lit reaction.
    pub const FIRE: &str = "🔥";
    /// Clap/applause reaction.
    pub const CLAP: &str = "👏";
    /// Thinking reaction.
    pub const THINKING: &str = "🤔";
    /// Celebrate/party reaction.
    pub const CELEBRATE: &str = "🎉";

    /// Default set of quick reaction options.
    pub const QUICK_REACTIONS: &[&str] = &[LIKE, LOVE, LAUGH, WOW, SAD, ANGRY];
}

/// A single reaction entry for display purposes.
#[derive(Clone, Debug)]
pub struct ReactionDisplay {
    /// The emoji used for the reaction.
    pub emoji: String,
    /// Number of users who used this reaction.
    pub count: u32,
    /// Whether the current user has used this reaction.
    pub is_selected: bool,
}

impl ReactionDisplay {
    /// Create a new reaction display entry.
    pub fn new(emoji: impl Into<String>, count: u32, is_selected: bool) -> Self {
        Self {
            emoji: emoji.into(),
            count,
            is_selected,
        }
    }
}

/// Convert a reaction summary to display entries for a specific user.
pub fn reactions_for_display(
    summary: &ReactionSummary,
    current_user: Option<&OwnedUserId>,
) -> Vec<ReactionDisplay> {
    summary
        .top_reactions(10)
        .into_iter()
        .map(|(emoji, count)| {
            let is_selected =
                current_user.is_some_and(|user_id| summary.has_user_reacted(emoji, user_id));
            ReactionDisplay::new(emoji.clone(), count, is_selected)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user_id(name: &str) -> OwnedUserId {
        format!("@{}:example.org", name).try_into().unwrap()
    }

    fn event_id(id: &str) -> OwnedEventId {
        format!("${}:example.org", id).try_into().unwrap()
    }

    #[test]
    fn test_add_reaction() {
        let mut summary = ReactionSummary::new();
        summary.add_reaction("👍", user_id("alice"), event_id("1"));
        summary.add_reaction("👍", user_id("bob"), event_id("2"));
        summary.add_reaction("❤️", user_id("alice"), event_id("3"));

        assert_eq!(summary.count("👍"), 2);
        assert_eq!(summary.count("❤️"), 1);
        assert_eq!(summary.total(), 3);
    }

    #[test]
    fn test_duplicate_reaction_ignored() {
        let mut summary = ReactionSummary::new();
        summary.add_reaction("👍", user_id("alice"), event_id("1"));
        summary.add_reaction("👍", user_id("alice"), event_id("2"));

        assert_eq!(summary.count("👍"), 1);
        assert_eq!(summary.total(), 1);
    }

    #[test]
    fn test_remove_reaction() {
        let mut summary = ReactionSummary::new();
        summary.add_reaction("👍", user_id("alice"), event_id("1"));
        summary.add_reaction("👍", user_id("bob"), event_id("2"));

        let removed = summary.remove_reaction("👍", &user_id("alice"));
        assert!(removed.is_some());
        assert_eq!(summary.count("👍"), 1);
        assert_eq!(summary.total(), 1);
    }

    #[test]
    fn test_has_user_reacted() {
        let mut summary = ReactionSummary::new();
        summary.add_reaction("👍", user_id("alice"), event_id("1"));

        assert!(summary.has_user_reacted("👍", &user_id("alice")));
        assert!(!summary.has_user_reacted("👍", &user_id("bob")));
        assert!(!summary.has_user_reacted("❤️", &user_id("alice")));
    }

    #[test]
    fn test_top_reactions() {
        let mut summary = ReactionSummary::new();
        summary.add_reaction("👍", user_id("alice"), event_id("1"));
        summary.add_reaction("👍", user_id("bob"), event_id("2"));
        summary.add_reaction("❤️", user_id("alice"), event_id("3"));
        summary.add_reaction("🔥", user_id("alice"), event_id("4"));
        summary.add_reaction("🔥", user_id("bob"), event_id("5"));
        summary.add_reaction("🔥", user_id("charlie"), event_id("6"));

        let top = summary.top_reactions(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "🔥");
        assert_eq!(top[0].1, 3);
        assert_eq!(top[1].0, "👍");
        assert_eq!(top[1].1, 2);
    }

    #[test]
    fn test_reactions_for_display() {
        let mut summary = ReactionSummary::new();
        summary.add_reaction("👍", user_id("alice"), event_id("1"));
        summary.add_reaction("👍", user_id("bob"), event_id("2"));

        let display = reactions_for_display(&summary, Some(&user_id("alice")));
        assert_eq!(display.len(), 1);
        assert_eq!(display[0].emoji, "👍");
        assert_eq!(display[0].count, 2);
        assert!(display[0].is_selected);

        let display_bob = reactions_for_display(&summary, Some(&user_id("charlie")));
        assert!(!display_bob[0].is_selected);
    }
}
