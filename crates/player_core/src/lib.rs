use gpui::*;
use gpui_component::v_flex;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

mod core;
mod external;
pub mod state;
mod ui;

pub(crate) use self::core::PlatState;
use self::core::{FramePipeline, PlaybackRuntime};
pub use self::core::{
    PlayCoreDownload, PlayCoreDownloadRequest, PlayCoreExport, PlayCoreExportRequest,
    PlayCoreExportTrim, PlayCoreRealtimeTranscodeRequest, PlayCoreTranscodeFormat,
    PlayCoreTranscodeRequest, PlayCoreTranscodeSession, PlayCoreTranscoder,
};
pub use self::external::{PlayCoreMediaType, PlayCoreProgress, PlayCoreViewState};
use self::state::PlayCoreStateEvent::TogglePlay;
pub use self::state::{PlayCoreGlobalState, PlayCoreState, PlayCoreStateEvent};

pub(crate) fn rgb_to_u32(r: u8, g: u8, b: u8) -> Rgba {
    rgb((r as u32) << 16 | (g as u32) << 8 | b as u32)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayCoreFilterKind {
    Brightness,
    Contrast,
    Saturation,
    Hue,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayCoreFilterState {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub hue: f32,
}

impl Default for PlayCoreFilterState {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            hue: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlayStatic {
    pub id: String,
    pub title: String,
    pub url: String,
    pub headers: reqwest::header::HeaderMap,
}

impl Default for PlayStatic {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
        );

        Self {
            id: String::new(),
            title: String::new(),
            url: String::new(),
            headers,
        }
    }
}

pub struct PlayCore {
    pub current_player: PlayStatic,
    window_id: WindowId,
    show_frame: bool,
    playback: PlaybackRuntime,
    frames: FramePipeline,
    volume: f32,
    total_duration: Option<Duration>,
    position: Duration,
    frame_aspect: f32,
    frame_width: f32,
    frame_height: f32,
    frame_rate: f64,
    codec: Option<String>,
    media_type: PlayCoreMediaType,
    filter_state: PlayCoreFilterState,
    segment_end: Option<Duration>,
    surface_bounds: Option<Bounds<Pixels>>,
    is_dragging_progress_bar: bool,
    pending_seek_position: Option<Duration>,
    progress_bar_bounds: Option<Bounds<Pixels>>,
    volume_bar_bounds: Option<Bounds<Pixels>>,
}

impl PlayCore {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_with_frame(window, cx, true)
    }

    pub fn new_controls_only(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_with_frame(window, cx, false)
    }

    fn new_with_frame(window: &mut Window, cx: &mut Context<Self>, show_frame: bool) -> Self {
        let window_id = window.window_handle().window_id();
        let mut s = Self {
            current_player: PlayStatic::default(),
            window_id,
            show_frame,
            playback: PlaybackRuntime::default(),
            frames: FramePipeline::default(),
            volume: 0.6,
            total_duration: None,
            position: Duration::ZERO,
            frame_aspect: 16.0 / 9.0,
            frame_width: 0.0,
            frame_height: 0.0,
            frame_rate: 0.0,
            codec: None,
            media_type: PlayCoreMediaType::Unknown,
            filter_state: PlayCoreFilterState::default(),
            segment_end: None,
            surface_bounds: None,
            is_dragging_progress_bar: false,
            pending_seek_position: None,
            progress_bar_bounds: None,
            volume_bar_bounds: None,
        };
        s.init_subscribe(window_id, cx);
        s
    }

    fn init_subscribe(&mut self, window_id: WindowId, cx: &mut Context<Self>) {
        let state_handler = cx.global::<PlayCoreGlobalState>().0.clone();
        let self_entity_id = cx.entity_id().clone();
        cx.subscribe(
            &state_handler,
            move |this: &mut Self, _model, event: &PlayCoreStateEvent, cx| match event {
                TogglePlay(event_window_id, event_entity_id, data) => {
                    if event_window_id.as_u64() == window_id.as_u64()
                        && self_entity_id == *event_entity_id
                    {
                        this.current_player = data.clone();
                        this.retry(cx);
                        cx.notify();
                    }
                }
                PlayCoreStateEvent::PlayBackFished(..) => {}
            },
        )
        .detach();
    }

    pub fn is_playing(&self) -> bool {
        self.playback.state == PlatState::Playing
    }
}

impl Render for PlayCore {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.frames.drain_retired_images(window);

        if !self.show_frame {
            return self.render_control(window, cx).into_any_element();
        }

        if !self.current_player.title.trim().is_empty() {
            window.set_window_title(&self.current_player.title);
        }

        v_flex()
            .size_full()
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this._handle_file_drop(paths, cx);
            }))
            // .p_3()
            // .gap_3()
            .bg(rgb_to_u32(255, 255, 255))
            .child(
                v_flex()
                    .flex_grow_1()
                    .min_w_0()
                    .min_h_0()
                    .relative()
                    .child(self.render_frame(cx))
                    .child(self.render_control(window, cx)),
            )
            .into_any_element()
    }
}
