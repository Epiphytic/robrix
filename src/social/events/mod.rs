//! Event gatherings module for Robrix social features.
//!
//! This module provides services for creating and managing event rooms,
//! handling RSVPs, and coordinating event-related functionality.

pub mod event_room;
pub mod rsvp;

pub use event_room::{EventRole, EventRoomError, EventRoomService, event_room_power_levels};
pub use rsvp::{RsvpCounts, RsvpError, RsvpService, RsvpValidation, ValidatedRsvp, validate_rsvp_event};
