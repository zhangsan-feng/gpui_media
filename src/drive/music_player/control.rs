
use crate::entity;
use crate::entity::NetworkStatic;
use crate::state::StateEvent::{TogglePlayMusic, UpdateMusicPlatyList};
use crate::state::{GlobalState, StateEvent};
use anyhow::{Result, anyhow};
use gpui::prelude::*;
use gpui::*;
use gpui_component::*;
use log::{error, info};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use url::Url;
use uuid::Uuid;
use crate::drive::music_player::MusicPlayer;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::prelude::ElementExt as GstElementExt;

impl MusicPlayer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> MusicPlayer {
        let _ = window;
        let mut s = MusicPlayer {
            current_player: entity::NetworkStatic::default(),
            player_list: vec![],
            vm_scroll_handle: VirtualListScrollHandle::new(),
            is_player: false,
            play_err: None,
            audio_pipeline: None,
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
        let state_handler = cx.global::<GlobalState>().0.clone();

        cx.subscribe(
            &state_handler,
            |this: &mut Self, _model, event: &StateEvent, cx| match event {
                TogglePlayMusic(data) => {
                    // println!("{:?}", data.music_source);
                    this.current_player = data.clone();
                    this.play_current_music(cx);
                    cx.notify();
                }
                UpdateMusicPlatyList(data) => {
                    this.player_list = data.clone();
                }
                _ => {}
            },
        )
            .detach();
    }

    pub(crate) fn format_time(&self, duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
    
    fn set_pipeline(&mut self, _cx: &mut Context<Self>) -> Result<()> {
        let _ = gst::init();
        if let Some(playbin) = &self.audio_pipeline {
            let _ = playbin.set_state(gst::State::Null);
        }
        self.audio_pipeline = None;

        let source = if Path::new(&self.current_player.source).is_file() {
            let canonical = Path::new(&self.current_player.source).canonicalize()?;
            Url::from_file_path(canonical)
                .map_err(|_| anyhow!("invalid local file path"))?
                .to_string()
        } else {
            self.current_player.source.clone()
        };

        let playbin = gst::ElementFactory::make("playbin")
            .name("music-playbin")
            .build()?;
        playbin.set_property("uri", &source);
        playbin.set_property("volume", &(self.volume as f64));
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

        self.total_duration = total_duration.or_else(|| {
            playbin
                .query_duration::<gst::ClockTime>()
                .map(|d| Duration::from_nanos(d.nseconds()))
        });
        self.audio_pipeline = Some(playbin);
        Ok(())
    }

    // stop_pipeline clean_music 合并到 reset_pipeline
    fn reset_pipeline(&mut self, _cx: &mut Context<Self>) -> Result<()> {
        self.pause();
        if let Some(playbin) = &self.audio_pipeline {
            let _ = playbin.set_state(gst::State::Null);
        }
        self.audio_pipeline = None;
        self.duration_task = None;
        self.current_position = Duration::from_secs(0);
        self.scrub_position = None;
        self.is_scrubbing = false;
        self.total_duration = None;
        Ok(())
    }


    fn handler_local_file(&self, path: &Path) -> Option<entity::NetworkStatic> {
        if !path.is_file() {
            return None;
        }

        let file_name = path.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string_lossy().to_string());
        let file_path = path.to_string_lossy().to_string();

        Some(entity::NetworkStatic {
            id: Uuid::new_v4().to_string(),
            name: file_name.to_string(),
            img: "".to_string(),
            author: "".to_string(),
            headers: Default::default(),
            source:file_path,
            func: Arc::new(entity::LocalStatic),
        })
    }

    pub(crate) fn handle_file_drop(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        let mut added = Vec::new();
        for path in paths.paths() {
            let Some(track) = self.handler_local_file(path) else {
                continue;
            };
            if self.player_list.iter().any(|item| item.source == track.source){
                continue;
            }
            added.push(track);
        }

        if added.is_empty() {
            return;
        }

        self.current_player = added[0].clone();
        self.player_list.extend(added);
        self.play_current_music(cx);
        cx.notify();
    }

    pub(crate) fn seek_audio_progress(&mut self, position: Duration, cx: &mut Context<Self>) {
        if self.audio_pipeline.is_none() {
            if let Err(e) = self.set_pipeline(cx) {
                self.play_err = Some(e.to_string());
                error!("set_pipeline failed in seek_audio_progress: {}", e);
                return;
            }
        }
        if let Some(playbin) = &self.audio_pipeline {
            let nanos = position.as_nanos().min(u64::MAX as u128) as u64;
            let target = gst::ClockTime::from_nseconds(nanos);
            let _ = playbin.seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE, target);
            self.current_position = position;
            if self.is_player {
                let _ = playbin.set_state(gst::State::Playing);
                self.start_progress_task(cx);
            } else {
                let _ = playbin.set_state(gst::State::Paused);
            }
        }
    }

    fn update_audio_progress(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(playbin) = self.audio_pipeline.clone() {
            if let Some(pos) = playbin.query_position::<gst::ClockTime>() {
                self.current_position = Duration::from_nanos(pos.nseconds());
            }
            if self.total_duration.is_none() {
                if let Some(total) = playbin.query_duration::<gst::ClockTime>() {
                    self.total_duration = Some(Duration::from_nanos(total.nseconds()));
                }
            }
            if let Some(bus) = playbin.bus() {
                while let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(0)) {
                    match msg.view() {
                        gst::MessageView::Eos(_) => {
                            self.is_player = false;
                            self.next_music(cx);
                            break;
                        }
                        gst::MessageView::Error(err) => {
                            self.is_player = false;
                            self.play_err = Some(err.error().to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        let should_continue = self.is_player || self.is_scrubbing;
        if !should_continue {
            self.progress_task = None;
        }
        cx.notify();
        should_continue
    }

    fn start_progress_task(&mut self, cx: &mut Context<Self>) {
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

    pub(crate) fn set_volume_size(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(playbin) = &self.audio_pipeline {
            playbin.set_property("volume", &(self.volume as f64));
        }
    }

    pub(crate) fn get_progress_position(&self, position: Point<Pixels>, bounds: Bounds<Pixels> ) -> Option<Duration> {
        let total = self.total_duration?;
        let left = bounds.origin.x.as_f32();
        let width = bounds.size.width.as_f32().max(1.0);
        let ratio = ((position.x.as_f32() - left) / width).clamp(0.0, 1.0);
        let seconds = total.as_secs_f32() * ratio;
        Some(Duration::from_secs_f32(seconds))
    }

    pub(crate) fn get_volume_position(&self, position: Point<Pixels>, bounds: Bounds<Pixels>) -> f32 {
        let left = bounds.origin.x.as_f32();
        let width = bounds.size.width.as_f32().max(1.0);
        ((position.x.as_f32() - left) / width).clamp(0.0, 1.0)
    }

    fn get_music_index(&self) -> Option<usize> {
        let current_index = if !self.current_player.id.is_empty() {
            self.player_list.iter().position(|music| music.id == self.current_player.id)
        } else if !self.current_player.source.is_empty() {
            self.player_list.iter().position(|music| music.source == self.current_player.source)
        } else {
            None
        };
        current_index
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

        if let Err(e) = self.reset_pipeline(cx) {
            self.play_err = Some(e.to_string());
            error!("reset_pipeline failed in next_music: {}", e);
        }
        self.current_player = self.player_list[next_index].clone();
        self.play_current_music(cx);
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

        if let Err(e) = self.reset_pipeline(cx) {
            self.play_err = Some(e.to_string());
            error!("reset_pipeline failed in prev_music: {}", e);
        }
        self.current_player = self.player_list[prev_index].clone();
        self.play_current_music(cx);
    }

    fn play_current_music(&mut self, cx: &mut Context<Self>) {
        if let Err(e) = self.reset_pipeline(cx) {
            self.play_err = Some(e.to_string());
            error!("reset_pipeline failed in play_current_music: {}", e);
            return;
        }
        if let Err(e) = self.set_pipeline(cx) {
            self.play_err = Some(e.to_string());
            error!("set_pipeline failed in play_current_music: {}", e);
            return;
        }
        if let Some(playbin) = &self.audio_pipeline {
            let _ = playbin.set_state(gst::State::Playing);
            self.is_player = true;
            self.start_progress_task(cx);
        }
    }

    pub(crate) fn toggle_play(&mut self, cx: &mut Context<Self>) {
        if self.is_player {
            self.pause();
        } else {
            self.play(cx);
        }
    }

    fn play(&mut self, cx: &mut Context<Self>) {

        if self.current_player.source.is_empty() && !self.player_list.is_empty() {
            if let Some(player) = self.player_list.first() {
                self.current_player = player.clone();
                self.play_current_music(cx);
                return;
            }
        }

        self.current_player.source = self.current_player.play(self.current_player.source.as_str());

        if self.audio_pipeline.is_none() {
            if let Err(e) = self.set_pipeline(cx) {
                self.play_err = Some(e.to_string());
                error!("set_pipeline failed in play: {}", e);
                return;
            }
        }

        if let Some(playbin) = &self.audio_pipeline {
            let _ = playbin.set_state(gst::State::Playing);
            self.is_player = true;
            self.start_progress_task(cx);
        }
    }

    fn pause(&mut self) {
        if let Some(playbin) = &self.audio_pipeline {
            let _ = playbin.set_state(gst::State::Paused);
            if let Some(pos) = playbin.query_position::<gst::ClockTime>() {
                self.current_position = Duration::from_nanos(pos.nseconds());
            }
        }
        self.is_player = false;
    }


}
