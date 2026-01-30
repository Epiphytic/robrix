//! Friend list widget for displaying and managing friends.
//!
//! This widget renders the user's friend list with options to view profiles,
//! send messages, and remove friends. It also displays pending friend requests.

use makepad_widgets::*;
use matrix_sdk::ruma::OwnedUserId;
use std::sync::Arc;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::avatar::Avatar;

    /// Individual friend item in the list.
    FriendItem = <View> {
        width: Fill,
        height: Fit,
        padding: { left: 16, right: 16, top: 12, bottom: 12 },
        flow: Right,
        spacing: 12,
        show_bg: true,
        draw_bg: {
            color: #fff
        }

        // Friend's avatar
        avatar = <Avatar> {
            width: 48,
            height: 48,
        }

        // Friend info column
        info_column = <View> {
            width: Fill,
            height: Fit,
            flow: Down,
            spacing: 4,

            name_label = <Label> {
                width: Fill,
                height: Fit,
                text: "",
                draw_text: {
                    text_style: { font_size: 14.0 },
                    color: #000,
                }
            }

            username_label = <Label> {
                width: Fill,
                height: Fit,
                text: "",
                draw_text: {
                    text_style: { font_size: 12.0 },
                    color: #666,
                }
            }

            status_label = <Label> {
                width: Fill,
                height: Fit,
                text: "",
                draw_text: {
                    text_style: { font_size: 11.0 },
                    color: #999,
                }
            }
        }

        // Action buttons
        actions_row = <View> {
            width: Fit,
            height: Fit,
            flow: Right,
            spacing: 8,
            align: { x: 1.0, y: 0.5 },

            message_button = <Button> {
                width: 36,
                height: 36,
                text: "Msg",
                draw_bg: {
                    color: #f0f0f0,
                    radius: 18.0,
                }
                draw_text: {
                    color: #333,
                    text_style: { font_size: 10.0 },
                }
            }

            remove_button = <Button> {
                width: 36,
                height: 36,
                text: "X",
                draw_bg: {
                    color: #fff0f0,
                    radius: 18.0,
                }
                draw_text: {
                    color: #c00,
                    text_style: { font_size: 12.0 },
                }
            }
        }
    }

    /// Pending friend request item.
    FriendRequestItem = <View> {
        width: Fill,
        height: Fit,
        padding: { left: 16, right: 16, top: 12, bottom: 12 },
        flow: Right,
        spacing: 12,
        show_bg: true,
        draw_bg: {
            color: #fffef0
        }

        // Requester's avatar
        avatar = <Avatar> {
            width: 48,
            height: 48,
        }

        // Request info column
        info_column = <View> {
            width: Fill,
            height: Fit,
            flow: Down,
            spacing: 4,

            name_label = <Label> {
                width: Fill,
                height: Fit,
                text: "",
                draw_text: {
                    text_style: { font_size: 14.0 },
                    color: #000,
                }
            }

            username_label = <Label> {
                width: Fill,
                height: Fit,
                text: "",
                draw_text: {
                    text_style: { font_size: 12.0 },
                    color: #666,
                }
            }

            request_label = <Label> {
                width: Fill,
                height: Fit,
                text: "Wants to be your friend",
                draw_text: {
                    text_style: { font_size: 11.0 },
                    color: #996600,
                }
            }
        }

        // Accept/Decline buttons
        actions_row = <View> {
            width: Fit,
            height: Fit,
            flow: Right,
            spacing: 8,
            align: { x: 1.0, y: 0.5 },

            accept_button = <Button> {
                width: Fit,
                height: 32,
                text: "Accept",
                draw_bg: {
                    color: #1d9bf0,
                    radius: 16.0,
                }
                draw_text: {
                    color: #fff,
                    text_style: { font_size: 12.0 },
                }
            }

            decline_button = <Button> {
                width: Fit,
                height: 32,
                text: "Decline",
                draw_bg: {
                    color: #f0f0f0,
                    radius: 16.0,
                }
                draw_text: {
                    color: #666,
                    text_style: { font_size: 12.0 },
                }
            }
        }
    }

    /// Section header for friend list sections.
    FriendListSection = <View> {
        width: Fill,
        height: Fit,
        padding: { left: 16, right: 16, top: 16, bottom: 8 },
        show_bg: true,
        draw_bg: {
            color: #f8f8f8
        }

        section_label = <Label> {
            width: Fill,
            height: Fit,
            text: "",
            draw_text: {
                text_style: { font_size: 12.0 },
                color: #666,
            }
        }
    }

    /// Friend list view displaying all friends and pending requests.
    pub FriendListView = {{FriendListView}} {
        width: Fill,
        height: Fill,
        flow: Down,
        show_bg: true,
        draw_bg: {
            color: #fff
        }

        // Header
        header = <View> {
            width: Fill,
            height: Fit,
            padding: 16,
            flow: Right,
            show_bg: true,
            draw_bg: {
                color: #fff
            }

            title_label = <Label> {
                width: Fill,
                height: Fit,
                text: "Friends",
                draw_text: {
                    text_style: { font_size: 20.0 },
                    color: #000,
                }
            }

            add_friend_button = <Button> {
                width: Fit,
                height: Fit,
                text: "Add Friend",
                draw_bg: {
                    color: #1d9bf0,
                    radius: 16.0,
                }
                draw_text: {
                    color: #fff,
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

        // Scrollable content
        content = <View> {
            width: Fill,
            height: Fill,
            flow: Down,

            // Pending requests section (shown when there are requests)
            requests_section = <View> {
                width: Fill,
                height: Fit,
                flow: Down,
                visible: false,

                requests_header = <FriendListSection> {
                    section_label = {
                        text: "Friend Requests"
                    }
                }

                requests_list = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Down,
                }
            }

            // Friends section
            friends_section = <View> {
                width: Fill,
                height: Fit,
                flow: Down,

                friends_header = <FriendListSection> {
                    section_label = {
                        text: "All Friends"
                    }
                }

                friends_list = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Down,
                }
            }

            // Empty state
            empty_state = <View> {
                width: Fill,
                height: 200,
                align: { x: 0.5, y: 0.5 },
                visible: true,

                empty_label = <Label> {
                    width: Fit,
                    height: Fit,
                    text: "No friends yet. Add some friends to get started!",
                    draw_text: {
                        text_style: { font_size: 14.0 },
                        color: #999,
                    }
                }
            }
        }
    }
}

/// Information about a friend for display.
#[derive(Clone, Debug)]
pub struct FriendInfo {
    /// The friend's user ID
    pub user_id: OwnedUserId,
    /// Display name
    pub display_name: Option<String>,
    /// Online status (if available)
    pub status: Option<String>,
    /// Avatar image data
    pub avatar_data: Option<Arc<[u8]>>,
}

/// Information about a pending friend request.
#[derive(Clone, Debug)]
pub struct FriendRequestInfo {
    /// The requester's user ID
    pub user_id: OwnedUserId,
    /// Display name
    pub display_name: Option<String>,
    /// Avatar image data
    pub avatar_data: Option<Arc<[u8]>>,
    /// Request message (if any)
    pub message: Option<String>,
}

/// Actions that can be triggered from the friend list.
#[derive(Clone, Debug, DefaultNone)]
pub enum FriendListAction {
    /// User clicked Add Friend button
    AddFriend,
    /// User clicked to view a friend's profile
    ViewProfile(OwnedUserId),
    /// User clicked to message a friend
    MessageFriend(OwnedUserId),
    /// User clicked to remove a friend
    RemoveFriend(OwnedUserId),
    /// User accepted a friend request
    AcceptRequest(OwnedUserId),
    /// User declined a friend request
    DeclineRequest(OwnedUserId),
    /// No action
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct FriendListView {
    #[deref]
    view: View,

    /// List of friends to display.
    #[rust]
    friends: Vec<FriendInfo>,

    /// List of pending friend requests.
    #[rust]
    pending_requests: Vec<FriendRequestInfo>,
}

impl Widget for FriendListView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for FriendListView {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Handle Add Friend button
        let add_friend_button = self.button(ids!(add_friend_button));
        if add_friend_button.clicked(actions) {
            cx.action(FriendListAction::AddFriend);
        }

        // Individual friend item actions would be handled here
        // when we implement the dynamic list rendering
    }
}

impl FriendListView {
    /// Set the list of friends to display.
    pub fn set_friends(&mut self, cx: &mut Cx, friends: Vec<FriendInfo>) {
        self.friends = friends;
        self.update_display(cx);
    }

    /// Set the list of pending friend requests.
    pub fn set_pending_requests(&mut self, cx: &mut Cx, requests: Vec<FriendRequestInfo>) {
        self.pending_requests = requests;
        self.update_display(cx);
    }

    /// Add a friend to the list.
    pub fn add_friend(&mut self, cx: &mut Cx, friend: FriendInfo) {
        self.friends.push(friend);
        self.update_display(cx);
    }

    /// Remove a friend from the list.
    pub fn remove_friend(&mut self, cx: &mut Cx, user_id: &OwnedUserId) {
        self.friends.retain(|f| &f.user_id != user_id);
        self.update_display(cx);
    }

    /// Clear the friend list.
    pub fn clear(&mut self, cx: &mut Cx) {
        self.friends.clear();
        self.pending_requests.clear();
        self.update_display(cx);
    }

    /// Update the display based on current data.
    fn update_display(&mut self, cx: &mut Cx) {
        let has_requests = !self.pending_requests.is_empty();
        let has_friends = !self.friends.is_empty();

        // Show/hide requests section
        self.view(ids!(requests_section))
            .set_visible(cx, has_requests);

        // Show/hide empty state
        self.view(ids!(empty_state))
            .set_visible(cx, !has_friends && !has_requests);

        // Note: Dynamic list item creation requires PortalList or similar
        // For now, we update the visibility based on data state
        // Full implementation would:
        // 1. Clear existing list items
        // 2. Create FriendItem widgets for each friend
        // 3. Create FriendRequestItem widgets for each pending request
        // 4. Update section headers with counts
        let _ = has_friends;
    }

    /// Get the number of friends.
    pub fn friend_count(&self) -> usize {
        self.friends.len()
    }

    /// Get the number of pending requests.
    pub fn pending_request_count(&self) -> usize {
        self.pending_requests.len()
    }
}

impl FriendListViewRef {
    /// See [`FriendListView::set_friends()`].
    pub fn set_friends(&self, cx: &mut Cx, friends: Vec<FriendInfo>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_friends(cx, friends);
        }
    }

    /// See [`FriendListView::set_pending_requests()`].
    pub fn set_pending_requests(&self, cx: &mut Cx, requests: Vec<FriendRequestInfo>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_pending_requests(cx, requests);
        }
    }

    /// See [`FriendListView::add_friend()`].
    pub fn add_friend(&self, cx: &mut Cx, friend: FriendInfo) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.add_friend(cx, friend);
        }
    }

    /// See [`FriendListView::remove_friend()`].
    pub fn remove_friend(&self, cx: &mut Cx, user_id: &OwnedUserId) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.remove_friend(cx, user_id);
        }
    }

    /// See [`FriendListView::clear()`].
    pub fn clear(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.clear(cx);
        }
    }
}
