//! Privacy safeguard tests.
//!
//! Tests for privacy level enforcement and sharing guards that prevent
//! content from being leaked to less restrictive audiences.

use robrix::social::privacy::{PrivacyLevel, ShareValidation, SharingGuard};

/// Test privacy level ordering (less restrictive < more restrictive).
///
/// This ordering is fundamental to the sharing rules:
/// - Public (0) < Friends (1) < CloseFriends (2) < Private (3)
#[test]
fn test_privacy_level_ordering() {
    assert!(PrivacyLevel::Public < PrivacyLevel::Friends);
    assert!(PrivacyLevel::Friends < PrivacyLevel::CloseFriends);
    assert!(PrivacyLevel::CloseFriends < PrivacyLevel::Private);

    // Transitive ordering
    assert!(PrivacyLevel::Public < PrivacyLevel::CloseFriends);
    assert!(PrivacyLevel::Public < PrivacyLevel::Private);
    assert!(PrivacyLevel::Friends < PrivacyLevel::Private);
}

/// Test that content cannot be shared from restrictive to less restrictive destinations.
///
/// This prevents privacy leaks where friends-only content ends up public.
#[test]
fn test_cannot_share_private_to_public() {
    // Friends content cannot go to Public
    assert!(!PrivacyLevel::Friends.can_share_to(PrivacyLevel::Public));

    // CloseFriends content cannot go to Public
    assert!(!PrivacyLevel::CloseFriends.can_share_to(PrivacyLevel::Public));

    // Private content cannot go to Public
    assert!(!PrivacyLevel::Private.can_share_to(PrivacyLevel::Public));

    // CloseFriends content cannot go to Friends
    assert!(!PrivacyLevel::CloseFriends.can_share_to(PrivacyLevel::Friends));

    // Private content cannot go to Friends
    assert!(!PrivacyLevel::Private.can_share_to(PrivacyLevel::Friends));

    // Private content cannot go to CloseFriends
    assert!(!PrivacyLevel::Private.can_share_to(PrivacyLevel::CloseFriends));
}

/// Test that public content can be shared anywhere.
///
/// Public content has no restrictions on where it can be shared.
#[test]
fn test_can_share_public_anywhere() {
    assert!(PrivacyLevel::Public.can_share_to(PrivacyLevel::Public));
    assert!(PrivacyLevel::Public.can_share_to(PrivacyLevel::Friends));
    assert!(PrivacyLevel::Public.can_share_to(PrivacyLevel::CloseFriends));
    assert!(PrivacyLevel::Public.can_share_to(PrivacyLevel::Private));
}

/// Test that SharingGuard blocks privacy leaks.
///
/// The sharing guard should prevent cross-posting from a more restrictive
/// source to a less restrictive destination.
#[test]
fn test_sharing_guard_blocks_leak() {
    use matrix_sdk::ruma::OwnedRoomId;

    let source: OwnedRoomId = "!source:example.org".try_into().unwrap();
    let target: OwnedRoomId = "!target:example.org".try_into().unwrap();

    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Friends,
        &target,
        PrivacyLevel::Public,
        &[],
        &[],
    );

    assert!(matches!(result, ShareValidation::BlockedPrivacyLeak { .. }));

    // Verify the error contains the right privacy levels
    if let ShareValidation::BlockedPrivacyLeak {
        source_privacy,
        target_privacy,
    } = result
    {
        assert_eq!(source_privacy, PrivacyLevel::Friends);
        assert_eq!(target_privacy, PrivacyLevel::Public);
    }
}

/// Test that content can be shared to same privacy level.
#[test]
fn test_can_share_to_same_level() {
    assert!(PrivacyLevel::Public.can_share_to(PrivacyLevel::Public));
    assert!(PrivacyLevel::Friends.can_share_to(PrivacyLevel::Friends));
    assert!(PrivacyLevel::CloseFriends.can_share_to(PrivacyLevel::CloseFriends));
    assert!(PrivacyLevel::Private.can_share_to(PrivacyLevel::Private));
}

/// Test that content can be shared to more restrictive levels.
#[test]
fn test_can_share_to_more_restrictive() {
    // Friends can share to CloseFriends and Private
    assert!(PrivacyLevel::Friends.can_share_to(PrivacyLevel::CloseFriends));
    assert!(PrivacyLevel::Friends.can_share_to(PrivacyLevel::Private));

    // CloseFriends can share to Private
    assert!(PrivacyLevel::CloseFriends.can_share_to(PrivacyLevel::Private));
}

/// Test SharingGuard allows valid shares.
#[test]
fn test_sharing_guard_allows_valid() {
    use matrix_sdk::ruma::OwnedRoomId;

    let source: OwnedRoomId = "!source:example.org".try_into().unwrap();
    let target: OwnedRoomId = "!target:example.org".try_into().unwrap();

    // Public to Friends is allowed
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Public,
        &target,
        PrivacyLevel::Friends,
        &[],
        &[],
    );
    assert!(matches!(result, ShareValidation::Allowed));

    // Friends to CloseFriends is allowed
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Friends,
        &target,
        PrivacyLevel::CloseFriends,
        &[],
        &[],
    );
    assert!(matches!(result, ShareValidation::Allowed));

    // Same level is allowed
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Friends,
        &target,
        PrivacyLevel::Friends,
        &[],
        &[],
    );
    assert!(matches!(result, ShareValidation::Allowed));
}

/// Test SharingGuard blocks various leak scenarios.
#[test]
fn test_sharing_guard_blocks_various_leaks() {
    use matrix_sdk::ruma::OwnedRoomId;

    let source: OwnedRoomId = "!source:example.org".try_into().unwrap();
    let target: OwnedRoomId = "!target:example.org".try_into().unwrap();

    // CloseFriends to Public
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::CloseFriends,
        &target,
        PrivacyLevel::Public,
        &[],
        &[],
    );
    assert!(matches!(result, ShareValidation::BlockedPrivacyLeak { .. }));

    // Private to Public
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Private,
        &target,
        PrivacyLevel::Public,
        &[],
        &[],
    );
    assert!(matches!(result, ShareValidation::BlockedPrivacyLeak { .. }));

    // Private to Friends
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Private,
        &target,
        PrivacyLevel::Friends,
        &[],
        &[],
    );
    assert!(matches!(result, ShareValidation::BlockedPrivacyLeak { .. }));
}

/// Test SharingGuard with attachments.
#[test]
fn test_sharing_guard_attachment_privacy() {
    use matrix_sdk::ruma::OwnedRoomId;

    let source: OwnedRoomId = "!source:example.org".try_into().unwrap();
    let target: OwnedRoomId = "!target:example.org".try_into().unwrap();
    let attachment_room: OwnedRoomId = "!attachment:example.org".try_into().unwrap();

    // Sharing public content with a friends-only attachment to public should fail
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Public,
        &target,
        PrivacyLevel::Public,
        &[],
        &[(attachment_room.clone(), PrivacyLevel::Friends)],
    );
    assert!(matches!(
        result,
        ShareValidation::AttachmentPrivacyMismatch { .. }
    ));

    // Sharing public content with a public attachment to public should succeed
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Public,
        &target,
        PrivacyLevel::Public,
        &[],
        &[(attachment_room, PrivacyLevel::Public)],
    );
    assert!(matches!(result, ShareValidation::Allowed));
}

/// Test SharingGuard convenience method.
#[test]
fn test_sharing_guard_can_share() {
    assert!(SharingGuard::can_share(
        PrivacyLevel::Public,
        PrivacyLevel::Friends
    ));
    assert!(SharingGuard::can_share(
        PrivacyLevel::Friends,
        PrivacyLevel::Friends
    ));
    assert!(!SharingGuard::can_share(
        PrivacyLevel::Friends,
        PrivacyLevel::Public
    ));
}

/// Test ShareValidation is_allowed method.
#[test]
fn test_share_validation_is_allowed() {
    assert!(ShareValidation::Allowed.is_allowed());

    let blocked = ShareValidation::BlockedPrivacyLeak {
        source_privacy: PrivacyLevel::Friends,
        target_privacy: PrivacyLevel::Public,
    };
    assert!(!blocked.is_allowed());
}

/// Test ShareValidation error_message method.
#[test]
fn test_share_validation_error_message() {
    assert!(ShareValidation::Allowed.error_message().is_none());

    let blocked = ShareValidation::BlockedPrivacyLeak {
        source_privacy: PrivacyLevel::Friends,
        target_privacy: PrivacyLevel::Public,
    };
    let msg = blocked.error_message().unwrap();
    assert!(msg.contains("Friends"));
    assert!(msg.contains("Public"));
}

/// Test PrivacyLevel display.
#[test]
fn test_privacy_level_display() {
    assert_eq!(PrivacyLevel::Public.to_string(), "Public");
    assert_eq!(PrivacyLevel::Friends.to_string(), "Friends");
    assert_eq!(PrivacyLevel::CloseFriends.to_string(), "Close Friends");
    assert_eq!(PrivacyLevel::Private.to_string(), "Private");
}

/// Test PrivacyLevel description.
#[test]
fn test_privacy_level_description() {
    assert!(PrivacyLevel::Public.description().contains("anyone"));
    assert!(PrivacyLevel::Friends.description().contains("Friends"));
    assert!(PrivacyLevel::CloseFriends.description().contains("Close"));
    assert!(PrivacyLevel::Private.description().contains("only you"));
}

/// Test PrivacyLevel default.
#[test]
fn test_privacy_level_default() {
    let default = PrivacyLevel::default();
    assert_eq!(default, PrivacyLevel::Public);
}

/// Test PrivacyLevel clone and copy.
#[test]
fn test_privacy_level_clone_copy() {
    let level = PrivacyLevel::Friends;
    let cloned = level.clone();
    let copied = level;

    assert_eq!(level, cloned);
    assert_eq!(level, copied);
}
