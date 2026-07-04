use crate::com::window_center_options;
use crate::drive;
use crate::drive::music_player::MusicPlayer;
use crate::drive::video_player::VideoPlayer;
use crate::state::StateEvent::UpdateMusicPlatyList;
use crate::state::{GlobalState, StateEvent};
use anyhow::anyhow;
use gpui::prelude::*;
use gpui::*;
use gpui_component::Root;
use gstreamer as gst;
use gstreamer::prelude::ElementExt as GstElementExt;
use gstreamer::prelude::*;
use log::error;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

impl MusicPlayer {
    fn open_window(&self, window: &mut Window, cx: &mut Context<Self>) {
        cx.open_window(
            window_center_options(window, 1300., 700.),
            move |window, app| {
                let view = app.new(|cx| VideoPlayer::new(window, cx));
                app.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("open window failed");
    }

    fn handler_local_file(&self, path: &Path) -> Option<drive::NetworkStatic> {
        if !path.is_file() {
            return None;
        }

        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let file_path = path.to_string_lossy().to_string();

        Some(drive::NetworkStatic {
            id: Uuid::new_v4().to_string(),
            name: file_name.to_string(),
            img: "".to_string(),
            author: "".to_string(),
            category: String::new(),
            headers: Default::default(),
            source: file_path,
            func: Arc::new(drive::LocalStatic),
        })
    }

    pub(crate) fn handle_file_drop(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        let mut added = Vec::new();
        for path in paths.paths() {
            let Some(track) = self.handler_local_file(path) else {
                continue;
            };
            if self
                .player_list
                .iter()
                .any(|item| item.source == track.source)
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
        self.play_current_music(cx);
        cx.notify();
    }

    pub(crate) fn get_progress_position(
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

    pub(crate) fn play_current_music(&mut self, cx: &mut Context<Self>) {
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

        self.current_player.source = self
            .current_player
            .play(self.current_player.source.as_str());

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

    pub(crate) fn pause(&mut self) {
        if let Some(playbin) = &self.audio_pipeline {
            let _ = playbin.set_state(gst::State::Paused);
            if let Some(pos) = playbin.query_position::<gst::ClockTime>() {
                self.current_position = Duration::from_nanos(pos.nseconds());
            }
        }
        self.is_player = false;
    }

    pub(crate) fn get_music_index(&self) -> Option<usize> {
        let current_index = if !self.current_player.id.is_empty() {
            self.player_list
                .iter()
                .position(|music| music.id == self.current_player.id)
        } else if !self.current_player.source.is_empty() {
            self.player_list
                .iter()
                .position(|music| music.source == self.current_player.source)
        } else {
            None
        };
        current_index
    }
}
