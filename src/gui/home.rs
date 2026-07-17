use crate::gui::music_page::MusicPage;
use crate::gui::video_page::VideoPage;
use crate::drive::video_player::VideoPlayer;
use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::{h_flex, v_flex, Root};
use std::time::Duration;
use crate::component::color::rgb_to_u32;

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
}


impl HomeView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> HomeView {
        HomeView {
            select_id: Page::VideoPlayer,
            music_recommend_page: cx.new(|cx| MusicPage::new(window, cx)),
            video_recommend_page: cx.new(|cx| VideoPage::new(window, cx)),
            video_player_page: cx.new(|cx| VideoPlayer::new(window, cx)),
        }
    }

    fn render_nav_item(&self, label: &'static str, page: Page, cx: &Context<Self>) -> impl Element {
        let is_selected = self.select_id == page;
        div()
            .id(label)
            .w_full()
            .h(px(46.))
            .px_3()
            .flex()
            .items_center()
            .gap_3()
            .rounded_lg()
            .cursor_pointer()
            .text_size(px(14.))
            .font_weight(if is_selected { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
            .text_color(if is_selected { rgb_to_u32(37, 32, 61) } else { rgb_to_u32(103, 98, 122) })
            .hover(move |mut style| {
                if !is_selected { style.background = Some(rgb_to_u32(246, 242, 250).into()); }
                style
            })
            .child(
                div()
                    .size(px(28.))
                    .rounded_md()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(10.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(if is_selected { rgb_to_u32(190, 48, 139) } else { rgb_to_u32(145, 140, 162) })
                    .bg(if is_selected { rgb_to_u32(252, 226, 244) } else { rgb_to_u32(244, 241, 248) })
                    .child(match page { Page::MusicPage => "MU", Page::VideoPage => "VI", Page::VideoPlayer => "PL" }),
            )
            .child(label)
            .bg(if is_selected { rgb_to_u32(252, 236, 248) } else { rgb_to_u32(255, 255, 255) })
            .on_click(cx.listener(move |this, _, _, _| { this.select_id = page; }))
    }

    fn render_window_button(
        &self,
        id: &'static str,
        label: &'static str,
        control: WindowControlArea,
        hover_color: Rgba,
        cx: &Context<Self>,
    ) -> AnyElement {
        div()
            .id(id)
            .size(px(34.))
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb_to_u32(250, 247, 252))
            .text_color(rgb_to_u32(91, 82, 108))
            .hover(|style| style.bg(hover_color))
            .window_control_area(control)
            .when(cfg!(target_os = "linux"), move |this| {
                this.on_click(cx.listener(move |_, _, window, _| match control {
                    WindowControlArea::Min => window.minimize_window(),
                    WindowControlArea::Max => window.zoom_window(),
                    WindowControlArea::Close => window.remove_window(),
                    _ => {}
                }))
            })
            .child(
                div()
                    .text_size(px(14.))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(rgb_to_u32(73, 66, 92))
                    .child(label),
            )
            .into_any_element()
    }

    fn render_titlebar(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let minimize = self.render_window_button(
            "custom-titlebar-minimize",
            "−",
            WindowControlArea::Min,
            rgb_to_u32(232, 216, 240),
            cx,
        );
        let maximize = self.render_window_button(
            "custom-titlebar-maximize",
            if window.is_maximized() {
                "❐"
            } else {
                "□"
            },
            WindowControlArea::Max,
            rgb_to_u32(232, 216, 240),
            cx,
        );
        let close = self.render_window_button(
            "custom-titlebar-close",
            "×",
            WindowControlArea::Close,
            rgb_to_u32(244, 202, 215),
            cx,
        );

        h_flex()
            .id("custom-titlebar")
            .w_full()
            .h(px(38.))
            .flex_shrink_0()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(rgb_to_u32(231, 220, 235))
            .bg(rgb_to_u32(250, 247, 252))
            .child(
                h_flex()
                    .id("custom-titlebar-drag")
                    .h_full()
                    .flex_1()
                    .items_center()
                    .px_4()
                    .window_control_area(WindowControlArea::Drag)
                    .child(
                        div()
                            .px_2()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb_to_u32(73, 66, 92))
                            .child(""),
                    ),
            )
            .child(
                h_flex()
                    .h_full()
                    .items_center()
                    .border_l_1()
                    .border_color(rgb_to_u32(231, 220, 235))
                    .gap_0()
                    .children(vec![minimize, maximize, close]),
            )
    }
}

impl Render for HomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content_anim_id = match self.select_id {
            Page::MusicPage => "home-view-recommend",
            Page::VideoPage => "video-player-recommend",
            Page::VideoPlayer => "video-player",
        };

        v_flex().size_full().bg(rgb_to_u32(250, 247, 252))
            .child(self.render_titlebar(window, cx))
            .child(
            h_flex().size_full().flex_1()
                .child(
                    v_flex().justify_start().p_4().gap_3().h_full().w(px(196.))
                        .border_r_1().border_color(rgb_to_u32(238, 232, 244)).bg(rgb_to_u32(255, 255, 255))
                        .child(
                            v_flex()
                                .gap_2()
                                .mt_1()
                                .child(self.render_nav_item("音乐", Page::MusicPage, cx))
                                .child(self.render_nav_item("视频", Page::VideoPage, cx))
                                .child(self.render_nav_item("播放器", Page::VideoPlayer, cx)),
                        ),
                )
                .child(
                    v_flex().size_full().p_5().bg(rgb_to_u32(246, 243, 249))
                        .child(
                            div().size_full().child(match self.select_id {
                                Page::MusicPage => self.music_recommend_page.clone().into_any_element(),
                                Page::VideoPage => self.video_recommend_page.clone().into_any_element(),
                                Page::VideoPlayer => self.video_player_page.clone().into_any_element(),
                            }).with_animations(content_anim_id, vec![Animation::new(Duration::from_millis(300)).with_easing(ease_in_out)], |el, _, delta| el.opacity(delta)),
                        ),
                )
                .children(Root::render_dialog_layer(window, cx))
                .children(Root::render_notification_layer(window, cx))
                .children(Root::render_sheet_layer(window, cx)),
        )
    }
}
