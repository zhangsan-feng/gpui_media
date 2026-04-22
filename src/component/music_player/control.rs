use crate::com::rgb_u8;
use crate::entity;
use crate::entity::{DefaultPlatformInterface};
use crate::state::StateEvent::{TogglePlayMusic, UpdatePlatyList};
use crate::state::{GlobalState, StateEvent};
use anyhow::{Result, anyhow};
use gpui::prelude::*;
use gpui::*;
use gpui_component::*;
use log::info;
use rodio::{Decoder, DeviceSinkBuilder,  Player, Source};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use url::Url;
use uuid::Uuid;
use crate::component::music_player::{MusicPlayer, ProgressDrag, VolumeDrag};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::prelude::ElementExt;

impl MusicPlayer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> MusicPlayer {
        let _ = window;
        let mut s = MusicPlayer {
            current_player: entity::MusicConvertLayer {
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
            scroll_handle: VirtualListScrollHandle::new(),
            is_player: false,
            play_err: None,
            device_sink: None,
            player: None,
            total_duration: None,
            current_position: Duration::from_secs(0),
            is_scrubbing: false,
            scrub_position: None,
            volume: 0.6,
            progress_task: None,
            duration_task: None,
            progress_bar_bounds: None,
            volume_bar_bounds: None,
        };
        s.init_subscribe(cx);
        s
    }
    
    fn init_subscribe(&mut self, cx: &mut Context<Self>) {
        let state_handle = cx.global::<GlobalState>().0.clone();

        cx.subscribe(
            &state_handle,
            |this: &mut Self, _model, event: &StateEvent, cx| match event {
                TogglePlayMusic(data) => {
                    // println!("{:?}", data.music_source);
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

    fn load_music_source(&mut self, cx: &mut Context<Self>) -> Result<()> {
        self.load_output_deriver()?;
        let file = File::open(&self.current_player.music_file)?;
        let decoder = Decoder::try_from(file)?;
        self.total_duration = decoder.total_duration();
        if self.total_duration.is_none() {
            self.start_duration_scan(cx);
        }

        if let Some(player) = &self.player {
            player.clear();
            player.append(decoder);
            player.pause();
            Ok(())
        } else {
            Err(anyhow!("player not initialized"))
        }
    }

    fn start_duration_scan(&mut self, cx: &mut Context<Self>) {
        if self.duration_task.is_some() {
            return;
        }

        let file_for_scan = self.current_player.music_file.clone();
        let file_for_match = file_for_scan.clone();
        if file_for_scan.is_empty() {
            return;
        }

        let track_id_for_match = self.current_player.music_id.clone();
        let state_handle = cx.global::<GlobalState>().0.clone();
        let tokio_handler = state_handle.read(cx).clone().tokio_handle;
        let entity = cx.entity().clone();
        let mut cx_async = cx.to_async().clone();

        self.duration_task = Some(cx.spawn(|_, _: &mut AsyncApp| async move {
            let duration = tokio_handler
                .spawn_blocking(move || Self::duration_from_gstreamer_path(Path::new(&file_for_scan)))
                .await
                .ok()
                .flatten();

            let _ = entity.update(&mut cx_async, |this, cx| {
                this.duration_task = None;
                let same_track = if !track_id_for_match.is_empty() {
                    this.current_player.music_id == track_id_for_match
                } else {
                    this.current_player.music_file == file_for_match
                };
                if same_track && this.total_duration.is_none() {
                    this.total_duration = duration;
                    cx.notify();
                }
            });
        }));
    }

    fn duration_from_gstreamer_path(file_path: &Path) -> Option<Duration> {


        let canonical = file_path.canonicalize().ok()?;
        let uri = Url::from_file_path(canonical).ok()?.to_string();
        let _ = gst::init();

        let playbin = gst::ElementFactory::make("playbin").build().ok()?;
        let audio_sink = gst::ElementFactory::make("fakesink").build().ok()?;
        let video_sink = gst::ElementFactory::make("fakesink").build().ok()?;
        playbin.set_property("uri", &uri);
        playbin.set_property("audio-sink", &audio_sink);
        playbin.set_property("video-sink", &video_sink);
        let _ = playbin.set_state(gst::State::Paused);

        let mut total_duration = None;
        for _ in 0..10 {
            if let Some(total) = playbin.query_duration::<gst::ClockTime>() {
                if total.nseconds() > 0 {
                    total_duration = Some(Duration::from_nanos(total.nseconds()));
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(30));
        }

        let _ = playbin.set_state(gst::State::Null);
        total_duration
    }

    fn track_from_path(&self, path: &Path) -> Option<entity::MusicConvertLayer> {
        if !path.is_file() {
            return None;
        }

        let file_name = path.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string_lossy().to_string());
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
            func: Arc::new(DefaultPlatformInterface),
        })
    }

    pub(crate) fn handle_file_drop(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        let mut added = Vec::new();
        for path in paths.paths() {
            let Some(track) = self.track_from_path(path) else {
                continue;
            };
            if self.player_list.iter().any(|item| item.music_file == track.music_file){
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

    fn ensure_track_loaded(&mut self, cx: &mut Context<Self>) -> Result<()> {
        self.load_output_deriver()?;
        let needs_load = self.player.as_ref().map(|player| player.len() == 0).unwrap_or(true);
        if needs_load {
            self.load_music_source(cx)?;
        }
        Ok(())
    }

    pub(crate) fn toggle_play(&mut self, cx: &mut Context<Self>) {
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
        if self.ensure_track_loaded(cx).is_ok() {
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

    pub(crate) fn seek_audio_progress(&mut self, position: Duration, cx: &mut Context<Self>) {
        if self.ensure_track_loaded(cx).is_ok() {
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
                let should_continue = this.update(cx, |this, cx| this.update_audio_progress(cx)).unwrap_or(false);
                if !should_continue {
                    break;
                }
            }
        }));
    }

    pub(crate) fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(player) = &self.player {
            player.set_volume(self.volume);
        }
    }

    pub(crate) fn position_from_drag(&self, position: Point<Pixels>, bounds: Bounds<Pixels> ) -> Option<Duration> {
        let total = self.total_duration?;
        let left = bounds.origin.x.as_f32();
        let width = bounds.size.width.as_f32().max(1.0);
        let ratio = ((position.x.as_f32() - left) / width).clamp(0.0, 1.0);
        let seconds = total.as_secs_f32() * ratio;
        Some(Duration::from_secs_f32(seconds))
    }

    pub(crate) fn volume_from_position(&self, position: Point<Pixels>, bounds: Bounds<Pixels>) -> f32 {
        let left = bounds.origin.x.as_f32();
        let width = bounds.size.width.as_f32().max(1.0);
        ((position.x.as_f32() - left) / width).clamp(0.0, 1.0)
    }

    pub(crate) fn format_time(&self, duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    fn get_music_index(&self) -> Option<usize> {
        let current_index = if !self.current_player.music_id.is_empty() {
            self.player_list.iter().position(|music| music.music_id == self.current_player.music_id)
        } else if !self.current_player.music_source.is_empty() {
            self.player_list.iter().position(|music| music.music_source == self.current_player.music_source)
        } else {
            None
        };
        current_index
    }

    fn clean_music(&mut self) {
        self.pause();
        self.duration_task = None;
        self.current_position = Duration::from_secs(0);
        self.scrub_position = None;
        self.is_scrubbing = false;
        self.total_duration = None;
    }

    pub(crate) fn next_music(&mut self, cx: &mut Context<Self>) {
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

    pub(crate) fn prev_music(&mut self, cx: &mut Context<Self>) {
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
                        if this.load_music_source(cx).is_ok() {
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
    
}


