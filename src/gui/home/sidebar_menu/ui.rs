use super::CustomSidebarMenu;
use crate::component::color::rgb_to_u32;
use crate::gui::home::Page;
use gpui_kit::component::{button::Button, *};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::time::Duration;

impl CustomSidebarMenu {
    fn page_index(page: Page) -> usize {
        match page {
            Page::MusicPage => 0,
            Page::VideoPage => 1,
            Page::MediaWorkspace => 2,
        }
    }

    fn render_item(&self, label: &'static str, page: Page, cx: &Context<Self>) -> impl Element {
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
            .font_weight(if is_selected {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(if is_selected {
                rgb_to_u32(37, 32, 61)
            } else {
                rgb_to_u32(103, 98, 122)
            })
            .hover(move |mut style| {
                if !is_selected {
                    style.background = Some(rgb_to_u32(246, 242, 250).into());
                }
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
                    .text_color(if is_selected {
                        rgb_to_u32(190, 48, 139)
                    } else {
                        rgb_to_u32(145, 140, 162)
                    })
                    .bg(if is_selected {
                        rgb_to_u32(252, 226, 244)
                    } else {
                        rgb_to_u32(244, 241, 248)
                    })
                    .child(match page {
                        Page::MusicPage => "MU",
                        Page::VideoPage => "VI",
                        Page::MediaWorkspace => "PL",
                    }),
            )
            .when(!self.collapsed, |item| item.child(label))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_page(page, cx);
            }))
    }
}

impl CustomSidebarMenu {
    pub(super) fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let previous_index = Self::page_index(self.previous_select_id);
        let selected_index = Self::page_index(self.select_id);

        v_flex()
            .p_4()
            .gap_3()
            .h_full()
            .w(px(if self.collapsed { 84. } else { 196. }))
            .flex_shrink_0()
            .border_r_1()
            .border_color(rgb_to_u32(238, 232, 244))
            .bg(rgb_to_u32(255, 255, 255))
            .justify_start()
            .child(
                Button::new("sidebar-toggle")
                    .icon(if self.collapsed {
                        IconName::ChevronRight
                    } else {
                        IconName::ChevronLeft
                    })
                    .tooltip(if self.collapsed {
                        "展开菜单"
                    } else {
                        "收起菜单"
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_collapsed(cx))),
            )
            .child(
                v_flex()
                    .gap_2()
                    .mt_1()
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .h(px(46.))
                            .rounded_lg()
                            .bg(rgb_to_u32(252, 236, 248))
                            .with_animation(
                                format!("sidebar-selected-background-{selected_index}"),
                                Animation::new(Duration::from_millis(260)).with_easing(ease_in_out),
                                move |el, delta| {
                                    let start = previous_index as f32 * 54.0;
                                    let end = selected_index as f32 * 54.0;
                                    el.top(px(start + (end - start) * delta))
                                },
                            ),
                    )
                    .child(self.render_item("音乐", Page::MusicPage, cx))
                    .child(self.render_item("视频", Page::VideoPage, cx))
                    .child(self.render_item("播放器", Page::MediaWorkspace, cx)),
            )
    }
}
