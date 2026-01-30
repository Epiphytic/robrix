//! Profile page widget displaying user's social profile.
//!
//! This widget renders the full social profile page, including cover photo,
//! avatar, user information, and action buttons for social interactions.

use makepad_widgets::*;
use matrix_sdk::ruma::OwnedUserId;
use robrix_social_events::profile::SocialProfileEventContent;
use std::sync::Arc;

use crate::shared::avatar::AvatarWidgetExt;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::avatar::Avatar;

    // Default cover image placeholder
    IMG_DEFAULT_COVER = dep("crate://self/resources/img/default_avatar.png")

    /// Social profile page layout displaying user's extended profile information.
    pub SocialProfilePage = {{SocialProfilePage}} {
        width: Fill,
        height: Fill,
        flow: Down,
        show_bg: true,
        draw_bg: {
            color: #fff
        }

        // Cover photo banner
        cover_container = <View> {
            width: Fill,
            height: 200,

            cover_image = <Image> {
                width: Fill,
                height: Fill,
                fit: Cover,
                source: (IMG_DEFAULT_COVER),
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        // Gradient overlay for better text visibility
                        let color = self.get_color();
                        let gradient = mix(vec4(0., 0., 0., 0.3), vec4(0., 0., 0., 0.), self.pos.y);
                        return mix(color, gradient, 0.5);
                    }
                }
            }
        }

        // Profile info section
        profile_section = <View> {
            width: Fill,
            padding: 16,
            flow: Down,
            spacing: 12,

            // Avatar row - overlapping the cover photo
            avatar_row = <View> {
                width: Fill,
                height: Fit,
                margin: { top: -50 },
                flow: Right,
                align: { x: 0.0, y: 0.0 },

                avatar = <Avatar> {
                    width: 100,
                    height: 100,
                }

                <View> { width: Fill }

                // Edit button (for own profile)
                edit_button = <Button> {
                    width: Fit,
                    height: Fit,
                    margin: { top: 54 },
                    text: "Edit Profile",
                    draw_bg: {
                        color: #fff,
                        border_width: 1.0,
                        border_color: #ccc,
                        radius: 4.0,
                    }
                    draw_text: {
                        color: #333,
                    }
                }
            }

            // Name and username section
            name_section = <View> {
                width: Fill,
                height: Fit,
                flow: Down,
                spacing: 4,

                name_label = <Label> {
                    width: Fill,
                    height: Fit,
                    text: "",
                    draw_text: {
                        text_style: { font_size: 20.0, },
                        color: #000,
                        font_scale: 1.0,
                    }
                }

                username_label = <Label> {
                    width: Fill,
                    height: Fit,
                    text: "",
                    draw_text: {
                        text_style: { font_size: 14.0, },
                        color: #666,
                    }
                }
            }

            // Bio section
            bio_label = <Label> {
                width: Fill,
                height: Fit,
                text: "",
                draw_text: {
                    text_style: { font_size: 14.0, },
                    color: #333,
                    wrap: Word,
                }
            }

            // Location and website metadata row
            meta_row = <View> {
                width: Fill,
                height: Fit,
                flow: Right,
                spacing: 16,

                location_row = <View> {
                    width: Fit,
                    height: Fit,
                    flow: Right,
                    spacing: 4,
                    visible: false,

                    location_icon = <Label> {
                        width: Fit,
                        height: Fit,
                        text: "Location:",
                        draw_text: {
                            text_style: { font_size: 12.0 },
                            color: #666,
                        }
                    }

                    location_label = <Label> {
                        width: Fit,
                        height: Fit,
                        text: "",
                        draw_text: {
                            text_style: { font_size: 12.0 },
                            color: #666,
                        }
                    }
                }

                website_row = <View> {
                    width: Fit,
                    height: Fit,
                    flow: Right,
                    spacing: 4,
                    visible: false,

                    website_icon = <Label> {
                        width: Fit,
                        height: Fit,
                        text: "Website:",
                        draw_text: {
                            text_style: { font_size: 12.0 },
                            color: #666,
                        }
                    }

                    website_label = <Label> {
                        width: Fit,
                        height: Fit,
                        text: "",
                        draw_text: {
                            text_style: { font_size: 12.0 },
                            color: #1a0dab,
                        }
                    }
                }
            }

            // Action buttons row
            action_row = <View> {
                width: Fill,
                height: Fit,
                flow: Right,
                spacing: 8,
                margin: { top: 8 },

                follow_button = <Button> {
                    width: Fit,
                    height: Fit,
                    text: "Follow",
                    draw_bg: {
                        color: #1d9bf0,
                        radius: 20.0,
                    }
                    draw_text: {
                        color: #fff,
                    }
                }

                friend_request_button = <Button> {
                    width: Fit,
                    height: Fit,
                    text: "Add Friend",
                    draw_bg: {
                        color: #fff,
                        border_width: 1.0,
                        border_color: #ccc,
                        radius: 20.0,
                    }
                    draw_text: {
                        color: #333,
                    }
                }

                message_button = <Button> {
                    width: Fit,
                    height: Fit,
                    text: "Message",
                    draw_bg: {
                        color: #fff,
                        border_width: 1.0,
                        border_color: #ccc,
                        radius: 20.0,
                    }
                    draw_text: {
                        color: #333,
                    }
                }
            }
        }

        // Divider
        <View> {
            width: Fill,
            height: 1,
            show_bg: true,
            draw_bg: {
                color: #eee
            }
        }

        // Posts/Activity tabs section
        tabs_row = <View> {
            width: Fill,
            height: 48,
            flow: Right,
            padding: { left: 16, right: 16 },

            posts_tab = <Button> {
                width: Fit,
                height: Fill,
                text: "Posts",
                draw_bg: {
                    color: #0000
                }
                draw_text: {
                    color: #1d9bf0,
                }
            }

            media_tab = <Button> {
                width: Fit,
                height: Fill,
                text: "Media",
                draw_bg: {
                    color: #0000
                }
                draw_text: {
                    color: #666,
                }
            }

            likes_tab = <Button> {
                width: Fit,
                height: Fill,
                text: "Likes",
                draw_bg: {
                    color: #0000
                }
                draw_text: {
                    color: #666,
                }
            }
        }

        // Posts feed section (placeholder)
        posts_section = <View> {
            width: Fill,
            height: Fill,

            posts_placeholder = <Label> {
                width: Fill,
                height: Fill,
                padding: 32,
                text: "Posts will appear here...",
                draw_text: {
                    text_style: { font_size: 14.0 },
                    color: #999,
                    wrap: Word,
                }
            }

            // Will embed SocialFeedView in Phase 3
        }
    }
}

/// Loaded profile data for display.
#[derive(Clone, Debug)]
pub struct LoadedProfile {
    /// User ID of the profile owner
    pub user_id: OwnedUserId,
    /// Display name from Matrix profile
    pub display_name: Option<String>,
    /// Extended social profile data
    pub social_profile: Option<SocialProfileEventContent>,
    /// Avatar image data
    pub avatar_data: Option<Arc<[u8]>>,
    /// Cover image data
    pub cover_data: Option<Arc<[u8]>>,
}

/// Actions that can be triggered from the profile page.
#[derive(Clone, Debug, DefaultNone)]
pub enum SocialProfileAction {
    /// User clicked the Follow button
    Follow(OwnedUserId),
    /// User clicked the Add Friend button
    SendFriendRequest(OwnedUserId),
    /// User clicked the Message button
    OpenDirectMessage(OwnedUserId),
    /// User clicked the Edit Profile button
    EditProfile,
    /// User clicked on the website link
    OpenWebsite(String),
    /// No action
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct SocialProfilePage {
    #[deref]
    view: View,

    /// The user ID of the profile being displayed.
    #[rust]
    user_id: Option<OwnedUserId>,

    /// The loaded profile data.
    #[rust]
    profile: Option<LoadedProfile>,

    /// Whether this is the current user's own profile.
    #[rust]
    is_own_profile: bool,
}

impl Widget for SocialProfilePage {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for SocialProfilePage {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let follow_button = self.button(ids!(follow_button));
        let friend_request_button = self.button(ids!(friend_request_button));
        let message_button = self.button(ids!(message_button));
        let edit_button = self.button(ids!(edit_button));

        // Handle button clicks
        if follow_button.clicked(actions) {
            if let Some(user_id) = &self.user_id {
                cx.action(SocialProfileAction::Follow(user_id.clone()));
            }
        }

        if friend_request_button.clicked(actions) {
            if let Some(user_id) = &self.user_id {
                cx.action(SocialProfileAction::SendFriendRequest(user_id.clone()));
            }
        }

        if message_button.clicked(actions) {
            if let Some(user_id) = &self.user_id {
                cx.action(SocialProfileAction::OpenDirectMessage(user_id.clone()));
            }
        }

        if edit_button.clicked(actions) {
            cx.action(SocialProfileAction::EditProfile);
        }
    }
}

impl SocialProfilePage {
    /// Set the user ID for this profile page.
    pub fn set_user_id(&mut self, user_id: OwnedUserId, is_own_profile: bool) {
        self.user_id = Some(user_id);
        self.is_own_profile = is_own_profile;
    }

    /// Set the loaded profile data and update the UI.
    pub fn set_profile(&mut self, cx: &mut Cx, profile: LoadedProfile) {
        // Update name label
        let name = profile
            .display_name
            .clone()
            .unwrap_or_else(|| profile.user_id.localpart().to_string());
        self.label(ids!(name_label)).set_text(cx, &name);

        // Update username label
        self.label(ids!(username_label))
            .set_text(cx, &profile.user_id.to_string());

        // Update bio if available
        if let Some(ref social) = profile.social_profile {
            if let Some(ref bio) = social.bio {
                self.label(ids!(bio_label)).set_text(cx, bio);
            }

            // Update location if available
            if let Some(ref location) = social.location {
                self.label(ids!(location_label)).set_text(cx, location);
                self.view(ids!(location_row)).set_visible(cx, true);
            } else {
                self.view(ids!(location_row)).set_visible(cx, false);
            }

            // Update website if available
            if let Some(ref website) = social.website {
                self.label(ids!(website_label))
                    .set_text(cx, website.as_str());
                self.view(ids!(website_row)).set_visible(cx, true);
            } else {
                self.view(ids!(website_row)).set_visible(cx, false);
            }
        }

        // Update avatar with first letter of name
        self.avatar(ids!(avatar)).set_text(cx, &name);

        // Show/hide action buttons based on whether this is own profile
        self.view(ids!(action_row))
            .set_visible(cx, !self.is_own_profile);
        self.button(ids!(edit_button))
            .set_visible(cx, self.is_own_profile);

        self.profile = Some(profile);
    }

    /// Clear the profile data.
    pub fn clear(&mut self, cx: &mut Cx) {
        self.user_id = None;
        self.profile = None;
        self.is_own_profile = false;

        self.label(ids!(name_label)).set_text(cx, "");
        self.label(ids!(username_label)).set_text(cx, "");
        self.label(ids!(bio_label)).set_text(cx, "");
        self.view(ids!(location_row)).set_visible(cx, false);
        self.view(ids!(website_row)).set_visible(cx, false);
    }
}

impl SocialProfilePageRef {
    /// See [`SocialProfilePage::set_user_id()`].
    pub fn set_user_id(&self, user_id: OwnedUserId, is_own_profile: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_user_id(user_id, is_own_profile);
        }
    }

    /// See [`SocialProfilePage::set_profile()`].
    pub fn set_profile(&self, cx: &mut Cx, profile: LoadedProfile) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_profile(cx, profile);
        }
    }

    /// See [`SocialProfilePage::clear()`].
    pub fn clear(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.clear(cx);
        }
    }
}
