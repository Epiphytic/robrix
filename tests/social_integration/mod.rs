//! Integration tests for social features.
//!
//! These tests validate the social media functionality of Robrix,
//! including profile rooms, feeds, friends, events, and privacy controls.
//!
//! # Running Tests
//!
//! Tests require the `social` feature to be enabled:
//!
//! ```bash
//! cargo test --features social
//! ```
//!
//! # Test Organization
//!
//! - `profile_tests`: Profile room creation and management
//! - `feed_tests`: Feed rooms and post creation
//! - `friend_tests`: Friend network and relationships
//! - `event_tests`: Events, gatherings, and RSVPs
//! - `privacy_tests`: Privacy level enforcement and sharing guards

#[cfg(feature = "social")]
mod profile_tests;

#[cfg(feature = "social")]
mod feed_tests;

#[cfg(feature = "social")]
mod friend_tests;

#[cfg(feature = "social")]
mod event_tests;

#[cfg(feature = "social")]
mod privacy_tests;
