
use crate::component::music_recommend::MusicRecommend;
use crate::component::video_player::VideoPlayer;
use gpui::*;
use gpui_component::{Root, h_flex, v_flex};
use std::time::Duration;


#[derive(PartialEq, Clone, Copy)]
pub enum Page {
    MusicRecommendPage,
    VideoPlayerPage,
}

pub struct HomeView {
    select_id: Page,
    recommend_page: Entity<MusicRecommend>,

    video_player_page: Entity<VideoPlayer>,
}

impl HomeView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> HomeView {
        let window_bounds_subscription = cx.observe_window_bounds(window, |_, _, cx| {
            cx.notify();
        });
        let s = HomeView {
            select_id: Page::MusicRecommendPage,
            recommend_page: cx.new(|cx| MusicRecommend::new(window, cx)),
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
            .h_10()
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
            Page::VideoPlayerPage => "video-player-page",
        };
        v_flex()
            .size_full()
            .child(
                h_flex()
                    .size_full()
                    .child(
                        v_flex()
                            .justify_center()
                            .p_4()
                            .gap_2()
                            .h_full()
                            .w(px(240.))
                            .bg(rgb_to_u32(233, 238, 246))
                            // .rounded_2xl()
                            .child(self.render_nav_item("歌曲推荐", Page::MusicRecommendPage, cx))
                            .child(self.render_nav_item("视频播放器", Page::VideoPlayerPage, cx)),
                    )
                    .child(
                        v_flex().size_full().child(
                            div()
                                .size_full()
                                .child(match self.select_id {
                                    Page::MusicRecommendPage => self.recommend_page.clone().into_any_element(),
                                    Page::VideoPlayerPage => {
                                        self.video_player_page.clone().into_any_element()
                                    }
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
