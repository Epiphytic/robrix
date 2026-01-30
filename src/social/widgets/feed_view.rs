//! Feed view widget displaying a scrollable list of posts.
//!
//! This widget renders an aggregated feed of posts from multiple
//! feed rooms, supporting infinite scroll and refresh.

use makepad_widgets::*;
use matrix_sdk::ruma::OwnedEventId;

use crate::social::widgets::post_card::{PostCardData, SocialPostCardAction};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::social::widgets::post_card::SocialPostCard;
    use crate::social::widgets::post_composer::SocialPostComposer;

    FEED_BG_COLOR = #f0f2f5
    SPINNER_COLOR = #1d9bf0

    /// Feed view widget displaying a scrollable list of posts.
    pub SocialFeedView = {{SocialFeedView}} {
        width: Fill,
        height: Fill,
        flow: Down,
        show_bg: true,
        draw_bg: {
            color: (FEED_BG_COLOR)
        }

        // Composer at top (optional, can be hidden)
        composer_section = <View> {
            width: Fill,
            height: Fit,
            visible: true,

            composer = <SocialPostComposer> {}
        }

        // Feed content
        feed_scroll = <PortalList> {
            width: Fill,
            height: Fill,
            flow: Down,

            // Template for post items
            post_item = <SocialPostCard> {
                margin: { bottom: 8 }
            }

            // Loading indicator at bottom
            loading_item = <View> {
                width: Fill,
                height: 60,
                align: { x: 0.5, y: 0.5 },

                loading_spinner = <View> {
                    width: 32,
                    height: 32,
                    show_bg: true,
                    draw_bg: {
                        color: (SPINNER_COLOR),
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            let center = self.rect_size / 2.;
                            let radius = min(center.x, center.y) - 2.;
                            sdf.circle(center.x, center.y, radius);
                            sdf.stroke(self.color, 3.);
                            return sdf.result;
                        }
                    }
                }
            }

            // Empty state
            empty_state = <View> {
                width: Fill,
                height: Fill,
                align: { x: 0.5, y: 0.5 },
                padding: 32,
                flow: Down,
                spacing: 16,

                empty_icon = <Label> {
                    width: Fit,
                    height: Fit,
                    text: "📭",
                    draw_text: {
                        text_style: { font_size: 48.0 },
                        color: #999,
                    }
                }

                empty_title = <Label> {
                    width: Fit,
                    height: Fit,
                    text: "No posts yet",
                    draw_text: {
                        text_style: { font_size: 18.0 },
                        color: #333,
                    }
                }

                empty_subtitle = <Label> {
                    width: Fit,
                    height: Fit,
                    text: "Follow some people to see their posts here",
                    draw_text: {
                        text_style: { font_size: 14.0 },
                        color: #666,
                        wrap: Word,
                    }
                }
            }
        }

        // Pull-to-refresh indicator (for mobile)
        refresh_indicator = <View> {
            width: Fill,
            height: 0,
            visible: false,
            align: { x: 0.5, y: 0.5 },

            refresh_spinner = <Label> {
                width: Fit,
                height: Fit,
                text: "⟳",
                draw_text: {
                    text_style: { font_size: 24.0 },
                    color: (SPINNER_COLOR),
                }
            }
        }
    }
}

/// Current state of the feed view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FeedState {
    /// Initial state, no data loaded.
    #[default]
    Empty,
    /// Loading initial data.
    Loading,
    /// Data loaded and displaying.
    Loaded,
    /// Loading more data at the end.
    LoadingMore,
    /// Refreshing data.
    Refreshing,
    /// Error occurred.
    Error,
}

/// Actions that can be triggered from the feed view.
#[derive(Clone, Debug, DefaultNone)]
pub enum SocialFeedViewAction {
    /// User wants to refresh the feed.
    Refresh,
    /// User scrolled to the bottom, load more posts.
    LoadMore,
    /// User interacted with a post (delegated from PostCard).
    PostAction(SocialPostCardAction),
    /// No action.
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct SocialFeedView {
    #[deref]
    view: View,

    /// Current posts to display.
    #[rust]
    posts: Vec<PostCardData>,

    /// Current state of the feed.
    #[rust]
    state: FeedState,

    /// Whether the composer should be shown.
    #[rust]
    show_composer: bool,
}

impl Widget for SocialFeedView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        // Handle scroll events for infinite scroll
        if let Event::Scroll(scroll) = event {
            self.handle_scroll(cx, scroll);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Update visibility of sections based on state
        self.update_visibility(cx);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for SocialFeedView {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Forward post card actions
        for action in actions {
            if let Some(post_action) = action.downcast_ref::<SocialPostCardAction>() {
                match post_action {
                    SocialPostCardAction::None => {}
                    _ => {
                        cx.action(SocialFeedViewAction::PostAction(post_action.clone()));
                    }
                }
            }
        }
    }
}

impl SocialFeedView {
    /// Set the posts to display in the feed.
    pub fn set_posts(&mut self, cx: &mut Cx, posts: Vec<PostCardData>) {
        self.posts = posts;
        self.state = if self.posts.is_empty() {
            FeedState::Empty
        } else {
            FeedState::Loaded
        };
        self.redraw(cx);
    }

    /// Append more posts to the feed.
    pub fn append_posts(&mut self, cx: &mut Cx, posts: Vec<PostCardData>) {
        self.posts.extend(posts);
        self.state = FeedState::Loaded;
        self.redraw(cx);
    }

    /// Prepend new posts to the feed (for refresh).
    pub fn prepend_posts(&mut self, cx: &mut Cx, posts: Vec<PostCardData>) {
        let mut new_posts = posts;
        new_posts.append(&mut self.posts);
        self.posts = new_posts;
        self.state = FeedState::Loaded;
        self.redraw(cx);
    }

    /// Set the feed state.
    pub fn set_state(&mut self, cx: &mut Cx, state: FeedState) {
        self.state = state;
        self.redraw(cx);
    }

    /// Show or hide the composer.
    pub fn set_show_composer(&mut self, cx: &mut Cx, show: bool) {
        self.show_composer = show;
        self.view(ids!(composer_section)).set_visible(cx, show);
    }

    /// Clear all posts.
    pub fn clear(&mut self, cx: &mut Cx) {
        self.posts.clear();
        self.state = FeedState::Empty;
        self.redraw(cx);
    }

    /// Get the number of posts.
    pub fn post_count(&self) -> usize {
        self.posts.len()
    }

    /// Get a reference to the posts.
    pub fn posts(&self) -> &[PostCardData] {
        &self.posts
    }

    /// Find a post by event ID.
    pub fn find_post(&self, event_id: &OwnedEventId) -> Option<&PostCardData> {
        self.posts.iter().find(|p| &p.event_id == event_id)
    }

    /// Update a post by event ID.
    pub fn update_post(&mut self, cx: &mut Cx, event_id: &OwnedEventId, data: PostCardData) {
        if let Some(post) = self.posts.iter_mut().find(|p| &p.event_id == event_id) {
            *post = data;
            self.redraw(cx);
        }
    }

    /// Remove a post by event ID.
    pub fn remove_post(&mut self, cx: &mut Cx, event_id: &OwnedEventId) {
        self.posts.retain(|p| &p.event_id != event_id);
        if self.posts.is_empty() {
            self.state = FeedState::Empty;
        }
        self.redraw(cx);
    }

    /// Handle scroll events for infinite scrolling.
    fn handle_scroll(&mut self, cx: &mut Cx, _scroll: &event::ScrollEvent) {
        // Check if we're near the bottom and should load more
        // This is a simplified check - real implementation would use
        // the scroll position and content height
        if self.state == FeedState::Loaded && self.posts.len() >= 10 {
            // Trigger load more when near bottom
            // (actual scroll position check would go here)
            // cx.action(SocialFeedViewAction::LoadMore);
        }
        let _ = cx;
    }

    /// Update visibility of UI elements based on current state.
    fn update_visibility(&mut self, cx: &mut Cx2d) {
        // Show/hide empty state
        let show_empty = self.state == FeedState::Empty;
        self.view(ids!(empty_state)).set_visible(cx, show_empty);

        // Show/hide loading indicator
        let show_loading = self.state == FeedState::Loading || self.state == FeedState::LoadingMore;
        self.view(ids!(loading_item)).set_visible(cx, show_loading);

        // Show/hide refresh indicator
        let show_refresh = self.state == FeedState::Refreshing;
        self.view(ids!(refresh_indicator))
            .set_visible(cx, show_refresh);
    }
}

impl SocialFeedViewRef {
    /// See [`SocialFeedView::set_posts()`].
    pub fn set_posts(&self, cx: &mut Cx, posts: Vec<PostCardData>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_posts(cx, posts);
        }
    }

    /// See [`SocialFeedView::append_posts()`].
    pub fn append_posts(&self, cx: &mut Cx, posts: Vec<PostCardData>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.append_posts(cx, posts);
        }
    }

    /// See [`SocialFeedView::prepend_posts()`].
    pub fn prepend_posts(&self, cx: &mut Cx, posts: Vec<PostCardData>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.prepend_posts(cx, posts);
        }
    }

    /// See [`SocialFeedView::set_state()`].
    pub fn set_state(&self, cx: &mut Cx, state: FeedState) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_state(cx, state);
        }
    }

    /// See [`SocialFeedView::set_show_composer()`].
    pub fn set_show_composer(&self, cx: &mut Cx, show: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_show_composer(cx, show);
        }
    }

    /// See [`SocialFeedView::clear()`].
    pub fn clear(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.clear(cx);
        }
    }

    /// See [`SocialFeedView::post_count()`].
    pub fn post_count(&self) -> usize {
        self.borrow().map(|inner| inner.post_count()).unwrap_or(0)
    }
}
