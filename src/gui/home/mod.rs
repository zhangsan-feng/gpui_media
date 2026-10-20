mod media_editor;
mod medio_player;
mod sidebar_menu;

use crate::component::color::rgb_to_u32;
use crate::component::window_title_bar::CustomTitleBar;
use crate::gui::home::media_editor::MediaEditor;
use crate::gui::home::medio_player::MedioPlayer;
use crate::gui::home::sidebar_menu::CustomSidebarMenu;
use crate::gui::music_page::MusicPage;
use crate::gui::video_page::VideoPage;
use gpui_kit::component::{Root, h_flex, v_flex};
use gpui_kit::*;
use std::time::Duration;

#[derive(PartialEq, Clone, Copy)]
pub enum Page {
    MusicPage,
    VideoPage,
    MediaEditor,
    MedioPlayer,
}

pub struct HomeView {
    music_recommend_page: Entity<MusicPage>,
    video_recommend_page: Entity<VideoPage>,
    media_editor: Entity<MediaEditor>,
    medio_player: Entity<MedioPlayer>,
    title_bar: Entity<CustomTitleBar>,
    sidebar_menu: Entity<CustomSidebarMenu>,
}

impl HomeView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> HomeView {
        HomeView {
            title_bar: cx.new(|cx| CustomTitleBar::new(window, cx)),
            sidebar_menu: cx.new(|cx| CustomSidebarMenu::new(window, cx)),
            music_recommend_page: cx.new(|cx| MusicPage::new(window, cx)),
            video_recommend_page: cx.new(|cx| VideoPage::new(window, cx)),
            media_editor: cx.new(|cx| MediaEditor::new(window, cx)),
            medio_player: cx.new(|cx| MedioPlayer::new(window, cx)),
        }
    }
}

impl Render for HomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let select_page = self.sidebar_menu.read(cx).select_id.clone();

        let content_anim_id = match select_page {
            Page::MusicPage => "home-view-recommend",
            Page::VideoPage => "video-player-recommend",
            Page::MediaEditor => "media-editor",
            Page::MedioPlayer => "medio-player",
        };

        v_flex()
            .size_full()
            .bg(rgb_to_u32(250, 247, 252))
            .child(self.title_bar.clone())
            .child(
                h_flex()
                    .size_full()
                    .min_w_0()
                    .min_h_0()
                    .child(self.sidebar_menu.clone())
                    .child(
                        v_flex()
                            .flex_1()
                            .h_full()
                            .min_w_0()
                            .min_h_0()
                            .p_5()
                            .bg(rgb_to_u32(246, 243, 249))
                            .child(
                                div()
                                    .flex_1()
                                    .h_full()
                                    .min_w_0()
                                    .min_h_0()
                                    .child(match select_page {
                                        Page::MusicPage => {
                                            self.music_recommend_page.clone().into_any_element()
                                        }
                                        Page::VideoPage => {
                                            self.video_recommend_page.clone().into_any_element()
                                        }
                                        Page::MediaEditor => {
                                            self.media_editor.clone().into_any_element()
                                        }
                                        Page::MedioPlayer => {
                                            self.medio_player.clone().into_any_element()
                                        }
                                    })
                                    .with_animations(
                                        content_anim_id,
                                        vec![
                                            Animation::new(Duration::from_millis(500))
                                                .with_easing(ease_in_out),
                                        ],
                                        |el, _, delta| el.opacity(0.2 + 0.8 * delta),
                                    ),
                            ),
                    )
                    .children(Root::render_dialog_layer(window, cx))
                    .children(Root::render_notification_layer(window, cx))
                    .children(Root::render_sheet_layer(window, cx)),
            )
    }
}
