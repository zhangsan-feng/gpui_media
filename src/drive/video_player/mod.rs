use crate::component::home::rgb_to_u32;
use crate::drive::NetworkStatic;
use gpui::*;
use gpui_component::input::InputState;
use gpui_component::scroll::ScrollableElement;
use gpui_component::text::markdown;
use gpui_component::{VirtualListScrollHandle, h_flex, v_flex};
use gstreamer::prelude::ElementExt;
use gstreamer_app as gst_app;
use gstreamer_app::gst;
use reqwest::header;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub mod control;
mod core;
mod ui;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatState {
    UnLoading,
    Loading,
    Playing,
    Paused,
    Cache(String),
    Error(String),
}

pub struct VideoPlayer {
    pub current_player: NetworkStatic,
    pub player_list: Vec<NetworkStatic>,
    play_state: PlatState,
    input_text: Entity<InputState>,

    video_request_headers: header::HeaderMap,
    vm_scroll_handle: VirtualListScrollHandle,
    video_player_volume: f32,
    video_frame_pipeline: Option<gst::Element>,
    video_frame_data: Option<gst_app::AppSink>,
    video_total_duration: Option<Duration>,
    video_player_duration: Duration,
    video_frame_size: f32,
    video_frame_bounds: Option<Bounds<Pixels>>,
    is_dragging_progress_bar: bool,
    pending_seek_position: Option<Duration>,
    progress_bar_bounds: Option<Bounds<Pixels>>,
    volume_bar_bounds: Option<Bounds<Pixels>>,
    progress_task: Option<Task<()>>,
    frame_task: Option<Task<()>>,
    bus_watch_task: Option<Task<()>>,
    loading_timeout_task: Option<Task<()>>,
    frame_buffer: Arc<Mutex<FrameBuffer>>,
    last_rendered_frame_sequence: u64,
    render_image: Option<Arc<RenderImage>>,
    stop_frames: Arc<AtomicBool>,
    bus_watch_started: bool,
    pending_drop_images: Vec<Arc<RenderImage>>,
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        if let Some(playbin) = &self.video_frame_pipeline {
            let _ = playbin.set_state(gst::State::Null);
        }
        self.stop_frame_thread();
    }
}

#[derive(Clone, Copy)]
struct ProgressDrag;

#[derive(Clone, Copy)]
struct VolumeDrag;

#[derive(Default)]
struct FrameBuffer {
    width: u32,
    height: u32,
    data: Vec<u8>,
    seq: u64,
}

impl Render for VideoPlayer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.free_video_frame(window);

        let total = self
            .video_total_duration
            .unwrap_or_else(|| Duration::from_secs(0));
        let display_position = self
            .pending_seek_position
            .filter(|_| self.is_dragging_progress_bar)
            .unwrap_or(self.video_player_duration);

        v_flex()
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            .size_full()
            .p_2()
            .gap_2()
            .child(self.video_frame_ui(cx))
            .child(
                v_flex()
                    .p_2()
                    .gap_2()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb_to_u32(228, 231, 235))
                    .child(self.player_progress_control_ui(window, cx))
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2()
                            .p_2()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .w(window.bounds().size.width * 0.2)
                                    .overflow_x_scrollbar()
                                    .text_color(rgb_to_u32(15, 23, 42))
                                    .child(
                                        markdown(if self.current_player.source.is_empty() {
                                            "没有加载视频来源".to_string()
                                        } else {
                                            format!(
                                                "{} / {}",
                                                self.current_player.name,
                                                self.current_player.source
                                            )
                                        })
                                        .selectable(true)
                                        .scrollable(false)
                                        .whitespace_nowrap()
                                        .cursor_text(),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(self.player_menu_ui(window, cx))
                                    .child(self.player_control_ui(cx))
                                    .child(self.player_volume_control_ui(cx)),
                            )
                            .child(
                                h_flex()
                                    .child(self.format_time(display_position))
                                    .child("/")
                                    .child(self.format_time(total)),
                            ),
                    ),
            )
    }
}
