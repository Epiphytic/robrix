//! Placeholder module when social features are disabled.

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    // Empty placeholder widgets that render nothing
    pub SocialFeedView = {{SocialFeedView}} {}
    pub SocialProfilePage = {{SocialProfilePage}} {}
    pub SocialPostComposer = {{SocialPostComposer}} {}
    pub SocialEventCard = {{SocialEventCard}} {}
    pub SocialFriendList = {{SocialFriendList}} {}
}

#[derive(Live, LiveHook, Widget)]
pub struct SocialFeedView {
    #[deref]
    view: View,
}

impl Widget for SocialFeedView {
    fn draw_walk(&mut self, _cx: &mut Cx2d, _scope: &mut Scope, _walk: Walk) -> DrawStep {
        DrawStep::done()
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SocialProfilePage {
    #[deref]
    view: View,
}

impl Widget for SocialProfilePage {
    fn draw_walk(&mut self, _cx: &mut Cx2d, _scope: &mut Scope, _walk: Walk) -> DrawStep {
        DrawStep::done()
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SocialPostComposer {
    #[deref]
    view: View,
}

impl Widget for SocialPostComposer {
    fn draw_walk(&mut self, _cx: &mut Cx2d, _scope: &mut Scope, _walk: Walk) -> DrawStep {
        DrawStep::done()
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SocialEventCard {
    #[deref]
    view: View,
}

impl Widget for SocialEventCard {
    fn draw_walk(&mut self, _cx: &mut Cx2d, _scope: &mut Scope, _walk: Walk) -> DrawStep {
        DrawStep::done()
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SocialFriendList {
    #[deref]
    view: View,
}

impl Widget for SocialFriendList {
    fn draw_walk(&mut self, _cx: &mut Cx2d, _scope: &mut Scope, _walk: Walk) -> DrawStep {
        DrawStep::done()
    }
}

/// Register placeholder widgets (no-op when social features disabled).
pub fn live_design(_cx: &mut Cx) {
    // No-op: social features are disabled
}
