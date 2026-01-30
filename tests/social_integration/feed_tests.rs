//! Feed and post tests.
//!
//! Tests for feed room management and post creation.

use robrix::social::{FeedPrivacy, FeedRoomError, Post, PostContent, UserFeeds};

/// Test FeedPrivacy ordering (less restrictive < more restrictive).
#[test]
fn test_feed_privacy_ordering() {
    assert!(FeedPrivacy::Public < FeedPrivacy::Friends);
    assert!(FeedPrivacy::Friends < FeedPrivacy::CloseFriends);

    // Verify full ordering
    assert!(FeedPrivacy::Public < FeedPrivacy::CloseFriends);
}

/// Test FeedPrivacy display names.
#[test]
fn test_feed_privacy_display() {
    assert_eq!(FeedPrivacy::Public.to_string(), "Public Feed");
    assert_eq!(FeedPrivacy::Friends.to_string(), "Friends Feed");
    assert_eq!(FeedPrivacy::CloseFriends.to_string(), "Close Friends Feed");
}

/// Test FeedPrivacy alias suffixes.
#[test]
fn test_feed_privacy_alias_suffix() {
    assert_eq!(FeedPrivacy::Public.alias_suffix(), "_public");
    assert_eq!(FeedPrivacy::Friends.alias_suffix(), "_friends");
    assert_eq!(FeedPrivacy::CloseFriends.alias_suffix(), "_close");
}

/// Test UserFeeds default state.
#[test]
fn test_user_feeds_default() {
    let feeds = UserFeeds::default();
    assert!(!feeds.has_any());
    assert!(feeds.public.is_none());
    assert!(feeds.friends.is_none());
    assert!(feeds.close_friends.is_none());
}

/// Test UserFeeds has_any detection.
#[test]
fn test_user_feeds_has_any() {
    use matrix_sdk::ruma::OwnedRoomId;

    let room_id: OwnedRoomId = "!test:example.org".try_into().unwrap();

    let with_public = UserFeeds {
        public: Some(room_id.clone()),
        ..Default::default()
    };
    assert!(with_public.has_any());

    let with_friends = UserFeeds {
        friends: Some(room_id.clone()),
        ..Default::default()
    };
    assert!(with_friends.has_any());

    let with_close = UserFeeds {
        close_friends: Some(room_id),
        ..Default::default()
    };
    assert!(with_close.has_any());
}

/// Test UserFeeds get method.
#[test]
fn test_user_feeds_get() {
    use matrix_sdk::ruma::OwnedRoomId;

    let public_room: OwnedRoomId = "!public:example.org".try_into().unwrap();
    let friends_room: OwnedRoomId = "!friends:example.org".try_into().unwrap();

    let feeds = UserFeeds {
        public: Some(public_room.clone()),
        friends: Some(friends_room.clone()),
        close_friends: None,
    };

    assert_eq!(feeds.get(FeedPrivacy::Public), Some(&public_room));
    assert_eq!(feeds.get(FeedPrivacy::Friends), Some(&friends_room));
    assert_eq!(feeds.get(FeedPrivacy::CloseFriends), None);
}

/// Test UserFeeds all method.
#[test]
fn test_user_feeds_all() {
    use matrix_sdk::ruma::OwnedRoomId;

    let room1: OwnedRoomId = "!room1:example.org".try_into().unwrap();
    let room2: OwnedRoomId = "!room2:example.org".try_into().unwrap();

    let feeds = UserFeeds {
        public: Some(room1),
        friends: Some(room2),
        close_friends: None,
    };

    let all = feeds.all();
    assert_eq!(all.len(), 2);
}

/// Test Post text creation.
#[test]
fn test_post_text_creation() {
    let post = Post::text("Hello, world!");
    assert!(matches!(
        post.content,
        PostContent::Text { body, .. } if body == "Hello, world!"
    ));
    assert_eq!(post.privacy_levels, vec![FeedPrivacy::Public]);
}

/// Test Post with privacy modifier.
#[test]
fn test_post_with_privacy() {
    let post = Post::text("Friends only post").with_privacy(vec![FeedPrivacy::Friends]);
    assert_eq!(post.privacy_levels, vec![FeedPrivacy::Friends]);
}

/// Test Post with targets modifier.
#[test]
fn test_post_with_targets() {
    use matrix_sdk::ruma::OwnedRoomId;

    let room: OwnedRoomId = "!target:example.org".try_into().unwrap();
    let post = Post::text("Targeted post").with_targets(vec![room.clone()]);
    assert_eq!(post.targets, vec![room]);
}

/// Test Post image creation with caption.
#[test]
fn test_post_image_with_caption() {
    use matrix_sdk::ruma::OwnedMxcUri;

    let mxc: OwnedMxcUri = "mxc://example.org/abc123".try_into().unwrap();
    let post = Post::image(mxc, 800, 600).with_caption("A beautiful sunset");

    assert!(matches!(
        post.content,
        PostContent::Image { caption: Some(c), width: 800, height: 600, .. } if c == "A beautiful sunset"
    ));
}

/// Test Post video creation.
#[test]
fn test_post_video_creation() {
    use matrix_sdk::ruma::OwnedMxcUri;

    let mxc: OwnedMxcUri = "mxc://example.org/video456".try_into().unwrap();
    let post = Post::video(mxc.clone());

    assert!(matches!(post.content, PostContent::Video { mxc_uri, .. } if mxc_uri == mxc));
}

/// Test Post link creation.
#[test]
fn test_post_link_creation() {
    let url: url::Url = "https://example.org/article".parse().unwrap();
    let post = Post::link(url.clone()).with_caption("Check this out!");

    assert!(
        matches!(post.content, PostContent::Link { url: u, comment: Some(c), .. } if u == url && c == "Check this out!")
    );
}

/// Test PostContent to room message conversion.
#[test]
fn test_post_to_room_message() {
    use matrix_sdk::ruma::events::room::message::MessageType;

    let post = Post::text("Test message");
    let msg = post.into_room_message();
    assert!(matches!(msg.msgtype, MessageType::Text(_)));
}

/// Test FeedRoomError display.
#[test]
fn test_feed_room_error_display() {
    let not_logged_in = FeedRoomError::NotLoggedIn;
    assert_eq!(not_logged_in.to_string(), "Not logged in");

    let feed_not_found = FeedRoomError::FeedNotFound;
    assert_eq!(feed_not_found.to_string(), "Feed room not found");

    let access_denied = FeedRoomError::AccessDenied;
    assert_eq!(access_denied.to_string(), "Access denied to feed room");
}

/// Test FeedRoomError AlreadyExists variant.
#[test]
fn test_feed_room_already_exists_error() {
    let error = FeedRoomError::AlreadyExists(FeedPrivacy::Public);
    let error_str = error.to_string();
    assert!(error_str.contains("Public"));
    assert!(error_str.contains("already exists"));
}

// Async tests requiring Matrix client connection
#[test]
#[ignore = "requires Matrix homeserver connection"]
fn test_create_feed_room() {
    // TODO: Implement with mock client
}

#[test]
#[ignore = "requires Matrix homeserver connection"]
fn test_join_feed() {
    // TODO: Implement with mock client
}
