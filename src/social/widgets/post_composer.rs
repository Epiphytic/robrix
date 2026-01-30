//! Post composer widget for creating new posts.
//!
//! This widget provides a UI for composing social media posts with
//! text input, media attachments, and audience/privacy selection.

use makepad_widgets::*;
use std::path::PathBuf;

use crate::shared::avatar::AvatarWidgetExt;
use crate::social::feed_room::FeedPrivacy;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::avatar::Avatar;
    use crate::shared::icon_button::RobrixIconButton;

    COMPOSER_BG_COLOR = #fff
    COMPOSER_BORDER_COLOR = #e0e0e0
    INPUT_BG_COLOR = #f5f5f5
    BUTTON_PRIMARY_COLOR = #1d9bf0
    BUTTON_DISABLED_COLOR = #87ceeb

    /// Post composer widget for creating new social posts.
    pub SocialPostComposer = {{SocialPostComposer}} {
        width: Fill,
        height: Fit,
        padding: 16,
        flow: Down,
        spacing: 12,
        show_bg: true,
        draw_bg: {
            color: (COMPOSER_BG_COLOR),
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 0.);
                sdf.fill(self.color);
                // Bottom border
                sdf.rect(0., self.rect_size.y - 1., self.rect_size.x, 1.);
                sdf.fill((COMPOSER_BORDER_COLOR));
                return sdf.result;
            }
        }

        // Header row with avatar and audience selector
        header_row = <View> {
            width: Fill,
            height: Fit,
            flow: Right,
            spacing: 12,
            align: { y: 0.5 },

            user_avatar = <Avatar> {
                width: 40,
                height: 40,
            }

            audience_dropdown = <DropDown> {
                width: Fit,
                height: Fit,
                labels: ["Public", "Friends", "Close Friends"],
            }
        }

        // Text input area
        text_input_container = <View> {
            width: Fill,
            height: Fit,
            padding: 12,
            show_bg: true,
            draw_bg: {
                color: (INPUT_BG_COLOR),
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 8.);
                    sdf.fill(self.color);
                    return sdf.result;
                }
            }

            text_input = <TextInput> {
                width: Fill,
                height: Fit,
                empty_message: "What's on your mind?",
                draw_bg: {
                    color: #0000
                }
                draw_text: {
                    text_style: { font_size: 14.0 },
                    color: #333,
                    fn get_color(self) -> vec4 {
                        return self.color;
                    }
                }
            }
        }

        // Media preview area (shown when media attached)
        media_preview = <View> {
            width: Fill,
            height: 200,
            visible: false,
            padding: 8,
            show_bg: true,
            draw_bg: {
                color: #f0f0f0,
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 8.);
                    sdf.fill(self.color);
                    return sdf.result;
                }
            }

            preview_image = <Image> {
                width: Fill,
                height: Fill,
                fit: Contain,
            }

            remove_media_button = <Button> {
                width: 24,
                height: 24,
                margin: { left: -32, top: 4 },
                text: "×",
                draw_bg: {
                    color: #00000080,
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.circle(self.rect_size.x / 2., self.rect_size.y / 2., self.rect_size.x / 2.);
                        sdf.fill(self.color);
                        return sdf.result;
                    }
                }
                draw_text: {
                    color: #fff,
                    text_style: { font_size: 16.0 }
                }
            }
        }

        // Link preview (shown when URL detected)
        link_preview_container = <View> {
            width: Fill,
            height: Fit,
            visible: false,
            padding: 12,
            margin: { top: 8 },
            show_bg: true,
            draw_bg: {
                color: #f8f8f8,
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 8.);
                    sdf.fill(self.color);
                    // Left border accent
                    sdf.rect(0., 0., 4., self.rect_size.y);
                    sdf.fill((BUTTON_PRIMARY_COLOR));
                    return sdf.result;
                }
            }

            link_preview_content = <View> {
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
                        color: #999,
                    }
                }
            }
        }

        // Character count
        char_count_row = <View> {
            width: Fill,
            height: Fit,
            flow: Right,
            align: { x: 1.0 },

            char_count_label = <Label> {
                width: Fit,
                height: Fit,
                text: "0/500",
                draw_text: {
                    text_style: { font_size: 12.0 },
                    color: #999,
                }
            }
        }

        // Action bar
        action_bar = <View> {
            width: Fill,
            height: Fit,
            flow: Right,
            spacing: 8,
            align: { y: 0.5 },

            attach_photo_button = <RobrixIconButton> {
                width: 36,
                height: 36,
                text: "📷",
                draw_bg: {
                    color: #0000,
                    border_size: 1.0,
                    border_color: #ddd,
                }
            }

            attach_video_button = <RobrixIconButton> {
                width: 36,
                height: 36,
                text: "🎥",
                draw_bg: {
                    color: #0000,
                    border_size: 1.0,
                    border_color: #ddd,
                }
            }

            attach_link_button = <RobrixIconButton> {
                width: 36,
                height: 36,
                text: "🔗",
                draw_bg: {
                    color: #0000,
                    border_size: 1.0,
                    border_color: #ddd,
                }
            }

            <View> { width: Fill, height: 1 }

            post_button = <Button> {
                width: 80,
                height: 36,
                text: "Post",
                draw_bg: {
                    color: (BUTTON_DISABLED_COLOR),
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 18.);
                        sdf.fill(self.color);
                        return sdf.result;
                    }
                }
                draw_text: {
                    color: #fff,
                    text_style: { font_size: 14.0 }
                }
            }
        }
    }
}

/// Media attached to a post being composed.
#[derive(Clone, Debug)]
pub enum AttachedMedia {
    /// An image file to be uploaded.
    Photo {
        /// Local file path.
        path: PathBuf,
        /// MXC URI after upload (if uploaded).
        mxc_uri: Option<matrix_sdk::ruma::OwnedMxcUri>,
    },
    /// A video file to be uploaded.
    Video {
        /// Local file path.
        path: PathBuf,
        /// MXC URI after upload (if uploaded).
        mxc_uri: Option<matrix_sdk::ruma::OwnedMxcUri>,
    },
}

/// Actions that can be triggered from the post composer.
#[derive(Clone, Debug, DefaultNone)]
pub enum SocialPostComposerAction {
    /// User submitted a post.
    SubmitPost {
        /// Text content of the post.
        text: String,
        /// Selected privacy/audience level.
        privacy: FeedPrivacy,
        /// Attached media, if any.
        media: Option<AttachedMedia>,
    },
    /// User wants to attach a photo.
    AttachPhoto,
    /// User wants to attach a video.
    AttachVideo,
    /// User wants to attach a link.
    AttachLink,
    /// User changed the audience selection.
    AudienceChanged(FeedPrivacy),
    /// User removed attached media.
    RemoveMedia,
    /// No action.
    None,
}

/// Maximum character count for posts.
const MAX_POST_LENGTH: usize = 500;

#[derive(Live, LiveHook, Widget)]
pub struct SocialPostComposer {
    #[deref]
    view: View,

    /// Currently selected audience/privacy level.
    #[rust]
    selected_audience: FeedPrivacy,

    /// Attached media, if any.
    #[rust]
    attached_media: Option<AttachedMedia>,

    /// Detected link URL in the text.
    #[rust]
    detected_link: Option<url::Url>,

    /// Current text content.
    #[rust]
    current_text: String,

    /// Whether the post button should be enabled.
    #[rust]
    can_post: bool,
}

impl Widget for SocialPostComposer {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for SocialPostComposer {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Handle text input changes
        if let Some(text) = self.text_input(ids!(text_input)).changed(actions) {
            self.current_text = text;
            self.update_can_post();
            self.update_char_count(cx);
            self.detect_links();
        }

        // Handle audience dropdown
        if let Some(selected) = self.drop_down(ids!(audience_dropdown)).selected(actions) {
            self.selected_audience = match selected {
                0 => FeedPrivacy::Public,
                1 => FeedPrivacy::Friends,
                2 => FeedPrivacy::CloseFriends,
                _ => FeedPrivacy::Public,
            };
            cx.action(SocialPostComposerAction::AudienceChanged(
                self.selected_audience,
            ));
        }

        // Handle button clicks
        if self.button(ids!(attach_photo_button)).clicked(actions) {
            cx.action(SocialPostComposerAction::AttachPhoto);
        }

        if self.button(ids!(attach_video_button)).clicked(actions) {
            cx.action(SocialPostComposerAction::AttachVideo);
        }

        if self.button(ids!(attach_link_button)).clicked(actions) {
            cx.action(SocialPostComposerAction::AttachLink);
        }

        if self.button(ids!(remove_media_button)).clicked(actions) {
            self.attached_media = None;
            self.view(ids!(media_preview)).set_visible(cx, false);
            self.update_can_post();
            cx.action(SocialPostComposerAction::RemoveMedia);
        }

        if self.button(ids!(post_button)).clicked(actions) && self.can_post {
            cx.action(SocialPostComposerAction::SubmitPost {
                text: self.current_text.clone(),
                privacy: self.selected_audience,
                media: self.attached_media.clone(),
            });
            // Clear after posting
            self.clear(cx);
        }
    }
}

impl SocialPostComposer {
    /// Set the user's avatar for display.
    pub fn set_user_avatar(&mut self, cx: &mut Cx, display_name: &str) {
        self.avatar(ids!(user_avatar)).set_text(cx, display_name);
    }

    /// Attach media to the post.
    pub fn attach_media(&mut self, cx: &mut Cx, media: AttachedMedia) {
        self.attached_media = Some(media);
        self.view(ids!(media_preview)).set_visible(cx, true);
        self.update_can_post();
    }

    /// Set the link preview data.
    pub fn set_link_preview(
        &mut self,
        cx: &mut Cx,
        title: Option<&str>,
        description: Option<&str>,
        url: &str,
    ) {
        if let Some(title) = title {
            self.label(ids!(link_title)).set_text(cx, title);
        }
        if let Some(description) = description {
            self.label(ids!(link_description)).set_text(cx, description);
        }
        self.label(ids!(link_url)).set_text(cx, url);
        self.view(ids!(link_preview_container))
            .set_visible(cx, true);
    }

    /// Clear the composer state.
    pub fn clear(&mut self, cx: &mut Cx) {
        self.current_text.clear();
        self.attached_media = None;
        self.detected_link = None;
        self.can_post = false;

        self.text_input(ids!(text_input)).set_text(cx, "");
        self.view(ids!(media_preview)).set_visible(cx, false);
        self.view(ids!(link_preview_container))
            .set_visible(cx, false);
        self.update_char_count(cx);
    }

    /// Check if the post button should be enabled.
    fn update_can_post(&mut self) {
        let has_content = !self.current_text.trim().is_empty() || self.attached_media.is_some();
        let within_limit = self.current_text.len() <= MAX_POST_LENGTH;
        self.can_post = has_content && within_limit;
    }

    /// Update the character count display.
    fn update_char_count(&mut self, cx: &mut Cx) {
        let count = self.current_text.len();
        let text = format!("{}/{}", count, MAX_POST_LENGTH);
        self.label(ids!(char_count_label)).set_text(cx, &text);
    }

    /// Detect URLs in the current text.
    fn detect_links(&mut self) {
        // Simple URL detection - could be enhanced with linkify crate
        for word in self.current_text.split_whitespace() {
            if let Ok(url) = url::Url::parse(word) {
                if url.scheme() == "http" || url.scheme() == "https" {
                    self.detected_link = Some(url);
                    return;
                }
            }
        }
        self.detected_link = None;
    }

    /// Get the current text content.
    pub fn text(&self) -> &str {
        &self.current_text
    }

    /// Get the selected privacy level.
    pub fn privacy(&self) -> FeedPrivacy {
        self.selected_audience
    }

    /// Get the attached media, if any.
    pub fn attached_media(&self) -> Option<&AttachedMedia> {
        self.attached_media.as_ref()
    }
}

impl SocialPostComposerRef {
    /// See [`SocialPostComposer::set_user_avatar()`].
    pub fn set_user_avatar(&self, cx: &mut Cx, display_name: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_user_avatar(cx, display_name);
        }
    }

    /// See [`SocialPostComposer::attach_media()`].
    pub fn attach_media(&self, cx: &mut Cx, media: AttachedMedia) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.attach_media(cx, media);
        }
    }

    /// See [`SocialPostComposer::clear()`].
    pub fn clear(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.clear(cx);
        }
    }
}
