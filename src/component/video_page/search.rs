use super::VideoRecommendPage;
use crate::component::home::rgb_to_u32;
use crate::drive;
use gpui::*;
use gpui_component::{h_flex, v_flex, v_virtual_list};
use std::rc::Rc;

#[derive(Clone)]
struct SearchSectionData {
    title: String,
    videos: Vec<drive::NetworkStatic>,
    offset: usize,
}

impl VideoRecommendPage {
    pub(in crate::component::video_page) fn search_ui(
        &self,
        cards_per_row: usize,
        layout_width: f32,
        section_width: f32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let sections = self.search_sections();

        if self.is_searching {
            return div()
                .w(px(layout_width))
                .h(px(96.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_lg()
                .border_1()
                .border_color(rgb_to_u32(226, 232, 240))
                .bg(rgb_to_u32(255, 255, 255))
                .text_color(rgb_to_u32(100, 116, 139))
                .child(format!("正在搜索 {}", self.active_search_keyword))
                .into_any_element();
        }

        if sections.is_empty() {
            return div()
                .w(px(layout_width))
                .h(px(96.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_lg()
                .border_1()
                .border_color(rgb_to_u32(226, 232, 240))
                .bg(rgb_to_u32(255, 255, 255))
                .text_color(rgb_to_u32(100, 116, 139))
                .child(format!("没有找到 {}", self.active_search_keyword))
                .into_any_element();
        }

        v_virtual_list(
            cx.entity().clone(),
            "search-video-section-list",
            Rc::new(
                sections
                    .iter()
                    .map(|section| {
                        size(
                            px(layout_width),
                            px(Self::section_height(
                                section.videos.len().div_ceil(cards_per_row),
                            )),
                        )
                    })
                    .collect(),
            ),
            move |view, visible_range, _, cx| {
                visible_range
                    .map(|index| {
                        let section = sections[index].clone();
                        h_flex()
                            .w(px(layout_width))
                            .h(px(Self::section_height(
                                section.videos.len().div_ceil(cards_per_row),
                            )))
                            .child(div().w(px(section_width)).h_full().child(
                                view.search_section_ui(
                                    &section.title,
                                    &section.videos,
                                    section.offset,
                                    cards_per_row,
                                    section_width,
                                    cx,
                                ),
                            ))
                            .child(div().flex_1().h_full())
                    })
                    .collect()
            },
        )
        .w(px(layout_width))
        .track_scroll(&self.search_scroll_handle)
        .into_any_element()
    }

    fn search_sections(&self) -> Vec<SearchSectionData> {
        let mut offset = 0;
        let mut sections = Vec::new();
        let mut keys: Vec<_> = self.search_result.keys().cloned().collect();
        keys.sort();

        for key in keys {
            let Some(videos) = self.search_result.get(&key) else {
                continue;
            };
            if videos.is_empty() {
                continue;
            }

            sections.push(SearchSectionData {
                title: format!("{} 搜索结果", key),
                videos: videos.clone(),
                offset,
            });
            offset += videos.len();
        }

        sections
    }

    fn search_section_ui(
        &self,
        title: &str,
        videos: &[drive::NetworkStatic],
        offset: usize,
        cards_per_row: usize,
        content_width: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let row_count = videos.len().div_ceil(cards_per_row);
        let row_height = Self::CARD_HEIGHT + Self::ROW_GAP;
        let visible_rows = row_count.min(Self::SECTION_VISIBLE_ROWS).max(1) as f32;
        let videos = videos.to_vec();

        v_flex()
            .w_full()
            .h(px(Self::section_height(row_count)))
            .gap_2()
            .child(
                h_flex()
                    .w_full()
                    .h(px(Self::SECTION_HEADER_HEIGHT))
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .text_size(px(18.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb_to_u32(15, 23, 42))
                            .child(title.to_string()),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .px_3()
                            .py_1()
                            .rounded_full()
                            .bg(rgb_to_u32(241, 245, 249))
                            .text_size(px(11.))
                            .text_color(rgb_to_u32(100, 116, 139))
                            .child(format!("{} 部", videos.len())),
                    ),
            )
            .child(
                div()
                    .p_2()
                    .w_full()
                    .h(px(
                        row_height * visible_rows + Self::SECTION_CONTENT_EXTRA_HEIGHT
                    ))
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb_to_u32(226, 232, 240))
                    .bg(rgb_to_u32(255, 255, 255))
                    .overflow_hidden()
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                    .child(
                        v_virtual_list(
                            cx.entity().clone(),
                            format!("search-video-vm-list-{}", title),
                            Rc::new(
                                (0..row_count)
                                    .map(|_| size(px(content_width), px(row_height)))
                                    .collect(),
                            ),
                            move |view, visible_range, _, cx| {
                                visible_range
                                    .map(|row_index| {
                                        let start = row_index * cards_per_row;
                                        let end =
                                            ((row_index + 1) * cards_per_row).min(videos.len());
                                        h_flex().w_full().gap_2().overflow_hidden().children(
                                            videos[start..end].iter().cloned().enumerate().map(
                                                |(card_index, video)| {
                                                    view.card_ui(
                                                        video,
                                                        offset + start + card_index,
                                                        cx,
                                                    )
                                                },
                                            ),
                                        )
                                    })
                                    .collect()
                            },
                        )
                        .w_full(),
                    ),
            )
            .into_any_element()
    }
}
