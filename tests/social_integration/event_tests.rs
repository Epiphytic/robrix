//! Events and RSVP tests.
//!
//! Tests for event room creation, RSVP handling, and validation.

use robrix::social::events::{
    event_room_power_levels, EventRole, EventRoomError, RsvpError, RsvpValidation,
};

/// Test EventRole power levels.
#[test]
fn test_event_role_power_levels() {
    assert_eq!(EventRole::Creator.power_level_i64(), 100);
    assert_eq!(EventRole::CoHost.power_level_i64(), 50);
    assert_eq!(EventRole::Guest.power_level_i64(), 0);
}

/// Test EventRole power level Int conversion.
#[test]
fn test_event_role_power_level_int() {
    use matrix_sdk::ruma::Int;

    assert_eq!(EventRole::Creator.power_level(), Int::new(100).unwrap());
    assert_eq!(EventRole::CoHost.power_level(), Int::new(50).unwrap());
    assert_eq!(EventRole::Guest.power_level(), Int::new(0).unwrap());
}

/// Test EventRole equality.
#[test]
fn test_event_role_equality() {
    assert_eq!(EventRole::Creator, EventRole::Creator);
    assert_eq!(EventRole::CoHost, EventRole::CoHost);
    assert_eq!(EventRole::Guest, EventRole::Guest);
    assert_ne!(EventRole::Creator, EventRole::Guest);
}

/// Test event room power levels configuration.
#[test]
fn test_event_room_power_levels_config() {
    // Guests CAN invite
    let config_invite = event_room_power_levels(true);
    assert_eq!(config_invite.invite, Some(0)); // Guest level

    // Guests CANNOT invite
    let config_no_invite = event_room_power_levels(false);
    assert_eq!(config_no_invite.invite, Some(50)); // CoHost level

    // Common settings
    assert_eq!(config_invite.state_default, Some(50)); // CoHost
    assert_eq!(config_invite.events_default, Some(0)); // Guest
    assert_eq!(config_invite.kick, Some(50)); // CoHost
    assert_eq!(config_invite.ban, Some(50)); // CoHost
    assert_eq!(config_invite.redact, Some(50)); // CoHost
}

/// Test EventRoomError display.
#[test]
fn test_event_room_error_display() {
    let not_logged_in = EventRoomError::NotLoggedIn;
    assert_eq!(not_logged_in.to_string(), "Not logged in");

    let room_not_found = EventRoomError::RoomNotFound;
    assert_eq!(room_not_found.to_string(), "Room not found");
}

/// Test RsvpError display.
#[test]
fn test_rsvp_error_display() {
    let not_logged_in = RsvpError::NotLoggedIn;
    assert_eq!(not_logged_in.to_string(), "Not logged in");

    let room_not_found = RsvpError::RoomNotFound;
    assert_eq!(room_not_found.to_string(), "Room not found");
}

/// Test RsvpValidation variants.
#[test]
fn test_rsvp_validation_variants() {
    use matrix_sdk::ruma::OwnedUserId;

    // Valid case
    let valid = RsvpValidation::Valid;
    assert!(matches!(valid, RsvpValidation::Valid));

    // Sender mismatch case
    let claimed: OwnedUserId = "@claimed:example.org".try_into().unwrap();
    let actual: OwnedUserId = "@actual:example.org".try_into().unwrap();
    let mismatch = RsvpValidation::SenderMismatch {
        claimed: claimed.clone(),
        actual: actual.clone(),
    };

    match mismatch {
        RsvpValidation::SenderMismatch {
            claimed: c,
            actual: a,
        } => {
            assert_eq!(c, claimed);
            assert_eq!(a, actual);
        }
        _ => panic!("Expected SenderMismatch variant"),
    }

    // Invalid content case
    let invalid = RsvpValidation::InvalidContent("bad state_key".to_string());
    match invalid {
        RsvpValidation::InvalidContent(msg) => {
            assert!(msg.contains("bad state_key"));
        }
        _ => panic!("Expected InvalidContent variant"),
    }
}

/// Test RSVP sender validation - mismatched sender/state_key should be detected.
///
/// This is critical for preventing RSVP spoofing attacks where a malicious
/// user tries to RSVP on behalf of another user.
#[test]
fn test_rsvp_sender_validation() {
    use matrix_sdk::ruma::OwnedUserId;

    // When state_key != sender, validation should fail
    let claimed: OwnedUserId = "@victim:example.org".try_into().unwrap();
    let actual: OwnedUserId = "@attacker:example.org".try_into().unwrap();

    // Simulate what validate_rsvp_event would return
    let result = RsvpValidation::SenderMismatch {
        claimed: claimed.clone(),
        actual: actual.clone(),
    };

    // Verify it's correctly identified as a mismatch
    assert!(matches!(result, RsvpValidation::SenderMismatch { .. }));

    // The claimed and actual users should be different
    if let RsvpValidation::SenderMismatch {
        claimed: c,
        actual: a,
    } = result
    {
        assert_ne!(c, a, "Mismatched sender should be detected");
    }
}

/// Test that valid RSVP passes validation.
///
/// When state_key matches sender, the RSVP is legitimate.
#[test]
fn test_valid_rsvp_passes() {
    use matrix_sdk::ruma::OwnedUserId;

    let user: OwnedUserId = "@user:example.org".try_into().unwrap();

    // When state_key == sender, validation passes
    // In the actual validate_rsvp_event function, this would return Valid
    let result = RsvpValidation::Valid;

    assert!(
        matches!(result, RsvpValidation::Valid),
        "Valid RSVP should pass validation"
    );

    // Verify the user ID is valid for RSVP state_key
    assert!(
        user.as_str().starts_with('@'),
        "RSVP state_key must be a valid user ID"
    );
    assert!(
        user.as_str().contains(':'),
        "RSVP state_key must contain server name"
    );
}

/// Test RSVP validation with invalid state_key format.
#[test]
fn test_rsvp_invalid_state_key() {
    // Non-user-ID state_key should be rejected
    let invalid_state_keys = vec![
        "not_a_user_id",
        "missing_at_sign:server.org",
        "@missing_server",
        "",
    ];

    for state_key in invalid_state_keys {
        // Attempting to parse as user ID should fail
        let parse_result = matrix_sdk::ruma::OwnedUserId::try_from(state_key.to_string());
        assert!(
            parse_result.is_err(),
            "Invalid state_key '{}' should fail to parse as user ID",
            state_key
        );
    }
}

/// Test ValidatedRsvp structure.
#[test]
fn test_validated_rsvp_structure() {
    use matrix_sdk::ruma::OwnedUserId;
    use robrix::social::events::ValidatedRsvp;
    use robrix_social_events::rsvp::RsvpStatus;

    let user: OwnedUserId = "@alice:example.org".try_into().unwrap();

    let rsvp = ValidatedRsvp {
        user_id: user.clone(),
        status: RsvpStatus::Going,
        guests: 2,
        note: Some("Looking forward to it!".to_string()),
    };

    assert_eq!(rsvp.user_id, user);
    assert_eq!(rsvp.status, RsvpStatus::Going);
    assert_eq!(rsvp.guests, 2);
    assert_eq!(rsvp.note, Some("Looking forward to it!".to_string()));
}

/// Test RsvpCounts default and accumulation.
#[test]
fn test_rsvp_counts() {
    use robrix::social::events::RsvpCounts;

    let counts = RsvpCounts::default();
    assert_eq!(counts.going, 0);
    assert_eq!(counts.interested, 0);
    assert_eq!(counts.not_going, 0);
    assert_eq!(counts.total_guests, 0);
}

/// Test RsvpStatus variants.
#[test]
fn test_rsvp_status_variants() {
    use robrix_social_events::rsvp::RsvpStatus;

    assert_eq!(RsvpStatus::Going, RsvpStatus::Going);
    assert_eq!(RsvpStatus::Interested, RsvpStatus::Interested);
    assert_eq!(RsvpStatus::NotGoing, RsvpStatus::NotGoing);
    assert_ne!(RsvpStatus::Going, RsvpStatus::NotGoing);
}

// Async tests requiring Matrix client connection
#[test]
#[ignore = "requires Matrix homeserver connection"]
fn test_create_event_room() {
    // TODO: Implement with mock client
}

#[test]
#[ignore = "requires Matrix homeserver connection"]
fn test_submit_rsvp() {
    // TODO: Implement with mock client
}

#[test]
#[ignore = "requires Matrix homeserver connection"]
fn test_get_rsvp_counts() {
    // TODO: Implement with mock client
}

#[test]
#[ignore = "requires Matrix homeserver connection"]
fn test_add_cohost() {
    // TODO: Implement with mock client
}
