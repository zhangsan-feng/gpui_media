use crate::component::color::rgb_to_u32;
use crate::drive::NetworkStatic;
use crate::drive::video_player::core::{FrameBuffer, PlatState};
use gpui::*;
use gpui_component::input::InputState;
use gpui_component::{VirtualListScrollHandle, h_flex, v_flex};
use gstreamer_app as gst_app;
use gstreamer_app::gst;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub mod control;
mod core;
mod ui;

pub struct VideoPlayer {
    pub current_player: NetworkStatic,
    pub player_list: Vec<NetworkStatic>,
    play_state: PlatState,
    input_text: Entity<InputState>,

    vm_scroll_handle: VirtualListScrollHandle,
    video_player_volume: f32,
    video_frame_pipeline: Option<gst::Element>,
    video_frame_data: Option<gst_app::AppSink>,
    video_total_duration: Option<Duration>,
    video_player_duration: Duration,
    video_frame_size_proportion: f32,
    video_height: f32,
    video_width: f32,
    video_frame_rate: f64,
    video_frame_player_size: Option<Bounds<Pixels>>,
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
            // .p_3()
            // .gap_3()
            .bg(rgb_to_u32(255, 255, 255))
            .child(
                v_flex()
                    .flex_grow_1()
                    .min_w_0()
                    .min_h_0()
                    .relative()
                    .child(self.video_frame_ui(cx))
                    .child(
                        v_flex()
                            .w_full()
                            .p_2()
                            .gap_2()
                            .rounded_xl()
                            .border_1()
                            .border_color(rgb_to_u32(203, 213, 225))
                            .bg(rgb_to_u32(248, 250, 252))
                            // .shadow_lg()
                            .child(self.player_progress_control_ui(window, cx))
                            .child(
                                h_flex()
                                    .w_full()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        h_flex()
                                            .flex_1()
                                            .gap_2()
                                            .justify_end()
                                            .child(self.player_menu_popover_ui(window, cx))
                                            .child(self.player_info_popover_ui(window, cx)),
                                    )
                                    .child(
                                        h_flex()
                                            .flex_1()
                                            .child(self.player_control_ui(cx))
                                            .child(self.player_volume_control_ui(cx)),
                                    )
                                    .child(
                                        h_flex()
                                            .justify_end()
                                            .gap_2()
                                            .child(self.format_time(display_position))
                                            .child("/")
                                            .child(self.format_time(total)),
                                    ),
                            )
                            .with_animations(
                                "video-player-animations",
                                vec![
                                    Animation::new(Duration::from_millis(500))
                                        .with_easing(ease_in_out),
                                ],
                                move |el, _, delta| {
                                    el.h(px(75.) * delta).opacity(0.2 + 0.8 * delta)
                                },
                            ),
                    ),
            )
    }
}
