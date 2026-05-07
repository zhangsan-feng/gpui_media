use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use gpui::*;
use gpui_component::{h_flex, v_flex, VirtualListScrollHandle};
use gpui_component::input::InputState;
use gstreamer::prelude::ElementExt;
use gstreamer_app::gst;
use gstreamer_app as gst_app;
use reqwest::header;
pub mod control;
mod ui;



pub struct VideoPlayer {
    current_player_video: String,
    player_list: Vec<String>,
    player_func:Arc<dyn Fn(String) -> String + Send + Sync>,
    player_name:String,
    video_request_headers: header::HeaderMap,


    vm_vm_scroll_handle: VirtualListScrollHandle,
    video_player_volume: f32,
    video_frame_pipeline: Option<gst::Element>,
    video_frame_data: Option<gst_app::AppSink>,
    is_player: bool,
    video_total_duration: Option<Duration>,
    video_player_duration: Duration,
    video_aspect: f32,
    is_scrubbing: bool,
    scrub_position: Option<Duration>,
    progress_bar_bounds: Option<Bounds<Pixels>>,
    volume_bar_bounds: Option<Bounds<Pixels>>,
    progress_task: Option<Task<()>>,
    frame_task: Option<Task<()>>,
    bus_watch_task: Option<Task<()>>,
    frame_buffer: Arc<Mutex<FrameBuffer>>,
    last_frame_seq: u64,
    render_image: Option<Arc<RenderImage>>,
    // frame_thread: Option<thread::JoinHandle<()>>,
    stop_frames: Arc<AtomicBool>,
    last_error: Option<String>,
    bus_watch_started: bool,
    pending_drop_images: Vec<Arc<RenderImage>>,
    input_text:Entity<InputState>
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
        self.drop_video_frame(window);

        v_flex()
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            .size_full()
            .p_2()
            .gap_2()
            .child(self.video_frame_ui(window, cx))
            .child(
                v_flex()
                    .gap_2()
                    .p_2()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(0xE2E8F0))
                    .child(self.player_progress_control_ui(window, cx))
                    .child(
                        h_flex()
                            .w_full()
                            .gap_4()
                            .justify_center()
                            .items_center()
                            .child(self.player_list_ui(window, cx))
                            .child(self.player_control_ui(window, cx))
                            .child(self.player_volume_control_ui(window, cx)),
                    )
            )
    }
}
