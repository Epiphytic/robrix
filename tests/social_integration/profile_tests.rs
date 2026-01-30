//! Profile room tests.
//!
//! Tests for profile room creation, discovery, and management.

use robrix::social::{ProfileRoomConfig, ProfileRoomError};

/// Test that default profile room config has expected values.
#[test]
fn test_default_profile_room_config() {
    let config = ProfileRoomConfig::default();
    assert_eq!(config.alias_prefix, "profile_");
}

/// Test that ProfileRoomError variants are properly defined.
#[test]
fn test_profile_room_error_display() {
    let not_logged_in = ProfileRoomError::NotLoggedIn;
    assert_eq!(not_logged_in.to_string(), "Not logged in");

    let room_not_found = ProfileRoomError::RoomNotFound;
    assert_eq!(room_not_found.to_string(), "Room not found");

    let invalid_alias = ProfileRoomError::InvalidAlias;
    assert_eq!(invalid_alias.to_string(), "Invalid room alias");
}

/// Test that profile room errors implement Debug.
#[test]
fn test_profile_room_error_debug() {
    let error = ProfileRoomError::NotLoggedIn;
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("NotLoggedIn"));
}

/// Test that AlreadyExists error contains the room ID.
#[test]
fn test_profile_room_already_exists_error() {
    use matrix_sdk::ruma::OwnedRoomId;

    let room_id: OwnedRoomId = "!test:example.org".try_into().unwrap();
    let error = ProfileRoomError::AlreadyExists(room_id.clone());

    let error_str = error.to_string();
    assert!(error_str.contains("!test:example.org"));
    assert!(error_str.contains("already exists"));
}

// Note: Full integration tests requiring a Matrix client connection
// would be added here when running against a test homeserver.
// For now, we test the types and error handling that don't require
// actual network connections.

/// Placeholder for async profile room creation test.
/// This would require a mock Matrix client or test homeserver.
#[test]
#[ignore = "requires Matrix homeserver connection"]
fn test_create_profile_room() {
    // TODO: Implement with mock client
    // let client = create_mock_client();
    // let service = ProfileRoomService::new(client);
    // let profile = SocialProfileEventContent { ... };
    // let result = service.create_profile_room(profile).await;
    // assert!(result.is_ok());
}

/// Placeholder for async profile room discovery test.
#[test]
#[ignore = "requires Matrix homeserver connection"]
fn test_find_profile_room() {
    // TODO: Implement with mock client
}

/// Placeholder for async profile update test.
#[test]
#[ignore = "requires Matrix homeserver connection"]
fn test_update_profile() {
    // TODO: Implement with mock client
}
