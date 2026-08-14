use crate::{PlatState, PlayCore};
use gpui::*;
use gstreamer::prelude::*;
use std::time::Duration;

impl PlayCore {
    pub(crate) fn start_loading_timeout_task(&mut self, cx: &mut Context<Self>) {
        if self.playback.loading_timeout_task.is_some() {
            return;
        }
        let source = self.current_player.url.clone();
        let session_id = self.playback.session_id;
        if source.starts_with("file://") || !self.show_frame {
            return;
        }

        self.playback.loading_timeout_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_secs(30))
                .await;
            let _ = this.update(cx, |this, cx| {
                if !this.playback.is_current_session(session_id) {
                    return;
                }
                let still_loading = this.current_player.url == source
                    && this.playback.pipeline.is_some()
                    && this.frames.current_image().is_none();
                this.playback.loading_timeout_task = None;
                if still_loading {
                    this.reset_pipeline();
                    this.playback.state = PlatState::Error("加载视频源超时".to_string());
                    cx.notify();
                }
            });
        }));
    }

    pub(crate) fn start_progress_task(&mut self, cx: &mut Context<Self>) {
        if self.playback.progress_task.is_some() {
            return;
        }
        let session_id = self.playback.session_id;
        self.playback.progress_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(200))
                    .await;
                if !this
                    .update(cx, |this, cx| this.update_progress(session_id, cx))
                    .unwrap_or(false)
                {
                    break;
                }
            }
        }));
    }

    pub(crate) fn start_frame_task(&mut self, cx: &mut Context<Self>) {
        if self.playback.frame_task.is_some() {
            return;
        }
        let session_id = self.playback.session_id;
        self.playback.frame_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(30))
                    .await;
                if !this
                    .update(cx, |this, cx| this.update_frame(session_id, cx))
                    .unwrap_or(false)
                {
                    break;
                }
            }
        }));
    }

    fn update_progress(&mut self, session_id: u64, cx: &mut Context<Self>) -> bool {
        if !self.playback.is_current_session(session_id) {
            return false;
        }
        if let Some(pipeline) = &self.playback.pipeline {
            if let Some(position) = pipeline.query_position::<gstreamer::ClockTime>() {
                self.position = self.clock_to_duration(position);
            }
            if self
                .total_duration
                .map(|duration| duration.is_zero())
                .unwrap_or(true)
            {
                if let Some(duration) = pipeline.query_duration::<gstreamer::ClockTime>() {
                    let duration = self.clock_to_duration(duration);
                    if !duration.is_zero() {
                        self.total_duration = Some(duration);
                    }
                }
            }
        }

        let keep_running = matches!(
            self.playback.state,
            PlatState::Loading | PlatState::Playing | PlatState::Cache(_)
        ) || self.is_dragging_progress_bar;
        if !keep_running {
            self.playback.progress_task = None;
        }
        cx.notify();
        keep_running
    }

    fn update_frame(&mut self, session_id: u64, cx: &mut Context<Self>) -> bool {
        if !self.playback.is_current_session(session_id) {
            return false;
        }
        if let Some(frame) = self.frames.submit_latest_frame() {
            self.frame_aspect = (frame.width as f32 / frame.height as f32).max(0.01);
            self.frame_width = frame.width as f32;
            self.frame_height = frame.height as f32;
            self.frame_rate = frame.frame_rate;
            if matches!(
                self.playback.state,
                PlatState::Loading | PlatState::Cache(_)
            ) {
                self.playback.state = PlatState::Playing;
            }
            cx.notify();
        }

        let keep_running = self.playback.pipeline.is_some();
        if !keep_running {
            self.playback.frame_task = None;
        }
        keep_running
    }

    pub(crate) fn get_progress_position(
        &self,
        position: Point<Pixels>,
        bounds: Bounds<Pixels>,
    ) -> Option<Duration> {
        let total = self.total_duration?;
        if total.is_zero() {
            return None;
        }
        let ratio = ((position.x.as_f32() - bounds.origin.x.as_f32())
            / bounds.size.width.as_f32().max(1.0))
        .clamp(0.0, 1.0);
        Some(Duration::from_secs_f32(total.as_secs_f32() * ratio))
    }

    pub(crate) fn get_volume_position(
        &self,
        position: Point<Pixels>,
        bounds: Bounds<Pixels>,
    ) -> f32 {
        ((position.x.as_f32() - bounds.origin.x.as_f32()) / bounds.size.width.as_f32().max(1.0))
            .clamp(0.0, 1.0)
    }
}
