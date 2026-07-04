mod ui;

use crate::com::window_center_options;
use crate::drive::video_player::VideoPlayer;
use crate::state::{GlobalState, StateEvent};
use crate::{drive, video_platform};
use gpui::*;
use gpui_component::Root;
use gpui_component::VirtualListScrollHandle;
use gpui_component::input::{InputEvent, InputState};
use log::info;

pub struct VideoRecommendPage {
    pub recommend_video: Vec<drive::NetworkStatic>,
    search_keyword: Entity<InputState>,
    is_loading: bool,
    layout_scroll_handle: VirtualListScrollHandle,
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
            recommend_video: Vec::new(),
            search_keyword,
            is_loading: true,
            layout_scroll_handle: VirtualListScrollHandle::new(),
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
        let global_state = cx.global::<GlobalState>().0.clone();
        let tokio_handler = global_state.read(cx).tokio_handle.clone();
        let mut cx_async = cx.to_async().clone();
        let entity = cx.entity().clone();

        cx.spawn(|_, _: &mut AsyncApp| async move {
            let res = tokio_handler.spawn(async move { video_platform::recommend().await });
            match res.await {
                Ok(r) => entity.update(&mut cx_async, |this, cx| {
                    log::info!("video recommend loaded: {}", r.len());
                    this.recommend_video = r;
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
