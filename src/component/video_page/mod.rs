mod recommend;
mod search;

use crate::com::window_center_options;
use crate::component::home::rgb_to_u32;
use crate::drive::video_player::VideoPlayer;
use crate::state::{GlobalState, StateEvent};
use crate::{drive, video_platform};
use gpui::*;
use gpui_component::Root;
use gpui_component::VirtualListScrollHandle;
use gpui_component::button::Button;
use gpui_component::input::Input;
use gpui_component::input::{InputEvent, InputState};
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::{h_flex, v_flex};
use log::info;
use recommend::{VideoSection, VideoSectionData};
use std::collections::HashMap;

pub struct VideoRecommendPage {
    pub recommend_result: Vec<drive::NetworkStatic>,
    pub search_result: HashMap<String, Vec<drive::NetworkStatic>>,
    search_cache: HashMap<String, HashMap<String, Vec<drive::NetworkStatic>>>,
    active_search_keyword: String,
    search_keyword: Entity<InputState>,
    is_loading: bool,
    is_searching: bool,
    layout_scroll_handle: VirtualListScrollHandle,
    search_scroll_handle: VirtualListScrollHandle,
    recommend_scroll_handle: VirtualListScrollHandle,
    movie_scroll_handle: VirtualListScrollHandle,
    tv_scroll_handle: VirtualListScrollHandle,
    anime_scroll_handle: VirtualListScrollHandle,
}

impl VideoRecommendPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> VideoRecommendPage {
        let search_keyword =
            cx.new(|cx| InputState::new(window, cx).placeholder("搜索电影、电视剧、动漫"));
        cx.subscribe(&search_keyword, |_, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        })
        .detach();

        let s = VideoRecommendPage {
            recommend_result: Vec::new(),
            search_result: HashMap::new(),
            search_cache: HashMap::new(),
            active_search_keyword: String::new(),
            search_keyword,
            is_loading: true,
            is_searching: false,
            layout_scroll_handle: VirtualListScrollHandle::new(),
            search_scroll_handle: VirtualListScrollHandle::new(),
            recommend_scroll_handle: VirtualListScrollHandle::new(),
            movie_scroll_handle: VirtualListScrollHandle::new(),
            tv_scroll_handle: VirtualListScrollHandle::new(),
            anime_scroll_handle: VirtualListScrollHandle::new(),
        };
        s.init_data(window, cx);
        // s.open_video_window(window, cx);
        s
    }

    pub fn init_data(&self, _: &mut Window, cx: &mut Context<Self>) {
        if !self.recommend_result.is_empty() {
            return;
        }

        let global_state = cx.global::<GlobalState>().0.clone();
        let tokio_handler = global_state.read(cx).tokio_handle.clone();
        let mut cx_async = cx.to_async().clone();
        let entity = cx.entity().clone();

        cx.spawn(|_, _: &mut AsyncApp| async move {
            let res = tokio_handler.spawn(async move { video_platform::recommend().await });
            match res.await {
                Ok(r) => entity.update(&mut cx_async, |this, cx| {
                    log::info!("video recommend loaded: {}", r.len());
                    this.recommend_result = r;
                    this.is_loading = false;
                    cx.notify()
                }),
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

    fn search_text(&self, cx: &mut Context<Self>) -> String {
        self.search_keyword
            .read(cx)
            .text()
            .to_string()
            .trim()
            .to_string()
    }

    fn search_video(&mut self, cx: &mut Context<Self>) {
        let keyword = self.search_text(cx);
        if keyword.is_empty() {
            self.active_search_keyword.clear();
            self.search_result.clear();
            self.is_searching = false;
            cx.notify();
            return;
        }

        if let Some(cached) = self.search_cache.get(&keyword).cloned() {
            self.active_search_keyword = keyword;
            self.search_result = cached;
            self.is_searching = false;
            cx.notify();
            return;
        }

        self.active_search_keyword = keyword.clone();
        self.search_result.clear();
        self.is_searching = true;
        cx.notify();

        let global_state = cx.global::<GlobalState>().0.clone();
        let tokio_handler = global_state.read(cx).tokio_handle.clone();
        let mut cx_async = cx.to_async().clone();
        let entity = cx.entity().clone();

        cx.spawn(|_, _: &mut AsyncApp| async move {
            let search_keyword = keyword.clone();
            let res = tokio_handler.spawn(async move { video_platform::search(keyword).await });
            match res.await {
                Ok(result) => entity.update(&mut cx_async, |this, cx| {
                    this.search_cache
                        .insert(search_keyword.clone(), result.clone());
                    this.active_search_keyword = search_keyword;
                    this.search_result = result;
                    this.is_searching = false;
                    cx.notify();
                }),
                Err(err) => {
                    let _ = entity.update(&mut cx_async, |this, cx| {
                        this.is_searching = false;
                        cx.notify();
                    });
                    log::error!("video search error: {}", err);
                }
            }
        })
        .detach();
    }

    fn return_to_recommend(&mut self, cx: &mut Context<Self>) {
        self.active_search_keyword.clear();
        self.search_result.clear();
        self.is_searching = false;
        cx.notify();
    }

    fn category_videos(
        videos: &[drive::NetworkStatic],
        category: &str,
    ) -> Vec<drive::NetworkStatic> {
        videos
            .iter()
            .filter(|video| video.category == category)
            .cloned()
            .collect()
    }

    fn recommend_sections(&self) -> Vec<VideoSectionData> {
        let videos = self.recommend_result.clone();
        let has_categories = videos.iter().any(|video| !video.category.is_empty());
        let (recommend, movie, tv, anime) = if has_categories {
            (
                Self::category_videos(&videos, "今日推荐"),
                Self::category_videos(&videos, "电影"),
                Self::category_videos(&videos, "电视剧"),
                Self::category_videos(&videos, "动漫"),
            )
        } else {
            let recommend_end = videos.len().min(12);
            let rest = &videos[recommend_end..];
            let chunk = rest.len().div_ceil(3).max(1);
            let movie_end = chunk.min(rest.len());
            let tv_end = (chunk * 2).min(rest.len());

            (
                videos[..recommend_end].to_vec(),
                rest[..movie_end].to_vec(),
                rest[movie_end..tv_end].to_vec(),
                rest[tv_end..].to_vec(),
            )
        };

        vec![
            VideoSectionData {
                title: "今日推荐".to_string(),
                videos: recommend.clone(),
                offset: 0,
                section: VideoSection::Recommend,
            },
            VideoSectionData {
                title: "电影".to_string(),
                videos: movie.clone(),
                offset: recommend.len(),
                section: VideoSection::Movie,
            },
            VideoSectionData {
                title: "电视剧".to_string(),
                videos: tv.clone(),
                offset: recommend.len() + movie.len(),
                section: VideoSection::Tv,
            },
            VideoSectionData {
                title: "动漫".to_string(),
                videos: anime,
                offset: recommend.len() + movie.len() + tv.len(),
                section: VideoSection::Anime,
            },
        ]
    }

    fn open_window(&self, window: &mut Window, cx: &mut Context<Self>) {
        cx.open_window(
            window_center_options(window, 1300., 700.),
            move |window, app| {
                let view = app.new(|cx| VideoPlayer::new(window, cx));
                app.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("open window failed");
    }

    fn play_video(&self, data: drive::NetworkStatic, window: &mut Window, cx: &mut Context<Self>) {
        let state_handler = cx.global::<GlobalState>().0.clone();
        let tokio_handler = state_handler.read(cx).tokio_handle.clone();
        let mut cx_async = cx.to_async().clone();
        self.open_window(window, cx);
        cx.spawn(|_, _: &mut AsyncApp| async move {
            let res = tokio_handler.spawn(async move { data.func.detail(&data) });
            match res.await {
                Ok(r) => {
                    state_handler.update(&mut cx_async, |_, cx| {
                        cx.emit(StateEvent::UpdateVideoPlayList(r.clone()))
                    });
                }
                Err(e) => info!("tokio run error:{}", e),
            }
        })
        .detach();
    }
}

fn should_show_search(active_search_keyword: &str, is_searching: bool) -> bool {
    !active_search_keyword.is_empty() || is_searching
}

impl Render for VideoRecommendPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content_width = (window.bounds().size.width.as_f32() - 110.).max(Self::CARD_WIDTH);
        let section_width = (content_width - Self::SECTION_SCROLL_GUTTER).max(Self::CARD_WIDTH);
        let cards_per_row = (section_width / (Self::CARD_WIDTH + 12.)).floor().max(1.) as usize;
        let show_search = should_show_search(&self.active_search_keyword, self.is_searching);
        let page_scroll_handle = if show_search {
            &self.search_scroll_handle
        } else {
            &self.layout_scroll_handle
        };
        let return_button = show_search.then(|| {
            Button::new("video-page-return-recommend-btn")
                .label("返回")
                .on_click(cx.listener(|this, _, _, cx| this.return_to_recommend(cx)))
                .into_any_element()
        });

        v_flex()
            .size_full()
            .gap_2()
            .p_2()
            .bg(rgb_to_u32(248, 250, 252))
            .child(
                h_flex()
                    .items_center()
                    .h(px(80.))
                    .w(px(content_width))
                    .gap_2()
                    .p_2()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb_to_u32(226, 232, 240))
                    .bg(rgb_to_u32(255, 255, 255))
                    .children(return_button)
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
                                .on_click(cx.listener(|this, _, _, cx| this.search_video(cx))),
                        ),
                    )
                    .child(
                        div()
                            .rounded_full()
                            .bg(rgb_to_u32(239, 246, 255))
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(37, 99, 235))
                            .child(
                                if show_search{
                                    format!("{} 部", self.search_result.len())
                                }else{
                                    format!("{} 部", self.recommend_result.len())
                                }
                            ),
                    ),
            )
            .child(
                h_flex()
                    .size_full()
                    .flex_1()
                    .overflow_hidden()
                    .gap_2()
                    .child(div().flex_1().h_full().w_full().child(if show_search {
                        self.search_ui(cards_per_row, content_width, section_width, cx)
                            .into_any_element()
                    } else {
                        self.recommend_ui(
                            self.recommend_sections(),
                            cards_per_row,
                            content_width,
                            section_width,
                            cx,
                        )
                        .into_any_element()
                    }))
                    .child(
                        div().w(px(8.)).h_full().child(
                            Scrollbar::vertical(page_scroll_handle)
                                .scrollbar_show(ScrollbarShow::Always)
                                .axis(ScrollbarAxis::Vertical),
                        ),
                    ),
            )
            .into_any_element()
    }
}
