
use crate::component::music_page::MusicRecommendPage;
use crate::drive::video_player::VideoPlayer;
use gpui::*;
use gpui_component::{Root, h_flex, v_flex};
use std::time::Duration;
use crate::component::video_page::VideoRecommendPage;

#[derive(PartialEq, Clone, Copy)]
pub enum Page {
    MusicRecommendPage,
    VideoRecommendPage,
}

pub struct HomeView {
    select_id: Page,
    music_recommend_page: Entity<MusicRecommendPage>,
    video_recommend_page: Entity<VideoRecommendPage>,
    video_player_page: Entity<VideoPlayer>,
}

impl HomeView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> HomeView {
        let s = HomeView {
            select_id: Page::MusicRecommendPage,
            music_recommend_page: cx.new(|cx| MusicRecommendPage::new(window, cx)),
            video_recommend_page: cx.new(|cx| VideoRecommendPage::new(window, cx)),
            video_player_page: cx.new(|cx| VideoPlayer::new(window, cx)),
        };
        s
    }
}

pub fn rgb_to_u32(r: u8, g: u8, b: u8) -> Rgba {
    let color: u32 = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
    rgb(color)
}



impl HomeView {
    fn render_nav_item(&self, label: &'static str, page: Page, cx: &Context<Self>) -> impl Element {
        let is_selected = self.select_id == page.clone();

        div()
            .id(label)
            .child(label)
            .w_full()
            .h(px(50.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .cursor_pointer()
            .hover(move |mut style| {
                if !is_selected {
                    style.background = Some(rgb_to_u32(220, 225, 233).into());
                }
                style
            })
            .on_click(cx.listener(move |this, _, _, _| {
                this.select_id = page;
            }))
            .bg(if is_selected {
                rgb_to_u32(211, 227, 253)
            } else {
                rgb_to_u32(233, 238, 246)
            })
    }
}

impl Render for HomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content_anim_id = match self.select_id {
            Page::MusicRecommendPage => "home-view-recommend",
            Page::VideoRecommendPage => "video-player-recommend",
        };
        v_flex()
            .size_full()
            .child(
                h_flex()
                    .size_full()
                    .child(
                        v_flex()
                            .justify_start()
                            .p_2()
                            .gap_2()
                            .h_full()
                            .w(px(80.))
                            .bg(rgb_to_u32(233, 238, 246))
                            // .rounded_2xl()
                            .child(self.render_nav_item("音乐", Page::MusicRecommendPage, cx))
                            .child(self.render_nav_item("视频", Page::VideoRecommendPage, cx))
                    )
                    .child(
                        v_flex().size_full().child(
                            div()
                                .size_full()
                                .child(match self.select_id {
                                    Page::MusicRecommendPage => self.music_recommend_page.clone().into_any_element(),
                                    Page::VideoRecommendPage => self.video_recommend_page.clone().into_any_element(),
                                })
                                .with_animations(
                                    content_anim_id,
                                    vec![
                                        Animation::new(Duration::from_millis(500)).with_easing(ease_in_out),
                                        // Animation::new(Duration::from_millis(300))
                                        //     .with_easing(ease_in_out),
                                    ],
                                    |el, ix, delta| match ix {
                                        _ => el.opacity(delta),
                                        // _ => el.opacity(delta),
                                    },
                                ),
                        ), // .child(Divider::horizontal().w_full())
                        // .child(self.music_play_component.clone()),
                    )
                    .children(Root::render_dialog_layer(window, cx))
                    .children(Root::render_notification_layer(window, cx))
                    .children(Root::render_sheet_layer(window, cx))
            )

    }
}
