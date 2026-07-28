mod sidebar_menu;
mod title_bar;

use crate::component::color::rgb_to_u32;
use crate::drive::video_player::VideoPlayer;
use crate::gui::home::sidebar_menu::CustomSidebarMenu;
use crate::gui::home::title_bar::CustomTitleBar;
use crate::gui::music_page::MusicPage;
use crate::gui::video_page::VideoPage;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{Root, h_flex, v_flex};
use std::time::Duration;

#[derive(PartialEq, Clone, Copy)]
pub enum Page {
    MusicPage,
    VideoPage,
    VideoPlayer,
}

pub struct HomeView {
    select_id: Page,
    music_recommend_page: Entity<MusicPage>,
    video_recommend_page: Entity<VideoPage>,
    video_player_page: Entity<VideoPlayer>,
    title_bar: Entity<CustomTitleBar>,
    sidebar_menu: Entity<CustomSidebarMenu>,
}

impl HomeView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> HomeView {
        HomeView {
            title_bar: cx.new(|cx| CustomTitleBar::new(window, cx)),
            sidebar_menu: cx.new(|cx| CustomSidebarMenu::new(window, cx)),
            select_id: Page::VideoPlayer,
            music_recommend_page: cx.new(|cx| MusicPage::new(window, cx)),
            video_recommend_page: cx.new(|cx| VideoPage::new(window, cx)),
            video_player_page: cx.new(|cx| VideoPlayer::new(window, cx)),
        }
    }
}

impl Render for HomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.select_id = self.sidebar_menu.read(cx).select_id.clone();

        let content_anim_id = match self.select_id {
            Page::MusicPage => "home-view-recommend",
            Page::VideoPage => "video-player-recommend",
            Page::VideoPlayer => "video-player",
        };

        v_flex()
            .size_full()
            .bg(rgb_to_u32(250, 247, 252))
            .child(self.title_bar.clone())
            .child(
                h_flex()
                    .size_full()
                    .child(self.sidebar_menu.clone())
                    .child(
                        v_flex()
                            .size_full()
                            .p_5()
                            .bg(rgb_to_u32(246, 243, 249))
                            .child(
                                div()
                                    .size_full()
                                    .child(match self.select_id {
                                        Page::MusicPage => {
                                            self.music_recommend_page.clone().into_any_element()
                                        }
                                        Page::VideoPage => {
                                            self.video_recommend_page.clone().into_any_element()
                                        }
                                        Page::VideoPlayer => {
                                            self.video_player_page.clone().into_any_element()
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
