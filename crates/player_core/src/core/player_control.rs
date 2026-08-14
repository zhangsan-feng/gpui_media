use crate::{PlatState, PlayCore, PlayCoreMediaType};
use gpui::Context;
use std::time::Duration;

impl PlayCore {
    pub(crate) fn toggle_play(&mut self, cx: &mut Context<Self>) {
        match &self.playback.state {
            PlatState::Playing => self.pause(cx),
            PlatState::Paused => {
                if self.resume() {
                    self.start_progress_task(cx);
                    cx.notify();
                } else {
                    self.play(cx);
                }
            }
            PlatState::Loading | PlatState::Cache(_) => {
                if self.resume() {
                    cx.notify();
                }
            }
            PlatState::UnLoading | PlatState::Error(_) => self.play(cx),
        }
    }

    pub(crate) fn retry(&mut self, cx: &mut Context<Self>) {
        if self.current_player.url.is_empty() {
            return;
        }
        self.reset_pipeline();
        self.play(cx);
    }

    pub(crate) fn play(&mut self, cx: &mut Context<Self>) {
        if self.current_player.url.is_empty() {
            return;
        }

        self.playback.state = PlatState::Loading;
        cx.notify();
        if let Err(error) = self.set_pipeline() {
            self.reset_pipeline();
            self.playback.state = PlatState::Error("播放失败".to_string());
            log::debug!("{}", error.backtrace());
            cx.notify();
            return;
        }

        if self.set_playing() {
            if !self.show_frame {
                self.playback.state = PlatState::Playing;
            }
            self.start_event_bus(cx);
            if self.show_frame {
                self.start_loading_timeout_task(cx);
                self.start_frame_task(cx);
            }
            self.start_progress_task(cx);
        }
        cx.notify();
    }

    fn pause(&mut self, cx: &mut Context<Self>) {
        self.pause_pipeline();
        cx.notify();
    }

    pub(crate) fn resume(&mut self) -> bool {
        if !self.set_playing() {
            return false;
        }
        self.playback.state = PlatState::Playing;
        true
    }

    pub(crate) fn pause_pipeline(&mut self) {
        self.set_paused();
        self.playback.state = PlatState::Paused;
    }

    pub(crate) fn seek(&mut self, position: Duration) {
        self.segment_end = None;
        if self.playback.pipeline.is_some() {
            self.seek_pipeline(position);
            self.position = position;
        }
    }

    pub(crate) fn play_segment(&mut self, start: Duration, end: Duration) -> bool {
        if start >= end || !self.seek_segment_pipeline(start, end) || !self.set_playing() {
            return false;
        }
        self.segment_end = Some(end);
        self.position = start;
        self.playback.state = PlatState::Playing;
        true
    }

    pub(crate) fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        self.set_pipeline_volume(self.volume);
    }

    pub(crate) fn reset_pipeline(&mut self) {
        self.playback.invalidate_session();
        self.reset_pipeline_state();
        self.playback.state = PlatState::UnLoading;
        self.playback.bus_watch_started = false;
        self.playback.progress_task = None;
        self.playback.frame_task = None;
        self.playback.bus_watch_task = None;
        self.playback.loading_timeout_task = None;
        self.total_duration = None;
        self.position = Duration::ZERO;
        self.frame_width = 0.0;
        self.frame_height = 0.0;
        self.frame_rate = 0.0;
        self.codec = None;
        self.media_type = PlayCoreMediaType::Unknown;
        self.segment_end = None;
        self.frames.reset();
        self.is_dragging_progress_bar = false;
        self.pending_seek_position = None;
    }

    pub(crate) fn clock_to_duration(&self, clock: gstreamer::ClockTime) -> Duration {
        Duration::from_nanos(clock.nseconds())
    }

    pub(crate) fn format_time(&self, duration: Duration) -> String {
        let total_secs = duration.as_secs();
        format!("{:02}:{:02}", total_secs / 60, total_secs % 60)
    }
}
