//! Friend request flow using Matrix knock mechanism.
//!
//! Friend requests are implemented using Matrix's knock feature:
//! - Sending a request = knocking on the target's friends-only feed room
//! - Accepting a request = inviting the requester to our friends feed
//! - Declining a request = rejecting the knock (kick from knock state)

use matrix_sdk::{
    ruma::{
        MilliSecondsSinceUnixEpoch, OwnedRoomId, OwnedRoomOrAliasId, OwnedUserId, RoomId, UserId,
    },
    Client,
};

/// Friend request state between two users.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FriendRequestState {
    /// No relationship established
    None,
    /// Pending outgoing request (we knocked on their feed)
    PendingOutgoing,
    /// Pending incoming request (they knocked on our feed)
    PendingIncoming,
    /// Accepted (mutual friends - both have access to each other's friends feed)
    Friends,
    /// Blocked (relationship is blocked)
    Blocked,
}

/// A pending friend request.
#[derive(Clone, Debug)]
pub struct PendingFriendRequest {
    /// The user who sent the request
    pub requester: OwnedUserId,
    /// The room they knocked on (our friends-only feed)
    pub room_id: OwnedRoomId,
    /// When the request was sent
    pub timestamp: MilliSecondsSinceUnixEpoch,
    /// Requester's display name (if available)
    pub display_name: Option<String>,
    /// Requester's avatar URL (if available)
    pub avatar_url: Option<String>,
}

/// Service for handling friend requests.
///
/// This service manages the friend request flow using Matrix's knock mechanism
/// as the underlying protocol for friend requests.
pub struct FriendRequestService {
    client: Client,
}

impl FriendRequestService {
    /// Create a new FriendRequestService.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Send a friend request by knocking on their friends-only feed.
    ///
    /// This initiates a friend request by "knocking" on the target user's
    /// friends-only feed room. The target user can then accept or decline.
    pub async fn send_friend_request(
        &self,
        target_friends_feed: &RoomId,
    ) -> Result<(), FriendRequestError> {
        // Convert RoomId to OwnedRoomOrAliasId for the knock API
        let room_or_alias: OwnedRoomOrAliasId = target_friends_feed.to_owned().into();

        // Knock on the room to request access (reason: None, server_names: empty)
        self.client
            .knock(room_or_alias, None, vec![])
            .await
            .map_err(FriendRequestError::MatrixError)?;

        Ok(())
    }

    /// Send a friend request with a custom message.
    ///
    /// Some Matrix implementations support a reason field in knocks,
    /// which can be used to include a personal message with the request.
    pub async fn send_friend_request_with_message(
        &self,
        target_friends_feed: &RoomId,
        message: &str,
    ) -> Result<(), FriendRequestError> {
        // Convert RoomId to OwnedRoomOrAliasId for the knock API
        let room_or_alias: OwnedRoomOrAliasId = target_friends_feed.to_owned().into();

        // Knock on the room with a reason/message
        self.client
            .knock(room_or_alias, Some(message.to_string()), vec![])
            .await
            .map_err(FriendRequestError::MatrixError)?;

        Ok(())
    }

    /// Accept a friend request (invite them to our friends feed).
    ///
    /// When accepting a friend request, we invite the requester to our
    /// friends-only feed room, completing the bidirectional friendship.
    pub async fn accept_friend_request(
        &self,
        requester: &UserId,
        our_friends_feed: &RoomId,
    ) -> Result<(), FriendRequestError> {
        let room = self
            .client
            .get_room(our_friends_feed)
            .ok_or(FriendRequestError::RoomNotFound)?;

        // Invite the requester to our friends feed
        room.invite_user_by_id(requester)
            .await
            .map_err(FriendRequestError::MatrixError)?;

        Ok(())
    }

    /// Decline a friend request.
    ///
    /// This rejects the knock by kicking the user from the knock state.
    /// The requester will be notified that their request was declined.
    pub async fn decline_friend_request(
        &self,
        requester: &UserId,
        our_friends_feed: &RoomId,
    ) -> Result<(), FriendRequestError> {
        let room = self
            .client
            .get_room(our_friends_feed)
            .ok_or(FriendRequestError::RoomNotFound)?;

        // Kick the user from knock state (rejects the request)
        room.kick_user(requester, Some("Friend request declined"))
            .await
            .map_err(FriendRequestError::MatrixError)?;

        Ok(())
    }

    /// Get pending incoming friend requests.
    ///
    /// Returns a list of users who have knocked on our friends-only feed
    /// rooms and are waiting for a response.
    pub async fn get_pending_requests(
        &self,
    ) -> Result<Vec<PendingFriendRequest>, FriendRequestError> {
        let pending = Vec::new();

        // TODO: Implement full knock state retrieval
        // This requires:
        // 1. Iterating through our friends-only feed rooms
        // 2. Fetching room members for each room
        // 3. Filtering for members in MembershipState::Knock
        // 4. Building PendingFriendRequest structs with profile info

        Ok(pending)
    }

    /// Get the friend request state with a specific user.
    ///
    /// Determines the current relationship state between the current user
    /// and the specified target user.
    pub async fn get_request_state(
        &self,
        _target_user: &UserId,
        _their_friends_feed: Option<&RoomId>,
        _our_friends_feed: Option<&RoomId>,
    ) -> Result<FriendRequestState, FriendRequestError> {
        // Check various membership states to determine relationship:
        // 1. If we're in their friends feed AND they're in ours = Friends
        // 2. If we knocked on their feed = PendingOutgoing
        // 3. If they knocked on our feed = PendingIncoming
        // 4. Otherwise = None

        // TODO: Implement full state determination
        // This requires checking membership states in both directions

        Ok(FriendRequestState::None)
    }

    /// Cancel a pending outgoing friend request.
    ///
    /// Withdraws a knock request that hasn't been accepted yet.
    pub async fn cancel_friend_request(
        &self,
        target_friends_feed: &RoomId,
    ) -> Result<(), FriendRequestError> {
        // Leave the room (withdraws the knock)
        if let Some(room) = self.client.get_room(target_friends_feed) {
            room.leave()
                .await
                .map_err(FriendRequestError::MatrixError)?;
        }

        Ok(())
    }

    /// Block a user (prevents future friend requests).
    ///
    /// Blocking prevents the user from sending friend requests and
    /// removes any existing friendship.
    pub async fn block_user(
        &self,
        user_id: &UserId,
        our_friends_feed: &RoomId,
    ) -> Result<(), FriendRequestError> {
        let room = self
            .client
            .get_room(our_friends_feed)
            .ok_or(FriendRequestError::RoomNotFound)?;

        // Ban the user from our friends feed
        room.ban_user(user_id, Some("User blocked"))
            .await
            .map_err(FriendRequestError::MatrixError)?;

        Ok(())
    }

    /// Unblock a user.
    ///
    /// Removes the block, allowing the user to send friend requests again.
    pub async fn unblock_user(
        &self,
        user_id: &UserId,
        our_friends_feed: &RoomId,
    ) -> Result<(), FriendRequestError> {
        let room = self
            .client
            .get_room(our_friends_feed)
            .ok_or(FriendRequestError::RoomNotFound)?;

        // Unban the user
        room.unban_user(user_id, Some("User unblocked"))
            .await
            .map_err(FriendRequestError::MatrixError)?;

        Ok(())
    }
}

/// Errors that can occur when handling friend requests.
#[derive(Debug, thiserror::Error)]
pub enum FriendRequestError {
    /// The specified room was not found.
    #[error("Room not found")]
    RoomNotFound,

    /// User is not logged in to the Matrix client.
    #[error("Not logged in")]
    NotLoggedIn,

    /// A friend request is already pending.
    #[error("Friend request already pending")]
    RequestAlreadyPending,

    /// The users are already friends.
    #[error("Already friends")]
    AlreadyFriends,

    /// The user is blocked.
    #[error("User is blocked")]
    UserBlocked,

    /// Cannot send request to self.
    #[error("Cannot send friend request to yourself")]
    CannotFriendSelf,

    /// An error occurred in the Matrix SDK.
    #[error("Matrix error: {0}")]
    MatrixError(#[from] matrix_sdk::Error),
}
