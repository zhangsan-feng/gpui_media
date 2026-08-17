use crate::{PlatState, PlayCore, PlayCoreFilterKind, PlayCoreFilterState, PlayStatic};
use gpui::http_client::Url;
use gpui::{
    App, AppContext, Bounds, Context, EntityId, ExternalPaths, IntoElement, Point, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowId, WindowOptions, px, size,
};
use gpui_component::Root;
use reqwest::header::HeaderMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct PlayCoreProgress {
    pub position: Duration,
    pub duration: Option<Duration>,
    pub ratio: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayCoreMediaType {
    #[default]
    Unknown,
    Audio,
    Video,
}

#[derive(Clone, Debug)]
pub struct PlayCoreViewState {
    pub player: PlayStatic,
    pub is_idle: bool,
    pub is_loading: bool,
    pub is_playing: bool,
    pub is_paused: bool,
    pub buffering_message: Option<String>,
    pub error_message: Option<String>,
    pub position: Duration,
    pub duration: Option<Duration>,
    pub volume: f32,
    pub frame_aspect: f32,
    pub frame_width: f32,
    pub frame_height: f32,
    pub frame_rate: f64,
    pub codec: Option<String>,
    pub media_type: PlayCoreMediaType,
    pub filters: PlayCoreFilterState,
}

impl PlayCore {
    pub fn _filters(&self) -> PlayCoreFilterState {
        self.filter_state
    }

    pub fn _drag_filter(
        &mut self,
        filter: PlayCoreFilterKind,
        value: f32,
        cx: &mut Context<Self>,
    ) -> f32 {
        let value = self.set_filter_value(filter, value);
        cx.notify();
        value
    }

    pub fn _reset_filters(&mut self, cx: &mut Context<Self>) {
        self.filter_state = PlayCoreFilterState::default();
        self.apply_filter_state();
        cx.notify();
    }

    pub fn _progress(&self) -> PlayCoreProgress {
        let duration = self.total_duration;
        let position = self
            .pending_seek_position
            .filter(|_| self.is_dragging_progress_bar)
            .unwrap_or(self.position);
        let ratio = duration
            .filter(|duration| duration.as_nanos() > 0)
            .map(|duration| (position.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0))
            .unwrap_or(0.0);

        PlayCoreProgress {
            position,
            duration,
            ratio,
        }
    }

    pub fn _drag_progress(&mut self, position: Duration, cx: &mut Context<Self>) -> bool {
        let Some(duration) = self.total_duration else {
            return false;
        };
        self.is_dragging_progress_bar = true;
        self.pending_seek_position = Some(position.min(duration));
        cx.notify();
        true
    }

    pub fn _drag_progress_at(
        &mut self,
        position: Point<gpui::Pixels>,
        bounds: Bounds<gpui::Pixels>,
        cx: &mut Context<Self>,
    ) -> Option<Duration> {
        let target = self.get_progress_position(position, bounds)?;
        self._drag_progress(target, cx);
        Some(target)
    }

    pub fn _commit_progress_drag(&mut self, cx: &mut Context<Self>) -> bool {
        let target = self.pending_seek_position.take();
        self.is_dragging_progress_bar = false;
        let Some(target) = target else {
            return false;
        };
        let seeked = self.seek(target);
        cx.notify();
        seeked
    }

    pub fn _drag_volume(&mut self, volume: f32, cx: &mut Context<Self>) -> f32 {
        let volume = volume.clamp(0.0, 1.0);
        self.set_volume(volume);
        cx.notify();
        volume
    }

    pub fn _drag_volume_at(
        &mut self,
        position: Point<gpui::Pixels>,
        bounds: Bounds<gpui::Pixels>,
        cx: &mut Context<Self>,
    ) -> f32 {
        let volume = self.get_volume_position(position, bounds);
        self._drag_volume(volume, cx)
    }

    pub fn _play_segment(
        &mut self,
        start: Duration,
        end: Duration,
        cx: &mut Context<Self>,
    ) -> bool {
        if start >= end {
            return false;
        }

        if self.playback.pipeline.is_none() {
            self.play(cx);
        }

        let played = self.play_segment(start, end);
        if played {
            self.start_progress_task(cx);
            cx.notify();
        }
        played
    }

    pub fn _view_state(&self) -> PlayCoreViewState {
        let (is_idle, is_loading, is_playing, is_paused, buffering_message, error_message) =
            match &self.playback.state {
                PlatState::UnLoading => (true, false, false, false, None, None),
                PlatState::Loading => (false, true, false, false, None, None),
                PlatState::Playing => (false, false, true, false, None, None),
                PlatState::Paused => (false, false, false, true, None, None),
                PlatState::Cache(message) => {
                    (false, false, false, false, Some(message.clone()), None)
                }
                PlatState::Error(message) => {
                    (false, false, false, false, None, Some(message.clone()))
                }
            };

        PlayCoreViewState {
            player: self.current_player.clone(),
            is_idle,
            is_loading,
            is_playing,
            is_paused,
            buffering_message,
            error_message,
            position: self.position,
            duration: self.total_duration,
            volume: self.volume,
            frame_aspect: self.frame_aspect,
            frame_width: self.frame_width,
            frame_height: self.frame_height,
            frame_rate: self.frame_rate,
            codec: self.codec.clone(),
            media_type: self.media_type,
            filters: self.filter_state,
        }
    }

    pub fn _play_source(&mut self, player: PlayStatic, cx: &mut Context<Self>) {
        self.current_player = player;
        self.retry(cx);
        cx.notify();
    }

    pub fn _play(&mut self, cx: &mut Context<Self>) {
        self.play(cx);
    }

    pub fn _pause(&mut self, cx: &mut Context<Self>) {
        self.pause_pipeline();
        cx.notify();
    }

    pub fn _toggle_play(&mut self, cx: &mut Context<Self>) {
        self.toggle_play(cx);
    }

    pub fn _frame_ui(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.frames.drain_retired_images(window);
        self.render_frame(cx)
    }

    pub fn _progress_ui(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.player_progress_control_ui(window, cx)
    }

    pub fn _volume_ui(&self, cx: &mut Context<Self>) -> impl IntoElement {
        self.player_volume_control_ui(cx)
    }

    pub fn _duration_ui(&self) -> impl IntoElement {
        let total = self.total_duration.unwrap_or(Duration::ZERO);
        let position = self
            .pending_seek_position
            .filter(|_| self.is_dragging_progress_bar)
            .unwrap_or(self.position);
        self.player_duration_display_ui(position, total)
    }

    pub fn _file_drop_source(&self, paths: &ExternalPaths) -> Option<PlayStatic> {
        let Some(path) = paths.paths().iter().find(|path| path.is_file()) else {
            return None;
        };
        let Some(url) = Url::from_file_path(Path::new(path))
            .ok()
            .map(|url| url.to_string())
        else {
            return None;
        };

        Some(PlayStatic {
            id: Uuid::new_v4().to_string(),
            title: Path::new(path)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "本地媒体".to_string()),
            url,
            headers: HeaderMap::new(),
        })
    }

    pub fn _handle_file_drop(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        let Some(player) = self._file_drop_source(paths) else {
            return;
        };
        self._play_source(player, cx);
    }

    pub fn _open_window(window: &mut Window, cx: &mut App, title: &str) -> (WindowId, EntityId) {
        let player_entity_id = Arc::new(Mutex::new(None));
        let player_entity_id_for_window = player_entity_id.clone();
        let options = window_center_settings(window, 1400., 800., title);
        let handler = cx
            .open_window(options, move |window, app| {
                let view = app.new(|cx| PlayCore::new(window, cx));
                *player_entity_id_for_window.lock().unwrap() = Some(view.entity_id());
                app.new(|cx| Root::new(view, window, cx))
            })
            .expect("open window failed");
        let player_entity_id = player_entity_id
            .lock()
            .unwrap()
            .expect("video player entity was not created");
        (handler.window_id(), player_entity_id)
    }
}

fn window_center_settings(window: &mut Window, w: f32, h: f32, title: &str) -> WindowOptions {
    let parent_bounds = window.bounds();
    let window_size = size(px(w), px(h));
    let bounds = Bounds {
        origin: Point {
            x: parent_bounds.origin.x + (parent_bounds.size.width - px(w)) / 2.0,
            y: parent_bounds.origin.y + (parent_bounds.size.height - px(h)) / 2.0,
        },
        size: window_size,
    };
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(window_size),
        is_resizable: true,
        titlebar: Some(TitlebarOptions {
            title: Some(SharedString::from(title.to_string())),
            appears_transparent: false,
            ..Default::default()
        }),
        ..Default::default()
    }
}
