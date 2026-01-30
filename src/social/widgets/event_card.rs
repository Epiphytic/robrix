//! Event card widget for displaying event information and RSVP actions.
//!
//! This widget renders an event card with event details, time/location info,
//! RSVP counts, and action buttons for responding to events.

use makepad_widgets::*;
use matrix_sdk::ruma::OwnedRoomId;
use robrix_social_events::event::{EventLocation, SocialEventEventContent};
use robrix_social_events::rsvp::RsvpStatus;

use crate::social::events::RsvpCounts;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;

    // Default event cover image placeholder
    IMG_DEFAULT_EVENT_COVER = dep("crate://self/resources/img/default_avatar.png")

    /// Event card widget for displaying event information.
    pub EventCard = {{EventCard}} {
        width: Fill,
        height: Fit,
        flow: Down,
        padding: 0,
        margin: { bottom: 12 },
        show_bg: true,
        draw_bg: {
            color: #fff,
            radius: 8.0,
        }

        // Cover image section
        cover_container = <View> {
            width: Fill,
            height: 150,

            cover_image = <Image> {
                width: Fill,
                height: Fill,
                fit: Cover,
                source: (IMG_DEFAULT_EVENT_COVER),
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let color = self.get_color();
                        // Slight gradient overlay at bottom for text readability
                        let gradient = mix(vec4(0., 0., 0., 0.4), vec4(0., 0., 0., 0.), 1.0 - self.pos.y);
                        return mix(color, gradient, 0.3);
                    }
                }
            }
        }

        // Event details section
        details_section = <View> {
            width: Fill,
            height: Fit,
            flow: Down,
            padding: 16,
            spacing: 8,

            // Title row
            title_label = <Label> {
                width: Fill,
                height: Fit,
                text: "Event Title",
                draw_text: {
                    text_style: { font_size: 18.0 },
                    color: #000,
                    font_scale: 1.0,
                }
            }

            // Date and time row
            datetime_row = <View> {
                width: Fill,
                height: Fit,
                flow: Right,
                spacing: 8,

                date_icon = <Label> {
                    width: Fit,
                    height: Fit,
                    text: "📅",
                    draw_text: {
                        text_style: { font_size: 14.0 },
                        color: #666,
                    }
                }

                datetime_label = <Label> {
                    width: Fill,
                    height: Fit,
                    text: "",
                    draw_text: {
                        text_style: { font_size: 14.0 },
                        color: #666,
                    }
                }
            }

            // Location row
            location_row = <View> {
                width: Fill,
                height: Fit,
                flow: Right,
                spacing: 8,
                visible: false,

                location_icon = <Label> {
                    width: Fit,
                    height: Fit,
                    text: "📍",
                    draw_text: {
                        text_style: { font_size: 14.0 },
                        color: #666,
                    }
                }

                location_label = <Label> {
                    width: Fill,
                    height: Fit,
                    text: "",
                    draw_text: {
                        text_style: { font_size: 14.0 },
                        color: #666,
                    }
                }
            }

            // Description
            description_label = <Label> {
                width: Fill,
                height: Fit,
                text: "",
                draw_text: {
                    text_style: { font_size: 14.0 },
                    color: #333,
                    wrap: Word,
                }
            }

            // RSVP counts row
            rsvp_counts_row = <View> {
                width: Fill,
                height: Fit,
                flow: Right,
                spacing: 16,
                padding: { top: 8 },

                going_count = <Label> {
                    width: Fit,
                    height: Fit,
                    text: "0 Going",
                    draw_text: {
                        text_style: { font_size: 12.0 },
                        color: #22c55e,
                    }
                }

                interested_count = <Label> {
                    width: Fit,
                    height: Fit,
                    text: "0 Interested",
                    draw_text: {
                        text_style: { font_size: 12.0 },
                        color: #f59e0b,
                    }
                }
            }

            // Divider
            <View> {
                width: Fill,
                height: 1,
                margin: { top: 8 },
                show_bg: true,
                draw_bg: {
                    color: #eee
                }
            }

            // RSVP buttons row
            rsvp_buttons_row = <View> {
                width: Fill,
                height: Fit,
                flow: Right,
                spacing: 8,
                padding: { top: 8 },

                going_button = <Button> {
                    width: Fit,
                    height: Fit,
                    text: "Going",
                    draw_bg: {
                        color: #22c55e,
                        radius: 4.0,
                    }
                    draw_text: {
                        color: #fff,
                    }
                }

                interested_button = <Button> {
                    width: Fit,
                    height: Fit,
                    text: "Interested",
                    draw_bg: {
                        color: #fff,
                        border_width: 1.0,
                        border_color: #f59e0b,
                        radius: 4.0,
                    }
                    draw_text: {
                        color: #f59e0b,
                    }
                }

                not_going_button = <Button> {
                    width: Fit,
                    height: Fit,
                    text: "Not Going",
                    draw_bg: {
                        color: #fff,
                        border_width: 1.0,
                        border_color: #ccc,
                        radius: 4.0,
                    }
                    draw_text: {
                        color: #666,
                    }
                }
            }
        }
    }
}

/// Loaded event data for display.
#[derive(Clone, Debug)]
pub struct LoadedEvent {
    /// Room ID of the event.
    pub room_id: OwnedRoomId,
    /// Event content from state.
    pub event: SocialEventEventContent,
    /// RSVP counts.
    pub rsvp_counts: RsvpCounts,
    /// Current user's RSVP status.
    pub user_rsvp: Option<RsvpStatus>,
    /// Cover image data.
    pub cover_data: Option<std::sync::Arc<[u8]>>,
}

/// Actions that can be triggered from the event card.
#[derive(Clone, Debug, DefaultNone)]
pub enum EventCardAction {
    /// User clicked Going button.
    RsvpGoing(OwnedRoomId),
    /// User clicked Interested button.
    RsvpInterested(OwnedRoomId),
    /// User clicked Not Going button.
    RsvpNotGoing(OwnedRoomId),
    /// User clicked to view event details.
    ViewEvent(OwnedRoomId),
    /// User clicked location to view map.
    ViewLocation(EventLocation),
    /// No action.
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct EventCard {
    #[deref]
    view: View,

    /// The room ID of the event being displayed.
    #[rust]
    room_id: Option<OwnedRoomId>,

    /// The loaded event data.
    #[rust]
    event: Option<LoadedEvent>,
}

impl Widget for EventCard {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for EventCard {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let going_button = self.button(ids!(going_button));
        let interested_button = self.button(ids!(interested_button));
        let not_going_button = self.button(ids!(not_going_button));

        if let Some(room_id) = &self.room_id {
            if going_button.clicked(actions) {
                cx.action(EventCardAction::RsvpGoing(room_id.clone()));
            }

            if interested_button.clicked(actions) {
                cx.action(EventCardAction::RsvpInterested(room_id.clone()));
            }

            if not_going_button.clicked(actions) {
                cx.action(EventCardAction::RsvpNotGoing(room_id.clone()));
            }
        }
    }
}

impl EventCard {
    /// Set the event data and update the UI.
    pub fn set_event(&mut self, cx: &mut Cx, event: LoadedEvent) {
        self.room_id = Some(event.room_id.clone());

        // Update title
        self.label(ids!(title_label))
            .set_text(cx, &event.event.title);

        // Update datetime
        let datetime_str = format_event_time(event.event.start_time, event.event.end_time);
        self.label(ids!(datetime_label)).set_text(cx, &datetime_str);

        // Update location if available
        if let Some(ref location) = event.event.location {
            self.label(ids!(location_label))
                .set_text(cx, &location.name);
            self.view(ids!(location_row)).set_visible(cx, true);
        } else {
            self.view(ids!(location_row)).set_visible(cx, false);
        }

        // Update description
        if let Some(ref desc) = event.event.description {
            self.label(ids!(description_label)).set_text(cx, desc);
        } else {
            self.label(ids!(description_label)).set_text(cx, "");
        }

        // Update RSVP counts
        self.label(ids!(going_count))
            .set_text(cx, &format!("{} Going", event.rsvp_counts.going));
        self.label(ids!(interested_count))
            .set_text(cx, &format!("{} Interested", event.rsvp_counts.interested));

        // Highlight user's current RSVP (visual feedback)
        // This could be expanded to change button styles based on current RSVP

        self.event = Some(event);
    }

    /// Clear the event data.
    pub fn clear(&mut self, cx: &mut Cx) {
        self.room_id = None;
        self.event = None;

        self.label(ids!(title_label)).set_text(cx, "");
        self.label(ids!(datetime_label)).set_text(cx, "");
        self.label(ids!(description_label)).set_text(cx, "");
        self.view(ids!(location_row)).set_visible(cx, false);
        self.label(ids!(going_count)).set_text(cx, "0 Going");
        self.label(ids!(interested_count))
            .set_text(cx, "0 Interested");
    }
}

impl EventCardRef {
    /// See [`EventCard::set_event()`].
    pub fn set_event(&self, cx: &mut Cx, event: LoadedEvent) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_event(cx, event);
        }
    }

    /// See [`EventCard::clear()`].
    pub fn clear(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.clear(cx);
        }
    }
}

/// Format event time for display.
fn format_event_time(start_ms: u64, end_ms: Option<u64>) -> String {
    use chrono::{DateTime, Utc};

    let start = DateTime::from_timestamp_millis(start_ms as i64).unwrap_or_else(|| Utc::now());

    let start_str = start.format("%a, %b %d at %I:%M %p").to_string();

    if let Some(end) = end_ms {
        let end_dt = DateTime::from_timestamp_millis(end as i64).unwrap_or_else(|| Utc::now());

        // If same day, just show end time
        if start.date_naive() == end_dt.date_naive() {
            format!("{} - {}", start_str, end_dt.format("%I:%M %p"))
        } else {
            format!("{} - {}", start_str, end_dt.format("%a, %b %d at %I:%M %p"))
        }
    } else {
        start_str
    }
}
