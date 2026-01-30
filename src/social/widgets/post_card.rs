//! Post card widget for displaying social posts.
//!
//! This widget renders a single post in a feed, including author info,
//! content, media, reactions, and interaction buttons.

use makepad_widgets::*;
use matrix_sdk::ruma::{MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedUserId};

use crate::shared::avatar::AvatarWidgetExt;
use crate::social::reactions::{reactions_for_display, ReactionSummary};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::avatar::Avatar;
    use crate::shared::icon_button::RobrixIconButton;

    CARD_BG_COLOR = #fff
    CARD_BORDER_COLOR = #e0e0e0
    ICON_COLOR = #666
    ICON_HOVER_COLOR = #1d9bf0
    REACTION_SELECTED_BG = #e8f5fd
    REACTION_NORMAL_BG = #f0f2f5
    REACTION_SELECTED_BORDER = #1d9bf0
    REACTION_NORMAL_BORDER = #e0e0e0

    /// Button template for displaying a single reaction.
    ReactionButton = <Button> {
        width: Fit,
        height: Fit,
        padding: { top: 4, bottom: 4, left: 8, right: 8 },
        margin: { right: 4 },

        draw_bg: {
            instance reaction_bg_color: (REACTION_NORMAL_BG)
            instance reaction_border_color: (REACTION_NORMAL_BORDER)
            border_radius: 12.0
            border_size: 1.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(
                    self.border_size,
                    self.border_size,
                    self.rect_size.x - self.border_size * 2.0,
                    self.rect_size.y - self.border_size * 2.0,
                    self.border_radius
                );
                sdf.fill_keep(self.reaction_bg_color);
                sdf.stroke(self.reaction_border_color, self.border_size);
                return sdf.result;
            }
        }
        draw_text: {
            text_style: { font_size: 12.0 },
            color: #333,
        }
    }

    /// Post card widget displaying a single post in a feed.
    pub SocialPostCard = {{SocialPostCard}} {
        width: Fill,
        height: Fit,
        padding: 16,
        flow: Down,
        spacing: 12,
        show_bg: true,
        draw_bg: {
            color: (CARD_BG_COLOR),
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 0.);
                sdf.fill(self.color);
                // Bottom border
                sdf.rect(0., self.rect_size.y - 1., self.rect_size.x, 1.);
                sdf.fill((CARD_BORDER_COLOR));
                return sdf.result;
            }
        }

        // Header: Avatar, name, username, timestamp
        header = <View> {
            width: Fill,
            height: Fit,
            flow: Right,
            spacing: 12,
            align: { y: 0.0 },

            author_avatar = <Avatar> {
                width: 48,
                height: 48,
            }

            author_info = <View> {
                width: Fill,
                height: Fit,
                flow: Down,
                spacing: 2,

                name_row = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Right,
                    spacing: 8,
                    align: { y: 0.5 },

                    author_name = <Label> {
                        width: Fit,
                        height: Fit,
                        text: "",
                        draw_text: {
                            text_style: { font_size: 14.0 },
                            color: #000,
                        }
                    }

                    author_username = <Label> {
                        width: Fit,
                        height: Fit,
                        text: "",
                        draw_text: {
                            text_style: { font_size: 14.0 },
                            color: #666,
                        }
                    }

                    timestamp = <Label> {
                        width: Fit,
                        height: Fit,
                        text: "",
                        draw_text: {
                            text_style: { font_size: 14.0 },
                            color: #666,
                        }
                    }
                }

                edited_indicator = <Label> {
                    width: Fit,
                    height: Fit,
                    visible: false,
                    text: "(edited)",
                    draw_text: {
                        text_style: { font_size: 12.0 },
                        color: #999,
                    }
                }
            }

            // More options button
            more_button = <Button> {
                width: 32,
                height: 32,
                text: "⋯",
                draw_bg: {
                    color: #0000,
                }
                draw_text: {
                    color: #666,
                    text_style: { font_size: 18.0 }
                }
            }
        }

        // Content section
        content_section = <View> {
            width: Fill,
            height: Fit,
            flow: Down,
            spacing: 12,
            margin: { left: 60 },

            // Text content
            text_content = <Label> {
                width: Fill,
                height: Fit,
                text: "",
                draw_text: {
                    text_style: { font_size: 14.0 },
                    color: #333,
                    wrap: Word,
                }
            }

            // Media content (image/video)
            media_container = <View> {
                width: Fill,
                height: Fit,
                visible: false,

                media_image = <Image> {
                    width: Fill,
                    height: 300,
                    fit: Contain,
                    draw_bg: {
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 12.);
                            let color = self.get_color();
                            sdf.fill(color);
                            return sdf.result;
                        }
                    }
                }
            }

            // Link preview
            link_preview = <View> {
                width: Fill,
                height: Fit,
                visible: false,
                padding: 12,
                show_bg: true,
                draw_bg: {
                    color: #f8f8f8,
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 8.);
                        sdf.fill(self.color);
                        sdf.stroke((CARD_BORDER_COLOR), 1.);
                        return sdf.result;
                    }
                }

                link_content = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Down,
                    spacing: 4,

                    link_title = <Label> {
                        width: Fill,
                        height: Fit,
                        text: "",
                        draw_text: {
                            text_style: { font_size: 14.0 },
                            color: #333,
                        }
                    }

                    link_description = <Label> {
                        width: Fill,
                        height: Fit,
                        text: "",
                        draw_text: {
                            text_style: { font_size: 12.0 },
                            color: #666,
                            wrap: Word,
                        }
                    }

                    link_url = <Label> {
                        width: Fill,
                        height: Fit,
                        text: "",
                        draw_text: {
                            text_style: { font_size: 11.0 },
                            color: #1d9bf0,
                        }
                    }
                }
            }
        }

        // Reactions row
        reactions_row = {{SocialReactionsRow}} {
            width: Fill,
            height: Fit,
            flow: RightWrap,
            spacing: 4,
            margin: { left: 60, top: 4 },
            visible: false,

            // Template for reaction buttons
            reaction_template: <ReactionButton> {}
        }

        // Action bar: Comment, Share, Like, Bookmark
        action_bar = <View> {
            width: Fill,
            height: Fit,
            flow: Right,
            margin: { left: 60 },
            align: { y: 0.5 },

            comment_button = <RobrixIconButton> {
                width: Fit,
                height: 32,
                text: "💬 0",
                draw_bg: {
                    color: #0000,
                }
                draw_text: {
                    color: (ICON_COLOR),
                    text_style: { font_size: 13.0 }
                }
            }

            <View> { width: 40, height: 1 }

            share_button = <RobrixIconButton> {
                width: Fit,
                height: 32,
                text: "🔄 0",
                draw_bg: {
                    color: #0000,
                }
                draw_text: {
                    color: (ICON_COLOR),
                    text_style: { font_size: 13.0 }
                }
            }

            <View> { width: 40, height: 1 }

            like_button = <RobrixIconButton> {
                width: Fit,
                height: 32,
                text: "❤️ 0",
                draw_bg: {
                    color: #0000,
                }
                draw_text: {
                    color: (ICON_COLOR),
                    text_style: { font_size: 13.0 }
                }
            }

            <View> { width: Fill, height: 1 }

            bookmark_button = <RobrixIconButton> {
                width: Fit,
                height: 32,
                text: "🔖",
                draw_bg: {
                    color: #0000,
                }
                draw_text: {
                    color: (ICON_COLOR),
                    text_style: { font_size: 13.0 }
                }
            }
        }
    }
}

/// Data needed to display a post card.
#[derive(Clone, Debug)]
pub struct PostCardData {
    /// Event ID of the post.
    pub event_id: OwnedEventId,
    /// Room ID where the post lives.
    pub room_id: OwnedRoomId,
    /// Author's user ID.
    pub author_id: OwnedUserId,
    /// Author's display name.
    pub author_name: Option<String>,
    /// Post timestamp.
    pub timestamp: MilliSecondsSinceUnixEpoch,
    /// Text content of the post.
    pub text: String,
    /// Whether the post has been edited.
    pub is_edited: bool,
    /// Media URL if the post has media.
    pub media_url: Option<String>,
    /// Link preview data.
    pub link_preview: Option<LinkPreviewData>,
    /// Reaction summary.
    pub reactions: ReactionSummary,
    /// Comment count.
    pub comment_count: u32,
    /// Share/repost count.
    pub share_count: u32,
    /// Whether the current user has liked this post.
    pub is_liked: bool,
    /// Whether the current user has bookmarked this post.
    pub is_bookmarked: bool,
}

/// Link preview data for display.
#[derive(Clone, Debug)]
pub struct LinkPreviewData {
    /// Preview title.
    pub title: Option<String>,
    /// Preview description.
    pub description: Option<String>,
    /// Preview URL.
    pub url: String,
    /// Preview image URL.
    pub image_url: Option<String>,
}

/// Actions that can be triggered from a post card.
#[derive(Clone, Debug, DefaultNone)]
pub enum SocialPostCardAction {
    /// User tapped to view the full post.
    ViewPost(OwnedEventId),
    /// User tapped the author to view their profile.
    ViewAuthorProfile(OwnedUserId),
    /// User tapped to comment on the post.
    Comment(OwnedEventId),
    /// User tapped to share/repost.
    Share(OwnedEventId),
    /// User tapped to like the post.
    Like(OwnedEventId),
    /// User tapped to unlike the post.
    Unlike(OwnedEventId),
    /// User tapped to bookmark the post.
    Bookmark(OwnedEventId),
    /// User tapped to remove bookmark.
    RemoveBookmark(OwnedEventId),
    /// User tapped the more options button.
    ShowMoreOptions(OwnedEventId),
    /// User tapped on a link preview.
    OpenLink(String),
    /// User tapped on media to view full size.
    ViewMedia(OwnedEventId),
    /// User tapped a reaction to add/remove it.
    ToggleReaction {
        event_id: OwnedEventId,
        emoji: String,
    },
    /// No action.
    None,
}

/// Reaction button data for tracking click events.
#[derive(Clone, Debug)]
struct ReactionButtonData {
    emoji: String,
    is_selected: bool,
}

// Color constants for reaction buttons
const REACTION_BG_SELECTED: Vec4 = Vec4 {
    x: 0.91,
    y: 0.96,
    z: 0.99,
    w: 1.0,
}; // #e8f5fd
const REACTION_BG_NORMAL: Vec4 = Vec4 {
    x: 0.94,
    y: 0.95,
    z: 0.96,
    w: 1.0,
}; // #f0f2f5
const REACTION_BORDER_SELECTED: Vec4 = Vec4 {
    x: 0.11,
    y: 0.61,
    z: 0.94,
    w: 1.0,
}; // #1d9bf0
const REACTION_BORDER_NORMAL: Vec4 = Vec4 {
    x: 0.88,
    y: 0.88,
    z: 0.88,
    w: 1.0,
}; // #e0e0e0

/// Widget for displaying a row of reaction buttons.
#[derive(Live, LiveHook, Widget)]
pub struct SocialReactionsRow {
    #[redraw]
    #[rust]
    area: Area,

    /// Template for reaction buttons.
    #[live]
    reaction_template: Option<LivePtr>,

    /// Created reaction buttons with their data.
    #[rust]
    reaction_buttons: Vec<(ButtonRef, ReactionButtonData)>,

    /// Layout for the widget.
    #[layout]
    layout: Layout,

    /// Walk for the widget.
    #[walk]
    walk: Walk,

    /// The event ID these reactions are for.
    #[rust]
    event_id: Option<OwnedEventId>,
}

impl Widget for SocialReactionsRow {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // Handle click events on reaction buttons
        for (button_ref, data) in &self.reaction_buttons {
            if let Hit::FingerUp(fue) = event.hits(cx, button_ref.area()) {
                if fue.is_over && fue.is_primary_hit() && fue.was_tap() {
                    if let Some(event_id) = &self.event_id {
                        cx.action(SocialPostCardAction::ToggleReaction {
                            event_id: event_id.clone(),
                            emoji: data.emoji.clone(),
                        });
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        for (button, _) in self.reaction_buttons.iter_mut() {
            let _ = button.draw(cx, scope);
        }
        cx.end_turtle();
        DrawStep::done()
    }
}

impl SocialReactionsRow {
    /// Set the reactions to display.
    pub fn set_reactions(
        &mut self,
        cx: &mut Cx,
        reactions: &[crate::social::reactions::ReactionDisplay],
        event_id: OwnedEventId,
    ) {
        self.event_id = Some(event_id);
        self.reaction_buttons.clear();

        let Some(template) = self.reaction_template else {
            return;
        };

        for reaction in reactions {
            let button = WidgetRef::new_from_ptr(cx, Some(template)).as_button();
            button.set_text(cx, &format!("{} {}", reaction.emoji, reaction.count));

            // Apply styling based on whether the user has selected this reaction
            let (bg_color, border_color) = if reaction.is_selected {
                (REACTION_BG_SELECTED, REACTION_BORDER_SELECTED)
            } else {
                (REACTION_BG_NORMAL, REACTION_BORDER_NORMAL)
            };

            button.apply_over(
                cx,
                live! {
                    draw_bg: { reaction_bg_color: (bg_color), reaction_border_color: (border_color) }
                },
            );

            self.reaction_buttons.push((
                button,
                ReactionButtonData {
                    emoji: reaction.emoji.clone(),
                    is_selected: reaction.is_selected,
                },
            ));
        }
    }

    /// Clear all reactions.
    pub fn clear(&mut self) {
        self.reaction_buttons.clear();
        self.event_id = None;
    }
}

impl SocialReactionsRowRef {
    /// See [`SocialReactionsRow::set_reactions()`].
    pub fn set_reactions(
        &self,
        cx: &mut Cx,
        reactions: &[crate::social::reactions::ReactionDisplay],
        event_id: OwnedEventId,
    ) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_reactions(cx, reactions, event_id);
        }
    }

    /// See [`SocialReactionsRow::clear()`].
    pub fn clear(&self) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.clear();
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SocialPostCard {
    #[deref]
    view: View,

    /// The event ID of the displayed post.
    #[rust]
    event_id: Option<OwnedEventId>,

    /// The author's user ID.
    #[rust]
    author_id: Option<OwnedUserId>,

    /// Whether the current user has liked this post.
    #[rust]
    is_liked: bool,

    /// Whether the current user has bookmarked this post.
    #[rust]
    is_bookmarked: bool,

    /// Link URL if the post contains a link.
    #[rust]
    link_url: Option<String>,
}

impl Widget for SocialPostCard {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for SocialPostCard {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let Some(event_id) = &self.event_id else {
            return;
        };

        // Handle comment button
        if self.button(ids!(comment_button)).clicked(actions) {
            cx.action(SocialPostCardAction::Comment(event_id.clone()));
        }

        // Handle share button
        if self.button(ids!(share_button)).clicked(actions) {
            cx.action(SocialPostCardAction::Share(event_id.clone()));
        }

        // Handle like button
        if self.button(ids!(like_button)).clicked(actions) {
            if self.is_liked {
                cx.action(SocialPostCardAction::Unlike(event_id.clone()));
            } else {
                cx.action(SocialPostCardAction::Like(event_id.clone()));
            }
        }

        // Handle bookmark button
        if self.button(ids!(bookmark_button)).clicked(actions) {
            if self.is_bookmarked {
                cx.action(SocialPostCardAction::RemoveBookmark(event_id.clone()));
            } else {
                cx.action(SocialPostCardAction::Bookmark(event_id.clone()));
            }
        }

        // Handle more options button
        if self.button(ids!(more_button)).clicked(actions) {
            cx.action(SocialPostCardAction::ShowMoreOptions(event_id.clone()));
        }

        // Handle author avatar click
        if self.view(ids!(author_avatar)).finger_up(actions).is_some() {
            if let Some(author_id) = &self.author_id {
                cx.action(SocialPostCardAction::ViewAuthorProfile(author_id.clone()));
            }
        }

        // Handle media click
        if self
            .view(ids!(media_container))
            .finger_up(actions)
            .is_some()
        {
            cx.action(SocialPostCardAction::ViewMedia(event_id.clone()));
        }

        // Handle link preview click
        if self.view(ids!(link_preview)).finger_up(actions).is_some() {
            if let Some(url) = &self.link_url {
                cx.action(SocialPostCardAction::OpenLink(url.clone()));
            }
        }
    }
}

impl SocialPostCard {
    /// Set the post data to display.
    pub fn set_post(&mut self, cx: &mut Cx, data: &PostCardData) {
        self.event_id = Some(data.event_id.clone());
        self.author_id = Some(data.author_id.clone());
        self.is_liked = data.is_liked;
        self.is_bookmarked = data.is_bookmarked;

        // Set author info
        let display_name = data
            .author_name
            .as_deref()
            .unwrap_or_else(|| data.author_id.localpart());
        self.avatar(ids!(author_avatar)).set_text(cx, display_name);
        self.label(ids!(author_name)).set_text(cx, display_name);
        self.label(ids!(author_username))
            .set_text(cx, &format!("@{}", data.author_id.localpart()));

        // Set timestamp
        let timestamp_text = format_timestamp(data.timestamp);
        self.label(ids!(timestamp)).set_text(cx, &timestamp_text);

        // Set edited indicator
        self.label(ids!(edited_indicator))
            .set_visible(cx, data.is_edited);

        // Set text content
        self.label(ids!(text_content)).set_text(cx, &data.text);

        // Set media if present
        if data.media_url.is_some() {
            self.view(ids!(media_container)).set_visible(cx, true);
            // Note: Actual image loading would be done asynchronously
        } else {
            self.view(ids!(media_container)).set_visible(cx, false);
        }

        // Set link preview if present
        if let Some(preview) = &data.link_preview {
            self.link_url = Some(preview.url.clone());
            if let Some(title) = &preview.title {
                self.label(ids!(link_title)).set_text(cx, title);
            }
            if let Some(description) = &preview.description {
                self.label(ids!(link_description)).set_text(cx, description);
            }
            self.label(ids!(link_url)).set_text(cx, &preview.url);
            self.view(ids!(link_preview)).set_visible(cx, true);
        } else {
            self.link_url = None;
            self.view(ids!(link_preview)).set_visible(cx, false);
        }

        // Set action button counts
        self.button(ids!(comment_button))
            .set_text(cx, &format!("💬 {}", data.comment_count));
        self.button(ids!(share_button))
            .set_text(cx, &format!("🔄 {}", data.share_count));

        // Set like button with state
        let like_count = data.reactions.count("❤️");
        let like_text = if self.is_liked {
            format!("❤️ {}", like_count)
        } else {
            format!("🤍 {}", like_count)
        };
        self.button(ids!(like_button)).set_text(cx, &like_text);

        // Set bookmark button state
        let bookmark_text = if self.is_bookmarked { "🔖" } else { "📑" };
        self.button(ids!(bookmark_button))
            .set_text(cx, bookmark_text);

        // Populate and show reactions row if there are reactions
        let has_reactions = !data.reactions.is_empty();
        if has_reactions {
            // Convert reactions to display format (using None for current user since
            // we track liked state separately via is_liked)
            let display_reactions = reactions_for_display(&data.reactions, None);
            if let Some(mut reactions_row) = self
                .view
                .widget(ids!(reactions_row))
                .borrow_mut::<SocialReactionsRow>()
            {
                reactions_row.set_reactions(cx, &display_reactions, data.event_id.clone());
            }
        }
        self.view(ids!(reactions_row))
            .set_visible(cx, has_reactions);
    }

    /// Update the like state.
    pub fn set_liked(&mut self, cx: &mut Cx, is_liked: bool, count: u32) {
        self.is_liked = is_liked;
        let like_text = if is_liked {
            format!("❤️ {}", count)
        } else {
            format!("🤍 {}", count)
        };
        self.button(ids!(like_button)).set_text(cx, &like_text);
    }

    /// Update the bookmark state.
    pub fn set_bookmarked(&mut self, cx: &mut Cx, is_bookmarked: bool) {
        self.is_bookmarked = is_bookmarked;
        let bookmark_text = if is_bookmarked { "🔖" } else { "📑" };
        self.button(ids!(bookmark_button))
            .set_text(cx, bookmark_text);
    }

    /// Set the media texture for displaying an image in the post.
    ///
    /// This method should be called when the media image has been loaded
    /// from the MediaCache or another source.
    ///
    /// # Arguments
    /// * `cx` - The Makepad context
    /// * `texture` - The texture to display, or None to clear the media
    pub fn set_media_texture(&mut self, cx: &mut Cx, texture: Option<Texture>) {
        let media_image = self.view.image(ids!(media_image));
        media_image.set_texture(cx, texture.clone());

        // Show media container if we have a texture
        self.view(ids!(media_container))
            .set_visible(cx, texture.is_some());
    }

    /// Load media image data into the post card.
    ///
    /// This is a convenience method that creates a texture from raw image data.
    ///
    /// # Arguments
    /// * `cx` - The Makepad context
    /// * `data` - The raw image data (PNG or JPEG)
    ///
    /// # Returns
    /// Ok(()) if the image was loaded successfully, Err otherwise.
    pub fn load_media_from_data(&mut self, cx: &mut Cx, data: &[u8]) -> Result<(), String> {
        let media_image = self.view.image(ids!(media_image));
        crate::utils::load_png_or_jpg(&media_image, cx, data)
            .map_err(|e| format!("Failed to load image: {:?}", e))?;

        // Show the media container
        self.view(ids!(media_container)).set_visible(cx, true);
        Ok(())
    }
}

impl SocialPostCardRef {
    /// See [`SocialPostCard::set_post()`].
    pub fn set_post(&self, cx: &mut Cx, data: &PostCardData) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_post(cx, data);
        }
    }

    /// See [`SocialPostCard::set_liked()`].
    pub fn set_liked(&self, cx: &mut Cx, is_liked: bool, count: u32) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_liked(cx, is_liked, count);
        }
    }

    /// See [`SocialPostCard::set_bookmarked()`].
    pub fn set_bookmarked(&self, cx: &mut Cx, is_bookmarked: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_bookmarked(cx, is_bookmarked);
        }
    }

    /// See [`SocialPostCard::set_media_texture()`].
    pub fn set_media_texture(&self, cx: &mut Cx, texture: Option<Texture>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_media_texture(cx, texture);
        }
    }

    /// See [`SocialPostCard::load_media_from_data()`].
    pub fn load_media_from_data(&self, cx: &mut Cx, data: &[u8]) -> Result<(), String> {
        if let Some(mut inner) = self.borrow_mut() {
            inner.load_media_from_data(cx, data)
        } else {
            Err("Widget not available".to_string())
        }
    }
}

/// Format a timestamp for display.
fn format_timestamp(ts: MilliSecondsSinceUnixEpoch) -> String {
    // Convert to seconds since epoch - UInt needs to use .into() for conversion
    let ts_millis: u64 = ts.get().into();
    let secs = ts_millis / 1000;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let diff = now.saturating_sub(secs);

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m", diff / 60)
    } else if diff < 86400 {
        format!("{}h", diff / 3600)
    } else if diff < 604800 {
        format!("{}d", diff / 86400)
    } else {
        // For older posts, show the date
        let datetime = chrono::DateTime::from_timestamp((secs) as i64, 0);
        datetime
            .map(|dt| dt.format("%b %d").to_string())
            .unwrap_or_else(|| "???".to_string())
    }
}
