use crate::com::window_center_options;
use crate::drive;
use crate::drive::video_player::VideoPlayer;
use crate::state::GlobalState;
use gpui::prelude::*;
use gpui::*;
use gpui_component::Root;
use gstreamer as gst;
use gstreamer::prelude::ElementExt as GstElementExt;
use gstreamer::prelude::*;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

impl VideoPlayer {
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
        let file_path = path.to_string_lossy().into_owned();

        Some(drive::NetworkStatic {
            id: Uuid::new_v4().to_string(),
            name: file_name,
            img: String::from(""),
            author: String::from(""),
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
        self.reset_pipeline();
        self.play(cx);

        cx.notify();
    }

    fn switch_to_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.player_list.len() {
            return;
        }
        let next = self.player_list[index].clone();
        if next.source.is_empty() {
            return;
        }
        self.current_player = next;
        self.refresh(cx);
    }

    fn current_playlist_index(&self) -> Option<usize> {
        self.player_list
            .iter()
            .position(|item| item.source == self.current_player.source)
    }

    pub(crate) fn prev_video(&mut self, cx: &mut Context<Self>) {
        let len = self.player_list.len();
        if len == 0 {
            return;
        }
        let index = match self.current_playlist_index() {
            Some(i) if i > 0 => i - 1,
            Some(_) => len - 1,
            None => len - 1,
        };
        self.switch_to_index(index, cx);
    }

    pub(crate) fn next_video(&mut self, cx: &mut Context<Self>) {
        let len = self.player_list.len();
        if len == 0 {
            return;
        }
        let index = match self.current_playlist_index() {
            Some(i) if i + 1 < len => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.switch_to_index(index, cx);
    }

    pub(crate) fn refresh(&mut self, cx: &mut Context<Self>) {
        self.reset_pipeline();
        self.play(cx)
    }

    pub(crate) fn toggle_play(&mut self, cx: &mut Context<Self>) {
        if self.is_player {
            self.pause();
        } else {
            self.play(cx);
        }
    }

    pub(crate) fn play(&mut self, cx: &mut Context<Self>) {
        if self.current_player.source.is_empty() && self.player_list.is_empty() {
            return;
        }

        if self.current_player.source.is_empty() && !self.player_list.is_empty() {
            if let Some(player) = self.player_list.first() {
                self.current_player = player.clone();
            }
        };
        let player = self.current_player.clone();
        self.last_error = None;
        self.is_player = false;



        cx.spawn(async move |this, cx| {
            let res = tokio::spawn(async move { player.play(player.source.as_str()) });

            match res.await {
                Ok(val) => {
                    let _ = this.update(cx, |this, cx| {
                        this.current_player.source = val;
                        if let Err(err) = this.set_pipeline() {
                            this.reset_pipeline();
                            this.last_error = Some(format!("failed to build pipeline: {err}"));
                            cx.notify();
                            return;
                        }

                        if let Some(playbin) = &this.video_frame_pipeline {
                            let _ = playbin.set_state(gst::State::Playing);
                            this.is_player = true;
                            this.start_event_bus(cx);
                            this.start_progress_task(cx);
                            this.start_frame_task(cx);
                        }
                        cx.notify();
                    });
                }
                Err(err) => {
                    let _ = this.update(cx, |this, cx| {
                        this.reset_pipeline();
                        this.last_error = Some(format!("failed to resolve video source: {err}"));
                        cx.notify();
                    });
                }
            };
        })
        .detach();
    }

    fn pause(&mut self) {
        if let Some(playbin) = &self.video_frame_pipeline {
            let _ = playbin.set_state(gst::State::Paused);
        }
        self.is_player = false;
    }
}
