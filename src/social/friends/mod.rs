//! Friend network management for social features.
//!
//! This module provides the friend relationship system, including:
//! - Friends space management (organizing friends in a Matrix space)
//! - Friend request flow (send, accept, decline requests using Matrix knock)
//!
//! ## Architecture
//!
//! Friends are tracked using Matrix spaces and room membership:
//! - Each user has a private "friends space" containing their friends' feed rooms
//! - Friend requests use the Matrix knock mechanism on friends-only feed rooms
//! - Mutual friendship requires bidirectional membership (both users in each other's spaces)
//!
//! ## Example
//!
//! ```rust,ignore
//! use matrix_sdk::Client;
//! use crate::social::friends::{FriendsSpaceService, FriendRequestService};
//!
//! // Create services
//! let friends_service = FriendsSpaceService::new(client.clone());
//! let request_service = FriendRequestService::new(client.clone());
//!
//! // Send a friend request
//! request_service.send_friend_request(&target_feed_room).await?;
//!
//! // Accept a friend request
//! request_service.accept_friend_request(&requester_id, &our_feed).await?;
//!
//! // Add friend to our space
//! friends_service.add_friend(&friend_feed_room).await?;
//! ```

pub mod friend_request;
pub mod friends_space;

pub use friend_request::{
    FriendRequestError, FriendRequestService, FriendRequestState, PendingFriendRequest,
};
pub use friends_space::{FriendsError, FriendsSpaceService};
