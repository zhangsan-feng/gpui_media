use crate::component::color::rgb_to_u32;
use crate::drive::NetworkStatic;
use gpui::*;
use gpui_component::input::InputState;
use gpui_component::{VirtualListScrollHandle, h_flex, v_flex};
use std::time::Duration;

pub mod control;
mod core;
mod ui;
mod external;

pub(crate) use core::PlatState;
use core::{FramePipeline, PlaybackRuntime};
use crate::drive;
use crate::state::{GlobalState, StateEvent};
use crate::state::StateEvent::{TogglePlayVideo, UpdateVideoPlayList};

pub struct VideoPlayer {
    pub current_player: NetworkStatic,
    pub player_list: Vec<NetworkStatic>,
    input_text: Entity<InputState>,
    vm_scroll_handle: VirtualListScrollHandle,
    playback: PlaybackRuntime,
    frames: FramePipeline,
    volume: f32,
    total_duration: Option<Duration>,
    position: Duration,
    frame_aspect: f32,
    frame_width: f32,
    frame_height: f32,
    frame_rate: f64,
    surface_bounds: Option<Bounds<Pixels>>,
    is_dragging_progress_bar: bool,
    pending_seek_position: Option<Duration>,
    progress_bar_bounds: Option<Bounds<Pixels>>,
    volume_bar_bounds: Option<Bounds<Pixels>>,
}


impl VideoPlayer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let window_id = window.window_handle().window_id();
        let mut s = Self {
            current_player: drive::NetworkStatic::default(),
            player_list: Vec::from([]),
            vm_scroll_handle: VirtualListScrollHandle::new(),
            playback: PlaybackRuntime::default(),
            frames: FramePipeline::default(),
            volume: 0.6,
            total_duration: None,
            position: Duration::ZERO,
            frame_aspect: 16.0 / 9.0,
            frame_width: 0.0,
            frame_height: 0.0,
            frame_rate: 0.0,
            surface_bounds: None,
            is_dragging_progress_bar: false,
            pending_seek_position: None,
            progress_bar_bounds: None,
            volume_bar_bounds: None,
            input_text: cx.new(|cx| InputState::new(window, cx)),
        };
        s.init_subscribe(window_id, cx);
        s
    }

    fn init_subscribe(&mut self, window_id: WindowId, cx: &mut Context<Self>) {
        let state_handler = cx.global::<GlobalState>().0.clone();
        let self_entity_id = cx.entity_id().clone();
        cx.subscribe(
            &state_handler,
            move |this: &mut Self, _model, event: &StateEvent, cx| match event {
                // ############################################################################# 跨组件传递数据
                TogglePlayVideo(event_window_id, event_entity_id, data) => {
                    if event_window_id.as_u64() == window_id.as_u64()
                        && self_entity_id == *event_entity_id
                    {
                        this.current_player = data.clone();
                        cx.notify();
                    }
                }
                UpdateVideoPlayList(event_window_id, event_entity_id, data) => {
                    if event_window_id.as_u64() == window_id.as_u64()
                        && self_entity_id == *event_entity_id
                    {
                        this.player_list = data.clone();
                        cx.notify();
                    }
                }
                _ => {} // ############################################################################# 跨组件传递数据
            },
        )
            .detach();
    }
}

impl Render for VideoPlayer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.frames.drain_retired_images(window);

        let total = self.total_duration.unwrap_or(Duration::ZERO);
        let display_position = self
            .pending_seek_position
            .filter(|_| self.is_dragging_progress_bar)
            .unwrap_or(self.position);

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
                    .child(self.render_video_frame(cx))
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
