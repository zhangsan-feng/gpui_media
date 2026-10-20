use crate::{PlatState, PlayCore, PlayStatic};
use gpui_kit::{Bounds, Context, ExternalPaths, Point, Task};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::time::Duration;

impl PlayCore {
    pub fn is_playing(&self) -> bool {
        self.pipeline.state == PlatState::Playing
    }

    pub fn _drag_progress(&mut self, position: Duration, cx: &mut Context<Self>) -> bool {
        let Some(duration) = self.progress.duration else {
            return false;
        };
        self.progress.is_dragging = true;
        self.progress.pending_seek_position = Some(position.min(duration));
        cx.notify();
        true
    }

    pub fn _drag_progress_at(
        &mut self,
        position: Point<gpui_kit::Pixels>,
        bounds: Bounds<gpui_kit::Pixels>,
        cx: &mut Context<Self>,
    ) -> Option<Duration> {
        let target = self.get_progress_position(position, bounds)?;
        self._drag_progress(target, cx);
        Some(target)
    }

    pub fn _commit_progress_drag(&mut self, cx: &mut Context<Self>) -> bool {
        let target = self.progress.pending_seek_position.take();
        self.progress.is_dragging = false;
        let Some(target) = target else {
            return false;
        };
        let seeked = self.seek(target, cx);
        cx.notify();
        seeked
    }

    pub fn _drag_volume(&mut self, volume: f32, cx: &mut Context<Self>) -> f32 {
        let volume = self.set_volume(volume);
        cx.notify();
        volume
    }

    pub fn _drag_volume_at(
        &mut self,
        position: Point<gpui_kit::Pixels>,
        bounds: Bounds<gpui_kit::Pixels>,
        cx: &mut Context<Self>,
    ) -> f32 {
        let volume = self.bar_ratio(position, bounds);
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

        let created_pipeline = match self.ensure_pipeline(cx) {
            Ok(created_pipeline) => created_pipeline,
            Err(error) => {
                self.reset_pipeline();
                self.pipeline.state = PlatState::Error("分段播放失败".to_string());
                log::warn!("[gst:segment] failed to create pipeline: {error:#}");
                cx.notify();
                return false;
            }
        };

        let played = self.play_segment(start, end);
        if played {
            self.start_progress_task(cx);
            self.start_frame_task(cx);
            cx.notify();
        } else if created_pipeline {
            self.reset_pipeline();
            self.pipeline.state = PlatState::Error("分段播放失败".to_string());
            cx.notify();
        }
        played
    }

    pub fn _play_source(&mut self, player: PlayStatic, cx: &mut Context<Self>) {
        self.player_static = player;
        self.retry(cx);
        cx.notify();
    }

    pub fn _stop_and_release(&mut self, cx: &mut Context<Self>) -> Task<()> {
        let pipeline = self.pipeline.pipeline.clone();
        self.player_static = PlayStatic::default();
        self.reset_pipeline_with_release_wait(false);
        cx.notify();

        cx.background_executor().spawn(async move {
            let Some(pipeline) = pipeline else {
                return;
            };
            let set_result = pipeline.set_state(gst::State::Null);
            let timeout = gst::ClockTime::from_seconds(2);
            let (wait_result, state, pending) = pipeline.state(Some(timeout));
            if set_result.is_err() || wait_result.is_err() || state != gst::State::Null {
                log::warn!(
                    "[gst:stop] pipeline did not reach Null cleanly: state={state:?} pending={pending:?} set_state={set_result:?} wait={wait_result:?}"
                );
            } else {
                log::info!("[gst:stop] pipeline reached Null and released its source");
            }
        })
    }

    pub fn _play(&mut self, cx: &mut Context<Self>) {
        if self.player_static.url.trim().is_empty() {
            return;
        }
        self.resume_or_play(cx);
    }

    pub fn _pause(&mut self, cx: &mut Context<Self>) {
        self.pause(cx);
    }

    pub fn _toggle_play(&mut self, cx: &mut Context<Self>) {
        self.toggle_play(cx);
    }

    pub fn _handle_file_drop(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        let Some(player) = self._file_drop_source(paths) else {
            return;
        };
        self._play_source(player, cx);
    }
}
