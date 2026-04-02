use crate::entity;
use crate::entity::{DefaultPlatformInterface, PlatformInterface};
use crate::state::StateEvent::{TogglePlayMusic, UpdatePlatyList};
use crate::state::{GlobalState, StateEvent};
use anyhow::{Result, anyhow};
use gpui::AnimationExt as _;
use gpui::prelude::*;
use gpui::*;
use gpui::{InteractiveElement, StatefulInteractiveElement};
use gpui_component::button::Button;
use gpui_component::popover::Popover;
use gpui_component::scroll::{ScrollableElement, Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::text::markdown;
use gpui_component::{
    Anchor, ElementExt, StyledExt, VirtualListScrollHandle, h_flex, v_flex, v_virtual_list,
};
use log::info;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone, Copy)]
struct ProgressDrag;

#[derive(Clone, Copy)]
struct VolumeDrag;

struct LocalMusicImpl;
impl PlatformInterface for LocalMusicImpl {
    fn download(&self, params: &entity::MusicConvertLayer) -> Result<entity::MusicConvertLayer> {
        Ok(params.clone())
    }
}

pub struct MusicPlayer {
    pub current_player: entity::MusicConvertLayer,
    pub player_list: Vec<entity::MusicConvertLayer>,
    pub is_player: bool,
    scroll_handle: VirtualListScrollHandle,
    play_err: Option<String>,
    device_sink: Option<MixerDeviceSink>,
    player: Option<Player>,
    total_duration: Option<Duration>,
    current_position: Duration,
    is_scrubbing: bool,
    scrub_position: Option<Duration>,
    volume: f32,
    progress_task: Option<Task<()>>,
    progress_bar_bounds: Option<Bounds<Pixels>>,
}

fn rgb_u8(r: u8, g: u8, b: u8) -> Rgba {
    let color: u32 = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
    rgb(color)
}

impl MusicPlayer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> MusicPlayer {
        let mut s = MusicPlayer {
            current_player: entity::MusicConvertLayer {
                music_id: "".to_string(),
                music_name: "当前没有加载音源".to_string(),
                music_author: "".to_string(),
                music_pic: "".to_string(),
                music_platform: "".to_string(),
                music_time: "".to_string(),
                music_source: "".to_string(),
                music_file: "".to_string(),
                func: Arc::new(DefaultPlatformInterface),
            },
            player_list: vec![],
            scroll_handle: VirtualListScrollHandle::new(),

            is_player: false,
            play_err: Option::from("".to_string()),
            device_sink: None,
            player: None,
            total_duration: None,
            current_position: Duration::from_secs(0),
            is_scrubbing: false,
            scrub_position: None,
            volume: 0.6,
            progress_task: None,
            progress_bar_bounds: None,
        };
        s.init_subscribe(cx);
        s
    }

    fn load_output_deriver(&mut self) -> Result<()> {
        if self.device_sink.is_some() && self.player.is_some() {
            return Ok(());
        }
        let sink = DeviceSinkBuilder::open_default_sink()?;
        let player = Player::connect_new(sink.mixer());
        player.set_volume(self.volume);
        self.device_sink = Some(sink);
        self.player = Some(player);
        Ok(())
    }

    fn load_music_source(&mut self) -> Result<()> {
        self.load_output_deriver()?;
        let file = File::open(&self.current_player.music_file)?;
        let decoder = Decoder::try_from(file)?;
        self.total_duration = decoder.total_duration();

        if let Some(player) = &self.player {
            player.clear();
            player.append(decoder);
            player.pause();
            Ok(())
        } else {
            Err(anyhow!("player not initialized"))
        }
    }

    fn track_from_path(&self, path: &Path) -> Option<entity::MusicConvertLayer> {
        if !path.is_file() {
            return None;
        }

        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let file_path = path.to_string_lossy().to_string();

        Some(entity::MusicConvertLayer {
            music_id: Uuid::new_v4().to_string(),
            music_name: file_name,
            music_author: "".to_string(),
            music_pic: "".to_string(),
            music_platform: "".to_string(),
            music_time: "".to_string(),
            music_source: "".to_string(),
            music_file: file_path,
            func: Arc::new(LocalMusicImpl),
        })
    }

    fn handle_file_drop(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        let mut added = Vec::new();
        for path in paths.paths() {
            let Some(track) = self.track_from_path(path) else {
                continue;
            };
            if self
                .player_list
                .iter()
                .any(|item| item.music_file == track.music_file)
            {
                continue;
            }
            added.push(track);
        }

        if added.is_empty() {
            return;
        }

        self.current_player = added[0].clone();
        self.player_list.extend(added);
        self.toggle_music(cx);
        cx.notify();
    }

    fn ensure_track_loaded(&mut self) -> Result<()> {
        self.load_output_deriver()?;
        let needs_load = self
            .player
            .as_ref()
            .map(|player| player.len() == 0)
            .unwrap_or(true);
        if needs_load {
            self.load_music_source()?;
        }
        Ok(())
    }

    fn toggle_play(&mut self, cx: &mut Context<Self>) {
        if self.is_player {
            self.pause();
        } else {
            self.play(cx);
        }
    }

    fn play(&mut self, cx: &mut Context<Self>) {
        if self.current_player.music_file.is_empty() && !self.player_list.is_empty() {
            if let Some(player) = self.player_list.first() {
                self.current_player = player.clone();
                self.toggle_music(cx);
            }
        }
        if self.ensure_track_loaded().is_ok() {
            if let Some(player) = &self.player {
                player.play();
                self.is_player = true;
                self.ensure_progress_task(cx);
            }
        }
    }

    fn pause(&mut self) {
        if let Some(player) = &self.player {
            player.pause();
            self.current_position = player.get_pos();
        }
        self.is_player = false;
    }

    fn seek_audio_progress(&mut self, position: Duration, cx: &mut Context<Self>) {
        if self.ensure_track_loaded().is_ok() {
            if let Some(player) = &self.player {
                let _ = player.try_seek(position);
                self.current_position = position;
                if self.is_player {
                    player.play();
                    self.ensure_progress_task(cx);
                } else {
                    player.pause();
                }
            }
        }
    }

    fn update_audio_progress(&mut self, _cx: &mut Context<Self>) -> bool {
        if let Some(player) = &self.player {
            self.current_position = player.get_pos();
            if player.empty() {
                self.is_player = false;
                self.next_music(_cx)
            }
        }

        let should_continue = self.is_player || self.is_scrubbing;
        if !should_continue {
            self.progress_task = None;
        }
        _cx.notify();
        should_continue
    }

    fn ensure_progress_task(&mut self, cx: &mut Context<Self>) {
        if self.progress_task.is_some() {
            return;
        }
        self.progress_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(200))
                    .await;
                let should_continue = this
                    .update(cx, |this, cx| this.update_audio_progress(cx))
                    .unwrap_or(false);
                if !should_continue {
                    break;
                }
            }
        }));
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(player) = &self.player {
            player.set_volume(self.volume);
        }
    }

    fn position_from_drag(
        &self,
        position: Point<Pixels>,
        bounds: Bounds<Pixels>,
    ) -> Option<Duration> {
        let total = self.total_duration?;
        let left = bounds.origin.x.as_f32();
        let width = bounds.size.width.as_f32().max(1.0);
        let ratio = ((position.x.as_f32() - left) / width).clamp(0.0, 1.0);
        let seconds = total.as_secs_f32() * ratio;
        Some(Duration::from_secs_f32(seconds))
    }

    fn format_time(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    fn get_music_index(&self) -> Option<usize> {
        let current_index = if !self.current_player.music_id.is_empty() {
            self.player_list
                .iter()
                .position(|music| music.music_id == self.current_player.music_id)
        } else if !self.current_player.music_source.is_empty() {
            self.player_list
                .iter()
                .position(|music| music.music_source == self.current_player.music_source)
        } else {
            None
        };
        current_index
    }

    fn clean_music(&mut self) {
        self.pause();
        self.current_position = Duration::from_secs(0);
        self.scrub_position = None;
        self.is_scrubbing = false;
        self.total_duration = None;
    }

    fn next_music(&mut self, cx: &mut Context<Self>) {
        if self.player_list.is_empty() {
            return;
        }

        let current_index = self.get_music_index();
        let next_index = match current_index {
            Some(index) => (index + 1) % self.player_list.len(),
            None => 0,
        };

        self.clean_music();
        self.current_player = self.player_list[next_index].clone();
        self.toggle_music(cx);
    }

    fn prev_music(&mut self, cx: &mut Context<Self>) {
        if self.player_list.is_empty() {
            return;
        }

        let current_index = self.get_music_index();
        let prev_index = match current_index {
            Some(index) => (index + self.player_list.len() - 1) % self.player_list.len(),
            None => 0,
        };

        self.clean_music();
        self.current_player = self.player_list[prev_index].clone();
        self.toggle_music(cx);
    }

    fn toggle_music(&mut self, cx: &mut Context<Self>) {
        self.clean_music();

        let mut cx_async = cx.to_async().clone();
        let entity = cx.entity().clone();
        let state_handle = cx.global::<GlobalState>().0.clone();
        let tokio_handler = state_handle.read(cx).clone().tokio_handle;
        let music_layer = self.current_player.clone();

        cx.spawn(|_, _: &mut AsyncApp| async move {
            let res = tokio_handler.spawn(async move { music_layer.download() });

            match res.await {
                Ok(Ok(val)) => entity.update(&mut cx_async, |this, cx| {
                    this.current_player = val;
                    if this.load_music_source().is_ok() {
                        this.play(cx);
                    }
                    cx.notify()
                }),
                Ok(Err(e)) => {
                    entity.update(&mut cx_async, |this, cx| {
                        this.play_err = Some(e.to_string());
                        // this.next_music(cx);
                    });
                    info!("http error: {:?}", e);
                }
                Err(e) => {
                    entity.update(&mut cx_async, |this, cx| {
                        this.play_err = Some(e.to_string());
                        // this.next_music(cx);
                    });
                    info!("tokio runtime error: {:?}", e);
                }
            }
        })
        .detach();
    }

    fn init_subscribe(&mut self, cx: &mut Context<Self>) {
        let state_handle = cx.global::<GlobalState>().0.clone();

        cx.subscribe(
            &state_handle,
            |this: &mut Self, _model, event: &StateEvent, cx| match event {
                TogglePlayMusic(data) => {
                    println!("{:?}", data.music_source);
                    this.current_player = data.clone();
                    this.toggle_music(cx);
                    cx.notify();
                }
                UpdatePlatyList(data) => {
                    this.player_list = data.clone();
                }
                _ => {}
            },
        )
        .detach();
    }

    fn player_progress_control_ui(
        &self,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let total = self
            .total_duration
            .unwrap_or_else(|| Duration::from_secs(0));

        let display_position = self
            .scrub_position
            .filter(|_| self.is_scrubbing)
            .unwrap_or(self.current_position);

        let progress_bar_width = self
            .progress_bar_bounds
            .as_ref()
            .map(|bounds| bounds.size.width.as_f32())
            .unwrap_or(0.0);

        let progress_ratio = if total.as_secs_f32() > 0.0 {
            (display_position.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let progress_bar_entity = cx.entity();

        div()
            .h(px(8.))
            .w_full()
            .rounded_full()
            .bg(rgb_u8(226, 232, 240))
            .cursor_pointer()
            .on_prepaint({
                let progress_bar_entity = progress_bar_entity.clone();
                move |bounds: Bounds<Pixels>, _window: &mut Window, cx: &mut App| {
                    let _ = progress_bar_entity.update(cx, |this, _cx| {
                        this.progress_bar_bounds = Some(bounds);
                    });
                }
            })
            .id("music_progress_bar")
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                    if let Some(bounds) = this.progress_bar_bounds {
                        if let Some(target) = this.position_from_drag(event.position, bounds) {
                            this.seek_audio_progress(target, cx);
                            this.is_scrubbing = false;
                            this.scrub_position = None;
                        }
                    }
                }),
            )
            .on_drag(ProgressDrag, |_value, _offset, _window, cx| {
                cx.new(|_| Empty)
            })
            .on_drag_move::<ProgressDrag>(cx.listener(
                |this, event: &DragMoveEvent<ProgressDrag>, _window, _cx| {
                    if let Some(target) =
                        this.position_from_drag(event.event.position, event.bounds)
                    {
                        this.is_scrubbing = true;
                        this.scrub_position = Some(target);
                    }
                },
            ))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    if this.is_scrubbing {
                        if let Some(target) = this.scrub_position.take() {
                            this.seek_audio_progress(target, cx);
                        }
                        this.is_scrubbing = false;
                    }
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    if this.is_scrubbing {
                        if let Some(target) = this.scrub_position.take() {
                            this.seek_audio_progress(target, cx);
                        }
                        this.is_scrubbing = false;
                    }
                }),
            )
            .child(
                div()
                    .h(px(8.))
                    .w(px(progress_bar_width * progress_ratio))
                    .rounded_full()
                    .bg(rgb_u8(59, 130, 246)),
            )
    }

    fn player_volume_control_ui(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let volume_ratio = self.volume.clamp(0.0, 1.0);
        let volume_bar_width = 150.0;
        h_flex()
            .w_full()
            .gap_4()
            .justify_center()
            .items_center()
            .flex_shrink_0()
            .child(self.player_list_ui(window, cx))
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        div()
                            .size(px(28.))
                            .rounded_full()
                            .bg(rgb_u8(241, 245, 249))
                            .border_1()
                            .border_color(rgb_u8(203, 213, 225))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(12.))
                            .text_color(rgb_u8(15, 23, 42))
                            .id("music_prev_button")
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.prev_music(cx);
                            }))
                            .child("<"),
                    )
                    .child(
                        div()
                            .size(px(36.))
                            .rounded_full()
                            .bg(rgb_u8(59, 130, 246))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(14.))
                            .text_color(white())
                            .cursor_pointer()
                            .id("music_play_button")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.toggle_play(cx);
                            }))
                            .child(
                                (if self.is_player {
                                    div().child("■")
                                } else {
                                    div().child("▶")
                                })
                                .with_animation(
                                    format!("play_toggle_{}", self.is_player),
                                    Animation::new(Duration::from_millis(1000))
                                        .with_easing(ease_in_out),
                                    |el, delta| el.opacity(0.35 + 0.65 * delta),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .size(px(28.))
                            .rounded_full()
                            .bg(rgb_u8(241, 245, 249))
                            .border_1()
                            .border_color(rgb_u8(203, 213, 225))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(12.))
                            .text_color(rgb_u8(15, 23, 42))
                            .cursor_pointer()
                            .id("music_nest_button")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.next_music(cx);
                            }))
                            .child(">"),
                    )
                    .child(
                        h_flex()
                            .w(px(220.))
                            .gap_2()
                            .items_center()
                            .ml_auto()
                            .child(img("icon/icons8-voice-100.png").size(px(24.)))
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(rgb_u8(100, 116, 139))
                                    .child(format!("{:.0}%", volume_ratio * 100.0)),
                            )
                            .child(
                                div()
                                    .h(px(8.))
                                    .w(px(volume_bar_width))
                                    .rounded_full()
                                    .bg(rgb_u8(226, 232, 240))
                                    .cursor_pointer()
                                    .id("music_volume_bar")
                                    .on_drag(VolumeDrag, |_value, _offset, _window, cx| {
                                        cx.new(|_| Empty)
                                    })
                                    .on_drag_move::<VolumeDrag>(cx.listener(
                                        |this, event: &DragMoveEvent<VolumeDrag>, _window, _cx| {
                                            let left = event.bounds.origin.x.as_f32();
                                            let width = event.bounds.size.width.as_f32().max(1.0);
                                            let ratio = ((event.event.position.x.as_f32() - left)
                                                / width)
                                                .clamp(0.0, 1.0);
                                            this.set_volume(ratio);
                                        },
                                    ))
                                    .child(
                                        div()
                                            .h(px(8.))
                                            .w(px(volume_bar_width * volume_ratio))
                                            .rounded_full()
                                            .bg(rgb_u8(148, 163, 184)),
                                    ),
                            ),
                    ),
            )
    }

    fn player_list_vm(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_virtual_list(
            cx.entity().clone(),
            "music-player-vm-list",
            Rc::new(
                self.player_list
                    .iter()
                    .map(|_| size(px(100.), px(40.)))
                    .collect(),
            ),
            |view, visible_range, _, cx| {
                visible_range
                    .map(|index| {
                        let data = view.player_list[index].clone();
                        div()
                            .flex()
                            .justify_between()
                            .w_full()
                            .pr_2()
                            .child(
                                div()
                                    .gap_2()
                                    .justify_between()
                                    .h_flex()
                                    .child(img(data.music_pic.clone()).size(px(24.)).rounded_full())
                                    .child(data.music_author.clone())
                                    .child(data.music_platform.clone())
                                    .child(data.music_name.clone()),
                            )
                            .child(if view.current_player.music_id == data.music_id {
                                div().child("正在播放").into_any_element()
                            } else {
                                Button::new(("music-play-index-", index))
                                    .label("播放")
                                    .on_click({
                                        let c = data.clone();
                                        cx.listener(move |_, _, _, cx| {
                                            let mut cx_async = cx.to_async().clone();
                                            let state_handle = cx.global::<GlobalState>().0.clone();
                                            let c = c.clone();
                                            cx.spawn(|_, _: &mut AsyncApp| async move {
                                                state_handle.update(&mut cx_async, |_, cx| {
                                                    cx.emit(StateEvent::TogglePlayMusic(c.clone()))
                                                });
                                            })
                                            .detach()
                                        })
                                    })
                                    .into_any_element()
                            })
                    })
                    .collect()
            },
        )
        .track_scroll(&self.scroll_handle)
    }

    fn player_list_ui(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Popover::new("default-open-popover")
            .anchor(Anchor::BottomRight)
            .trigger(Button::new("show-form").label("播放列表").outline())
            .child(
                div()
                    .h(px(600.))
                    .w(px(800.))
                    .child(
                        v_flex()
                            .gap_2()
                            .p_4()
                            .size_full()
                            .child(self.player_list_vm(window, cx))
                            .child(
                                Scrollbar::vertical(&self.scroll_handle)
                                    .scrollbar_show(ScrollbarShow::Always)
                                    .axis(ScrollbarAxis::Vertical),
                            ),
                    )
                    .with_animation(
                        "playlist-popover-anim",
                        Animation::new(Duration::from_millis(550)).with_easing(ease_in_out),
                        |el, delta| el.opacity(0.2 + 0.8 * delta).h(px(8. + 592. * delta)),
                    ),
            )
    }
}

impl Render for MusicPlayer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let total = self
            .total_duration
            .unwrap_or_else(|| Duration::from_secs(0));
        let display_position = self
            .scrub_position
            .filter(|_| self.is_scrubbing)
            .unwrap_or(self.current_position);

        v_flex()
            .size_full()
            .p_2()
            .gap_2()
            .bg(rgb_u8(248, 250, 252))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            .child(div().flex_grow())
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
                            .text_size(px(12.))
                            .w_full()
                            .items_center()
                            .child(
                                div()
                                    .flex_1()
                                    .justify_start()
                                    .overflow_y_scrollbar()
                                    .text_color(rgb_u8(15, 23, 42))
                                    .child(
                                        markdown(self.current_player.music_name.clone())
                                            .selectable(true)
                                            .cursor_text(),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(Self::format_time(display_position))
                                    .child("/")
                                    .child(Self::format_time(total)),
                            )
                            .child(div().flex_1()),
                    )
                    .child(self.player_volume_control_ui(window, cx)),
            )
    }
}
