use std::time::Duration;
use gpui::*;
use gpui_component::*;
use rodio::{MixerDeviceSink, Player};
use crate::component::home::rgb_to_u32;
use crate::entity;

pub mod control;
mod ui;

#[derive(Clone, Copy)]
struct ProgressDrag;

#[derive(Clone, Copy)]
struct VolumeDrag;


pub struct MusicPlayer {
    pub current_player: entity::MusicConvertLayer,
    pub player_list: Vec<entity::MusicConvertLayer>,
    pub is_player: bool,
    vm_scroll_handle: VirtualListScrollHandle,
    play_err: Option<String>,
    device_sink: Option<MixerDeviceSink>,
    player: Option<Player>,
    total_duration: Option<Duration>,
    current_position: Duration,
    is_scrubbing: bool,
    scrub_position: Option<Duration>,
    volume: f32,
    progress_task: Option<Task<()>>,
    duration_task: Option<Task<()>>,
    progress_bar_bounds: Option<Bounds<Pixels>>,
    volume_bar_bounds: Option<Bounds<Pixels>>,
}


impl Render for MusicPlayer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {


        v_flex()
            .size_full()
            .bg(rgb_to_u32(248, 250, 252))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            
            .child(
                v_flex()
                    .gap_2()
                    .p_2()
                    .rounded_md()
                    .border_2()
                    .border_color(rgb(0xE2E8F0))
                    .child(self.player_progress_control_ui(window, cx))

                    .child(
                        h_flex()
                            .justify_center()
                            .gap_4()
                            .child(self.player_list_ui(window, cx))
                            .child(self.player_control_ui(window, cx))
                            .child(self.player_volume_control_ui(window, cx))
                    ),
            )
    }
}