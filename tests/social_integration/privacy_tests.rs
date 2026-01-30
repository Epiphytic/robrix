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
    // Friends content cannot go to Public (without confirmation)
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

    // CloseFriends to Public should be blocked
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::CloseFriends,
        &target,
        PrivacyLevel::Public,
        &[], // mentioned_users
        &[], // target_members
    );

    assert!(matches!(result, ShareValidation::BlockedPrivacyLeak { .. }));
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
        &[], // mentioned_users
        &[], // target_members
    );
    assert!(matches!(result, ShareValidation::Allowed));

    // Friends to CloseFriends is allowed
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Friends,
        &target,
        PrivacyLevel::CloseFriends,
        &[], // mentioned_users
        &[], // target_members
    );
    assert!(matches!(result, ShareValidation::Allowed));

    // Same level is allowed
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Friends,
        &target,
        PrivacyLevel::Friends,
        &[], // mentioned_users
        &[], // target_members
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
        &[], // mentioned_users
        &[], // target_members
    );
    assert!(matches!(result, ShareValidation::BlockedPrivacyLeak { .. }));

    // Private to Public
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Private,
        &target,
        PrivacyLevel::Public,
        &[], // mentioned_users
        &[], // target_members
    );
    assert!(matches!(result, ShareValidation::BlockedPrivacyLeak { .. }));

    // Private to Friends
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Private,
        &target,
        PrivacyLevel::Friends,
        &[], // mentioned_users
        &[], // target_members
    );
    assert!(matches!(result, ShareValidation::BlockedPrivacyLeak { .. }));
}

/// Test SharingGuard with Friends to Public requires confirmation.
///
/// The implementation returns RequiresConfirmation for Friends -> Public
/// as a special case to warn the user.
#[test]
fn test_sharing_guard_friends_to_public_requires_confirmation() {
    use matrix_sdk::ruma::OwnedRoomId;

    let source: OwnedRoomId = "!source:example.org".try_into().unwrap();
    let target: OwnedRoomId = "!target:example.org".try_into().unwrap();

    // Friends to Public requires confirmation
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Friends,
        &target,
        PrivacyLevel::Public,
        &[], // mentioned_users
        &[], // target_members
    );
    assert!(matches!(
        result,
        ShareValidation::RequiresConfirmation { .. }
    ));
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

/// Test SharingGuard quote validation.
#[test]
fn test_sharing_guard_validate_quote() {
    // Quote from more private room requires confirmation
    let result = SharingGuard::validate_quote(PrivacyLevel::Friends, PrivacyLevel::Public);
    assert!(matches!(
        result,
        ShareValidation::RequiresConfirmation { .. }
    ));

    // Quote from same or less private room is allowed
    let result = SharingGuard::validate_quote(PrivacyLevel::Public, PrivacyLevel::Friends);
    assert!(matches!(result, ShareValidation::Allowed));

    let result = SharingGuard::validate_quote(PrivacyLevel::Friends, PrivacyLevel::Friends);
    assert!(matches!(result, ShareValidation::Allowed));
}

/// Test missing mentions detection.
#[test]
fn test_sharing_guard_missing_mentions() {
    use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};

    let source: OwnedRoomId = "!source:example.org".try_into().unwrap();
    let target: OwnedRoomId = "!target:example.org".try_into().unwrap();
    let mentioned_user: OwnedUserId = "@alice:example.org".try_into().unwrap();
    let other_user: OwnedUserId = "@bob:example.org".try_into().unwrap();

    // Mention a user not in target room
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Public,
        &target,
        PrivacyLevel::Public,
        &[mentioned_user.clone()], // mentioned_users
        &[other_user],             // target_members (does not include mentioned_user)
    );
    assert!(matches!(result, ShareValidation::MissingMentions { .. }));

    // When mentioned user is in target room, should be allowed
    let result = SharingGuard::validate_share(
        &source,
        PrivacyLevel::Public,
        &target,
        PrivacyLevel::Public,
        &[mentioned_user.clone()], // mentioned_users
        &[mentioned_user],         // target_members (includes mentioned_user)
    );
    assert!(matches!(result, ShareValidation::Allowed));
}
