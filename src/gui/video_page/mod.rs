use crate::component::color::rgb_to_u32;
use crate::drive::NetworkStatic;
use crate::drive::video_player::VideoPlayer;
use crate::plugins::extractor::video;
use crate::state::{GlobalState, StateEvent};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::VirtualListScrollHandle;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::Input;
use gpui_component::input::InputState;
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::{IconName, h_flex, v_flex, v_virtual_list};
use log::info;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy)]
enum Page {
    RecommendPage,
    SearchPage,
}

pub struct VideoPage {
    current_page: Page,
    is_loading: bool,
    is_searching: bool,
    search_keyword: Entity<InputState>,
    recommend_result: Vec<NetworkStatic>,
    search_result: HashMap<String, Vec<NetworkStatic>>,
    vm_scroll_handler: VirtualListScrollHandle,
}

impl VideoPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> VideoPage {
        let mut s = VideoPage {
            current_page: Page::RecommendPage,
            is_loading: false,
            is_searching: false,
            search_keyword: cx.new(|cx| InputState::new(window, cx)),
            recommend_result: Vec::new(),
            search_result: HashMap::new(),
            vm_scroll_handler: VirtualListScrollHandle::new(),
        };
        s.init_data(cx);
        s
    }

    pub fn init_data(&mut self, cx: &mut Context<Self>) {
        let mut cx_async = cx.to_async().clone();
        let entity = cx.entity().clone();
        self.is_loading = true;

        cx.spawn(|_, _: &mut AsyncApp| async move {
            let res = tokio::spawn(async move { video::recommend().await });
            match res.await {
                Ok(r) => {
                    entity.update(&mut cx_async, |this, cx| {
                        this.recommend_result = r;
                        this.is_loading = false;
                        cx.notify()
                    });
                }
                Err(e) => {
                    let _ = entity.update(&mut cx_async, |this, cx| {
                        this.is_loading = false;
                        cx.notify();
                    });
                    log::error!("{}", e)
                }
            }
        })
        .detach();
    }

    pub fn search_video(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        let mut cx_async = cx.to_async().clone();
        let entity = cx.entity().clone();
        self.is_loading = true;
        self.is_searching = true;
        self.current_page = Page::SearchPage;
        self.search_result.clear();
        let search_keyword = self.search_keyword.read(cx).text().to_string();
        cx.spawn(|_, _: &mut AsyncApp| async move {
            let res = tokio::spawn(async move { video::search(search_keyword).await });
            match res.await {
                Ok(r) => {
                    // log::info!("video recommend loaded: {}", r.len());
                    entity.update(&mut cx_async, |this, cx| {
                        this.search_result = r;
                        this.is_loading = false;
                        this.is_searching = false;
                        cx.notify()
                    });
                }
                Err(e) => {
                    let _ = entity.update(&mut cx_async, |this, cx| {
                        this.is_loading = false;
                        this.is_searching = false;
                        cx.notify();
                    });
                    log::error!("{}", e)
                }
            }
        })
        .detach();
    }

    pub fn clear_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.current_page = Page::RecommendPage;
        self.is_searching = false;
        self.search_result.clear();
        self.search_keyword
            .update(cx, |input, cx| input.set_value("", window, cx));
        cx.notify();
    }

    pub(crate) fn play_video(
        &self,
        data: NetworkStatic,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let state_handler = cx.global::<GlobalState>().0.clone();
        let mut cx_async = cx.to_async().clone();
        let (window_id, player_entity_id) = VideoPlayer::open_window(window, cx);

        cx.spawn(move |_, _: &mut AsyncApp| async move {
            let res = tokio::spawn(async move { data.func.detail(&data) });
            match res.await {
                Ok(r) => {
                    state_handler.update(&mut cx_async, |_, cx| {
                        cx.emit(StateEvent::UpdateVideoPlayList(
                            window_id,
                            player_entity_id,
                            r.clone(),
                        ))
                    });
                }
                Err(e) => info!("tokio run error:{}", e),
            }
        })
        .detach();
    }

    fn search_content(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if self.is_searching {
            return self.video_list_content(Vec::new(), "video-search-list", window, cx);
        }

        let items = self
            .search_result
            .values()
            .flat_map(|videos| videos.iter().cloned())
            .collect();

        self.video_list_content(items, "video-search-list", window, cx)
    }

    fn recommend_content(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        self.video_list_content(
            self.recommend_result.clone(),
            "video-recommend-list",
            window,
            cx,
        )
    }

    fn video_list_content(
        &self,
        items: Vec<NetworkStatic>,
        list_id: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        const VIDEO_LIST_ROW_HEIGHT: f32 = 124.;

        if items.is_empty() {
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb_to_u32(100, 116, 139))
                .child(self.placeholder_text())
                .into_any_element();
        }

        let available_width = (window.bounds().size.width.as_f32() - 100.).max(240.);
        let items = Rc::new(items);

        v_virtual_list(
            cx.entity().clone(),
            list_id,
            Rc::new(
                (0..items.len())
                    .map(|_| size(px(available_width), px(VIDEO_LIST_ROW_HEIGHT)))
                    .collect(),
            ),
            move |_view, visible_range, _, cx| {
                visible_range
                    .map(|index| {
                        div()
                            .w_full()
                            .h(px(VIDEO_LIST_ROW_HEIGHT))
                            .pb_3()
                            .child(VideoPage::video_card(items[index].clone(), cx))
                    })
                    .collect()
            },
        )
        .track_scroll(&self.vm_scroll_handler)
        .into_any_element()
    }

    fn placeholder_text(&self) -> &'static str {
        match self.current_page {
            Page::RecommendPage if self.is_loading => "加载中",
            Page::SearchPage if self.is_searching => "搜索中",
            Page::SearchPage => "暂无搜索结果",
            Page::RecommendPage => "暂无推荐内容",
        }
    }

    fn video_card(data: NetworkStatic, cx: &mut Context<Self>) -> AnyElement {
        let cover = if data.img.trim().is_empty() {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(13.))
                .text_color(rgb_to_u32(100, 116, 139))
                .child("暂无封面")
                .into_any_element()
        } else {
            img(data.img.clone())
                .size_full()
                .object_fit(ObjectFit::Cover)
                .into_any_element()
        };
        let mut extra_fields = data.extra.iter().collect::<Vec<_>>();
        extra_fields.sort_by(|left, right| left.0.cmp(right.0));
        let extra_fields = extra_fields
            .into_iter()
            .map(|(key, value)| {
                let value = value
                    .as_str()
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| value.to_string());
                div()
                    .w_full()
                    .min_w_0()
                    .text_size(px(11.))
                    .text_color(rgb_to_u32(148, 163, 184))
                    .text_ellipsis()
                    .child(format!("{key}: {value}"))
            })
            .collect::<Vec<_>>();

        div()
            .id(format!("video-card-{}", data.id))
            .w_full()
            .h(px(112.))
            .rounded_lg()
            .border_1()
            .border_color(rgb_to_u32(226, 232, 240))
            .bg(rgb_to_u32(255, 255, 255))
            .overflow_hidden()
            .cursor_pointer()
            .hover(|style| {
                style
                    .bg(rgb_to_u32(248, 250, 252))
                    .border_color(rgb_to_u32(148, 163, 184))
            })
            .on_click({
                let data = data.clone();
                cx.listener(move |this, _, window, cx| {
                    this.play_video(data.clone(), window, cx);
                })
            })
            .child(
                h_flex()
                    .size_full()
                    .child(
                        div()
                            .w(px(168.))
                            .h_full()
                            .flex_shrink_0()
                            .overflow_hidden()
                            .bg(rgb_to_u32(241, 245, 249))
                            .child(cover),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .p_3()
                            .gap_1()
                            .items_start()
                            .child(
                                div()
                                    .w_full()
                                    .text_size(px(15.))
                                    .text_color(rgb_to_u32(15, 23, 42))
                                    .text_ellipsis()
                                    .child(data.name.clone()),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .text_size(px(12.))
                                    .text_color(rgb_to_u32(100, 116, 139))
                                    .text_ellipsis()
                                    .child(format!(
                                        "来源：{}",
                                        if data.author.is_empty() {
                                            data.category.clone()
                                        } else {
                                            data.author.clone()
                                        }
                                    )),
                            )
                            .children(extra_fields),
                    ),
            )
            .into_any_element()
    }
}

impl Render for VideoPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_3()
            .p_3()
            .bg(rgb_to_u32(255, 255, 255))
            .child(
                h_flex()
                    .items_center()
                    .h(px(64.))
                    .w_full()
                    .gap_3()
                    .px_3()
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb_to_u32(238, 232, 244))
                    .bg(rgb_to_u32(252, 249, 254))
                    .child(
                        div()
                            .flex_grow_1()
                            .child(Input::new(&self.search_keyword).cleanable(true)),
                    )
                    .child(
                        div().child(
                            Button::new("video-page-search-btn")
                                .label(if self.is_searching {
                                    "搜索中"
                                } else {
                                    "搜索"
                                })
                                .on_click({
                                    cx.listener(|this, _, window, cx| this.search_video(window, cx))
                                }),
                        ),
                    )
                    .when(matches!(self.current_page, Page::SearchPage), |this| {
                        this.child(
                            Button::new("video-page-clear-search-btn")
                                .icon(IconName::Close)
                                .ghost()
                                .compact()
                                .tooltip("返回推荐")
                                .on_click({
                                    cx.listener(|this, _, window, cx| this.clear_search(window, cx))
                                }),
                        )
                    })
                    .child(
                        div()
                            .rounded_full()
                            .bg(rgb_to_u32(239, 246, 255))
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(37, 99, 235))
                            .child(match self.current_page {
                                Page::RecommendPage => self.recommend_result.len().to_string(),
                                Page::SearchPage => self
                                    .search_result
                                    .values()
                                    .map(Vec::len)
                                    .sum::<usize>()
                                    .to_string(),
                            }),
                    ),
            )
            .child(
                h_flex()
                    .size_full()
                    .flex_1()
                    .overflow_hidden()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .w(px(window.bounds().size.width.as_f32() - 100.))
                            .child(match self.current_page {
                                Page::RecommendPage => {
                                    self.recommend_content(window, cx).into_any_element()
                                }
                                Page::SearchPage => {
                                    self.search_content(window, cx).into_any_element()
                                }
                            }),
                    )
                    .child(
                        div().w(px(8.)).h_full().child(
                            Scrollbar::vertical(&self.vm_scroll_handler)
                                .scrollbar_show(ScrollbarShow::Always)
                                .axis(ScrollbarAxis::Vertical),
                        ),
                    ),
            )
            .into_any_element()
    }
}
