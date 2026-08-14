use super::VideoPage;
use crate::component::color::rgb_to_u32;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarMode};
use gpui_component::{h_flex, v_flex};
use std::cmp::min;
use std::rc::Rc;

pub(super) const EPISODES_PER_ROW: usize = 5;
pub(super) const EPISODE_ROW_HEIGHT: f32 = 84.;
const EPISODE_TILE_SIZE: f32 = 64.;

impl VideoPage {
    pub(super) fn render_detail_page(&self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(source) = self.detail_source.as_ref() else {
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb_to_u32(100, 116, 139))
                .child("暂无详情")
                .into_any_element();
        };

        if self.is_detail_loading {
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb_to_u32(100, 116, 139))
                .child("详情加载中")
                .into_any_element();
        }

        let cover = if source.img.trim().is_empty() {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb_to_u32(100, 116, 139))
                .child("暂无封面")
                .into_any_element()
        } else {
            img(source.img.clone())
                .size_full()
                .object_fit(ObjectFit::Cover)
                .into_any_element()
        };

        let episodes = Rc::new(self.detail_result.clone());
        let page = cx.entity().clone();
        let episode_count = episodes.len();
        let title = if source.name.trim().is_empty() {
            "未命名视频"
        } else {
            &source.name
        };
        let source_name = if source.author.trim().is_empty() {
            source.category.clone()
        } else {
            source.author.clone()
        };
        let episode_list = list(self.detail_list_state.clone(), move |row_index, _, _| {
            let start = row_index * EPISODES_PER_ROW;
            let end = min(start + EPISODES_PER_ROW, episodes.len());
            let episode_tiles = (start..end).map(|index| {
                let episode = episodes[index].clone();
                let label = if episode.name.trim().is_empty() {
                    format!("{}", index + 1)
                } else {
                    episode.name.clone()
                };
                let page = page.clone();

                Button::new(("video-episode-", index))
                    .label(label)
                    .outline()
                    .w(px(EPISODE_TILE_SIZE))
                    .h(px(EPISODE_TILE_SIZE))
                    .on_click(move |_, window, app| {
                        let _ = page.update(app, |this, cx| {
                            this.play_episode(episode.clone(), window, cx);
                        });
                    })
            });

            h_flex()
                .w_full()
                .h(px(EPISODE_ROW_HEIGHT))
                .gap_2()
                .items_center()
                .children(episode_tiles)
                .into_any_element()
        })
        .size_full();

        v_flex()
            .size_full()
            .gap_3()
            .child(
                h_flex()
                    .size_full()
                    .min_h_0()
                    .items_stretch()
                    .gap_3()
                    .child(
                        v_flex()
                            .w(px(320.))
                            .h_full()
                            .flex_shrink_0()
                            .min_h_0()
                            .gap_3()
                            .p_3()
                            .rounded_xl()
                            .border_1()
                            .border_color(rgb_to_u32(226, 232, 240))
                            .bg(rgb_to_u32(252, 249, 254))
                            .child(
                                div()
                                    .w_full()
                                    .h(px(250.))
                                    .rounded_lg()
                                    .overflow_hidden()
                                    .bg(rgb_to_u32(241, 245, 249))
                                    .child(cover),
                            )
                            .child(
                                v_flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .w_full()
                                            .text_size(px(20.))
                                            .text_color(rgb_to_u32(15, 23, 42))
                                            .text_ellipsis()
                                            .child(title.to_string()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .text_color(rgb_to_u32(100, 116, 139))
                                            .child(format!("来源：{source_name}")),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .text_color(rgb_to_u32(100, 116, 139))
                                            .child(format!("共 {episode_count} 集")),
                                    )
                                    .child(
                                        div()
                                            .w_full()
                                            .text_size(px(12.))
                                            .text_color(rgb_to_u32(148, 163, 184))
                                            .text_ellipsis()
                                            .child(source.source.clone()),
                                    ),
                            ),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .min_h_0()
                            .gap_2()
                            .child(
                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_size(px(18.))
                                            .text_color(rgb_to_u32(15, 23, 42))
                                            .child("选集"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .text_color(rgb_to_u32(100, 116, 139))
                                            .child(format!("{episode_count} 集")),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .flex_1()
                                    .min_h_0()
                                    .items_stretch()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .h_full()
                                            .min_h_0()
                                            .child(episode_list),
                                    )
                                    .child(
                                        div().w(px(10.)).h_full().child(
                                            Scrollbar::vertical(&self.detail_list_state)
                                                .mode(ScrollbarMode::Always)
                                                .axis(ScrollbarAxis::Vertical),
                                        ),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }
}
