use anyhow::{Result, anyhow};
use gpui::prelude::*;
use gpui::*;
use gpui::{InteractiveElement, StatefulInteractiveElement};
use gpui_component::{h_flex, v_flex, Anchor, WindowExt};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::fs::File;
use std::sync::Arc;
use std::time::Duration;
use gpui_component::button::Button;
use gpui_component::notification::NotificationType;
use gpui_component::popover::Popover;
use gpui_component::scroll::ScrollableElement;
use log::info;
use crate::entity;
use crate::entity::DefaultPlatformInterface;
use crate::state::{GlobalState, StateEvent};
use crate::state::StateEvent::{TogglePlayMusic, UpdatePlatyList};

#[derive(Clone, Copy)]
struct ProgressDrag;

#[derive(Clone, Copy)]
struct VolumeDrag;



pub struct MusicPlayer {
    pub current_player_music: entity::MusicConvertLayer,
    pub player_list: Vec<entity::MusicConvertLayer>,
    pub is_player: bool,
    play_err:Option<String>,
    device_sink: Option<MixerDeviceSink>,
    player: Option<Player>,
    total_duration: Option<Duration>,
    current_position: Duration,
    is_scrubbing: bool,
    scrub_position: Option<Duration>,
    volume: f32,
    progress_task: Option<Task<()>>,
}

fn rgb_u8(r: u8, g: u8, b: u8) -> Rgba {
    let color: u32 = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
    rgb(color)
}

impl MusicPlayer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> MusicPlayer {
        // let _ = (window, cx);
        let mut s = MusicPlayer {
            current_player_music: entity::MusicConvertLayer{
                music_id: "".to_string(),
                music_name: "".to_string(),
                music_author: "".to_string(),
                music_pic: "".to_string(),
                music_platform: "".to_string(),
                music_time: "".to_string(),
                music_source: "".to_string(),
                music_file: "".to_string(),
                func: Arc::new(DefaultPlatformInterface),
            },
            player_list: vec![],
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
        let file = File::open(&self.current_player_music.music_file)?;
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

    fn ensure_track_loaded(&mut self) -> Result<()> {
        self.load_output_deriver()?;
        let needs_load = self.player.as_ref().map(|player| player.len() == 0).unwrap_or(true);
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
                cx.background_executor().timer(Duration::from_millis(200)).await;
                let should_continue = this.update(cx, |this, cx| {
                    this.update_audio_progress(cx)
                }).unwrap_or(false);
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

    fn position_from_drag(&self, position: Point<Pixels>, bounds: Bounds<Pixels>, ) -> Option<Duration> {
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
        let current_index = if !self.current_player_music.music_id.is_empty() {
            self.player_list.iter().position(|music| music.music_id == self.current_player_music.music_id)
        } else if !self.current_player_music.music_source.is_empty() {
            self.player_list.iter().position(|music| music.music_source == self.current_player_music.music_source)
        } else {
            None
        };
        current_index
    }

    fn clean_music(&mut self){
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
        self.current_player_music = self.player_list[next_index].clone();
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
        self.current_player_music = self.player_list[prev_index].clone();
        self.toggle_music(cx);


    }

    fn toggle_music(&mut self, cx: &mut Context<Self>) {
        self.clean_music();

        let mut cx_async = cx.to_async().clone();
        let entity = cx.entity().clone();
        let state_handle = cx.global::<GlobalState>().0.clone();
        let tokio_handler = state_handle.read(cx).clone().tokio_handle;
        let music_layer = self.current_player_music.clone();

        cx.spawn(|_, _:&mut AsyncApp| async move {
            let res = tokio_handler.spawn(async move {
                music_layer.download()
            });

            match res.await {
                Ok(Ok(val)) => {
                    entity.update(&mut cx_async, |this, cx|{
                        this.current_player_music = val;
                        if  this.load_music_source().is_ok(){
                           this.play(cx);
                        }
                        cx.notify()
                    })
                },
                Ok(Err(e)) => {
                    entity.update(&mut cx_async, |this, cx|{
                        this.play_err = Some(e.to_string());
                        this.next_music(cx);
                    });
                    info!("http error: {:?}", e);
                },
                Err(e) => {
                    entity.update(&mut cx_async, |this, cx|{
                        this.play_err =  Some(e.to_string());
                        this.next_music(cx);
                    });
                    info!("tokio runtime error: {:?}", e);
                }
            }
        }).detach();
    }


    fn init_subscribe(&mut self, cx: &mut Context<Self>) {
        let state_handle = cx.global::<GlobalState>().0.clone();

        cx.subscribe(&state_handle, |this: &mut Self, _model, event: &StateEvent, cx| {
            match event {
                TogglePlayMusic(data)=>{
                    println!("{:?}", data.music_source);
                    this.current_player_music = data.clone();
                    this.toggle_music(cx);
                    cx.notify();
                }
                UpdatePlatyList(data)=>{
                    this.player_list = data.clone();
                }
                _ => {
                }
            }

        }).detach();

    }
}

impl Render for MusicPlayer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = window;
        let total = self.total_duration.unwrap_or_else(|| Duration::from_secs(0));
        let display_position = self.scrub_position.filter(|_| self.is_scrubbing).unwrap_or(self.current_position);
        let progress_ratio = if total.as_secs_f32() > 0.0 {
            (display_position.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let volume_ratio = self.volume.clamp(0.0, 1.0);
        let progress_bar_width = 280.0;
        let volume_bar_width = 150.0;


        v_flex()
            .w_full()
            .p_4()
            .gap_4()
            .bg(rgb_u8(248, 250, 252))
            .border_1()
            .border_color(rgb_u8(226, 232, 240))
            .rounded_lg()
            .child(
                h_flex()
                    .gap_4()
                    .items_center()
                    .child(
                        div()
                            .size(px(64.))
                            .rounded_md()
                            .bg(rgb_u8(226, 232, 240))
                            .border_1()
                            .border_color(rgb_u8(203, 213, 225))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(12.))
                            .text_color(rgb_u8(100, 116, 139))
                            .child(img(self.current_player_music.music_pic.clone()).size(px(64.))),
                    )
                    .child(
                        v_flex().gap_2().w(px(300.)).child(
                            v_flex()
                                .gap_1()
                                .child(
                                    div()
                                        .text_size(px(14.))
                                        .text_color(rgb_u8(15, 23, 42))
                                        .child(self.current_player_music.music_name.clone()),
                                )
                                .child(
                                    div()
                                        .h(px(8.))
                                        .w(px(progress_bar_width))
                                        .rounded_full()
                                        .bg(rgb_u8(226, 232, 240))
                                        .cursor_pointer()
                                        .id("music_progress_bar")
                                        .on_drag(ProgressDrag, |_value, _offset, _window, cx| {
                                            cx.new(|_| Empty)
                                        })
                                        .on_drag_move::<ProgressDrag>(cx.listener(|this, event: &DragMoveEvent<ProgressDrag>, _window, _cx| {
                                                if let Some(target) = this.position_from_drag(event.event.position, event.bounds, ) {
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
                                        ),
                                )
                                .child(
                                    h_flex()
                                        .justify_between()
                                        .text_size(px(11.))
                                        .text_color(rgb_u8(100, 116, 139))
                                        .child(Self::format_time(display_position))
                                        .child(Self::format_time(total)),
                                ),
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
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
                                        if self.is_player {
                                        div().child("■")
                                    }else {
                                        div().child("▶")
                                    }),
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
                                    .child(">")

                                ,
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
                                                    let ratio = ((event.event.position.x.as_f32() - left) / width)
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
                    .child(
                        Popover::new("default-open-popover")
                            .anchor(Anchor::BottomRight)
                            .trigger(Button::new("current-play-list").label("播放列表").outline())
                            .child(
                                div()
                                    .h(px(600.))
                                    .w(px(600.))
                                    .overflow_y_scrollbar()
                                    .flex()
                                    .flex_col()
                                    .justify_center()
                                    .gap_2()
                                    .p_4()
                                    .children(self.player_list.iter().enumerate().map(|(index, data)| {
                                        div()
                                            .flex()
                                            .justify_between()
                                            .w_full()
                                            .pr_2()
                                            .child(img(data.music_pic.clone()).size(px(24.)).rounded_full())
                                            .child(data.music_name.clone())
                                            .child(
                                                if self.current_player_music.music_id == data.music_id {
                                                    div()
                                                        .child("正在播放").into_any_element()
                                                }else{
                                                    Button::new(("music-play-index-", index))
                                                        .label("播放")
                                                        .on_click({
                                                            let c = data.clone();
                                                            cx.listener(move |_, _, _ , cx|{
                                                                let mut cx_async = cx.to_async().clone();
                                                                let entity = cx.entity().clone();
                                                                let c = c.clone();
                                                                cx.spawn(|_, _:&mut AsyncApp| async move {
                                                                    entity.update(&mut cx_async, |this, cx|{
                                                                        this.current_player_music = c;
                                                                        this.toggle_music(cx);
                                                                        cx.notify()
                                                                    })
                                                                }).detach()
                                                            })

                                                        }).into_any_element()
                                                }

                                            )
                                    }))
                            )
                    )
            )
    }
}
