use super::VideoRecommendPage;
use crate::component::home::rgb_to_u32;
use crate::drive;
use gpui::*;
use gpui_component::VirtualListScrollHandle;
use gpui_component::{h_flex, v_flex, v_virtual_list};
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(in crate::component::video_page) enum VideoSection {
    Recommend,
    Movie,
    Tv,
    Anime,
}

#[derive(Clone)]
pub(in crate::component::video_page) struct VideoSectionData {
    pub(in crate::component::video_page) title: String,
    pub(in crate::component::video_page) videos: Vec<drive::NetworkStatic>,
    pub(in crate::component::video_page) offset: usize,
    pub(in crate::component::video_page) section: VideoSection,
}

impl VideoRecommendPage {
    pub(in crate::component::video_page) const CARD_WIDTH: f32 = 150.;
    pub(in crate::component::video_page) const CARD_HEIGHT: f32 = 200.;
    pub(in crate::component::video_page) const COVER_HEIGHT: f32 = 132.;
    pub(in crate::component::video_page) const ROW_GAP: f32 = 10.;
    pub(in crate::component::video_page) const SECTION_HEADER_HEIGHT: f32 = 28.;
    pub(in crate::component::video_page) const SECTION_GAP: f32 = 10.;
    pub(in crate::component::video_page) const SECTION_PADDING_BOTTOM: f32 = 12.;
    pub(in crate::component::video_page) const SECTION_VISIBLE_ROWS: usize = 2;
    pub(in crate::component::video_page) const SECTION_CONTENT_EXTRA_HEIGHT: f32 = 18.;
    pub(in crate::component::video_page) const SECTION_SCROLL_GUTTER: f32 = 50.;

    pub(in crate::component::video_page) fn card_ui(
        &self,
        data: drive::NetworkStatic,
        index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let img_url = data.img.clone();
        let name = if data.name.is_empty() {
            "未命名视频".to_string()
        } else {
            data.name.clone()
        };

        v_flex()
            .id(format!("video-card-{}-{}", index, data.id.clone()))
            .w(px(Self::CARD_WIDTH))
            .h(px(Self::CARD_HEIGHT))
            .flex_shrink_0()
            .gap_2()
            .p_2()
            .rounded_lg()
            .text_center()
            .border_1()
            .border_color(rgb_to_u32(226, 232, 240))
            .bg(rgb_to_u32(255, 255, 255))
            .cursor_pointer()
            .hover(|style| {
                style
                    .bg(rgb_to_u32(248, 250, 252))
                    .border_color(rgb_to_u32(147, 197, 253))
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                this.play_video(data.clone(), window, cx);
            }))
            .child(if img_url.is_empty() {
                div()
                    .w_full()
                    .h(px(Self::COVER_HEIGHT))
                    .flex_shrink_0()
                    .rounded_md()
                    .bg(rgb_to_u32(226, 232, 240))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(13.))
                    .text_color(rgb_to_u32(100, 116, 139))
                    .child("暂无封面")
                    .into_any_element()
            } else {
                img(img_url)
                    .w_full()
                    .h(px(Self::COVER_HEIGHT))
                    .flex_shrink_0()
                    .rounded_md()
                    .object_fit(ObjectFit::Cover)
                    .into_any_element()
            })
            .child(div().flex_1())
            .child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb_to_u32(15, 23, 42))
                    .child(name),
            )
            .into_any_element()
    }

    fn section_ui(
        &self,
        title: &str,
        videos: &[drive::NetworkStatic],
        offset: usize,
        cards_per_row: usize,
        content_width: f32,
        scroll_handle: &VirtualListScrollHandle,
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
            .child(if videos.is_empty() {
                div()
                    .h(px(80.))
                    .w_full()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb_to_u32(226, 232, 240))
                    .bg(rgb_to_u32(248, 250, 252))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb_to_u32(100, 116, 139))
                    .child("暂无视频")
                    .into_any_element()
            } else {
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
                            format!("recommend-video-vm-list-{}", title),
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
                        .w_full()
                        .track_scroll(scroll_handle),
                    )
                    .into_any_element()
            })
            .into_any_element()
    }

    pub(in crate::component::video_page) fn section_height(row_count: usize) -> f32 {
        Self::SECTION_HEADER_HEIGHT
            + Self::SECTION_GAP
            + if row_count == 0 {
                80.
            } else {
                (Self::CARD_HEIGHT + Self::ROW_GAP)
                    * row_count.min(Self::SECTION_VISIBLE_ROWS).max(1) as f32
                    + Self::SECTION_CONTENT_EXTRA_HEIGHT
            }
            + Self::SECTION_PADDING_BOTTOM
    }

    fn section_scroll_handle(&self, section: VideoSection) -> &VirtualListScrollHandle {
        match section {
            VideoSection::Recommend => &self.recommend_scroll_handle,
            VideoSection::Movie => &self.movie_scroll_handle,
            VideoSection::Tv => &self.tv_scroll_handle,
            VideoSection::Anime => &self.anime_scroll_handle,
        }
    }

    pub(in crate::component::video_page) fn recommend_ui(
        &self,
        sections: Vec<VideoSectionData>,
        cards_per_row: usize,
        layout_width: f32,
        section_width: f32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_virtual_list(
            cx.entity().clone(),
            "recommend-video-section-list",
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
                            .child(div().w(px(section_width)).h_full().child(view.section_ui(
                                &section.title,
                                &section.videos,
                                section.offset,
                                cards_per_row,
                                section_width,
                                view.section_scroll_handle(section.section),
                                cx,
                            )))
                            .child(div().flex_1().h_full())
                    })
                    .collect()
            },
        )
        .w(px(layout_width))
        .track_scroll(&self.layout_scroll_handle)
    }
}
