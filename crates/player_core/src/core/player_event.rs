use crate::state::{PlayCoreGlobalState, PlayCoreStateEvent};
use crate::{PlatState, PlayCore};
use gpui::*;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::time::Duration;

impl PlayCore {
    pub(crate) fn start_event_bus(&mut self, cx: &mut Context<Self>) {
        if self.playback.bus_watch_started {
            return;
        }
        let Some(pipeline) = self.playback.pipeline.clone() else {
            return;
        };
        let Some(bus) = pipeline.bus() else {
            return;
        };

        let is_local_file = self.current_player.url.starts_with("file://");
        let session_id = self.playback.session_id;
        self.playback.bus_watch_started = true;
        self.playback.bus_watch_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;

                let current = this
                    .update(cx, |this, _| this.playback.is_current_session(session_id))
                    .unwrap_or(false);
                if !current {
                    break;
                }

                let mut stop_loop = false;
                while let Some(message) = bus.timed_pop(gst::ClockTime::from_mseconds(0)) {
                    match message.view() {
                        gst::MessageView::Error(error) => {
                            log::info!(
                                "[gst:error] source={} error={} debug={:?}",
                                message
                                    .src()
                                    .map(|src| src.path_string())
                                    .unwrap_or_else(|| "unknown".into()),
                                error.error(),
                                error.debug()
                            );
                            let _ = this.update(cx, |this, cx| {
                                if !this.playback.is_current_session(session_id) {
                                    return;
                                }
                                this.reset_pipeline();
                                this.playback.state = PlatState::Error("播放失败".to_string());
                                cx.notify();
                            });
                            stop_loop = true;
                            break;
                        }
                        gst::MessageView::Warning(warning) => {
                            log::info!(
                                "[gst:warning] source={} warning={} debug={:?}",
                                message
                                    .src()
                                    .map(|src| src.path_string())
                                    .unwrap_or_else(|| "unknown".into()),
                                warning.error(),
                                warning.debug()
                            );
                        }
                        gst::MessageView::Tag(tag) => {
                            let tags = tag.tags();
                            let codec = tags
                                .get::<gst::tags::VideoCodec>()
                                .map(|value| value.get().to_string())
                                .or_else(|| {
                                    tags.get::<gst::tags::Codec>()
                                        .map(|value| value.get().to_string())
                                });
                            let Some(codec) = codec.filter(|codec| !codec.is_empty()) else {
                                continue;
                            };
                            let _ = this.update(cx, |this, cx| {
                                if this.playback.is_current_session(session_id) {
                                    this.codec = Some(codec);
                                    cx.notify();
                                }
                            });
                        }
                        gst::MessageView::SegmentDone(_) => {
                            let _ = pipeline.set_state(gst::State::Paused);
                            let _ = this.update(cx, |this, cx| {
                                if !this.playback.is_current_session(session_id) {
                                    return;
                                }
                                if let Some(end) = this.segment_end.take() {
                                    this.position = end;
                                }
                                this.playback.state = PlatState::Paused;
                                cx.notify();
                            });
                        }
                        gst::MessageView::Buffering(buffering) if !is_local_file => {
                            let percent = buffering.percent();
                            if percent < 100 {
                                let _ = pipeline.set_state(gst::State::Paused);
                                let _ = this.update(cx, |this, cx| {
                                    if this.playback.is_current_session(session_id) {
                                        this.playback.state =
                                            PlatState::Cache(format!("缓冲中 {percent}%"));
                                        cx.notify();
                                    }
                                });
                            } else {
                                let _ = pipeline.set_state(gst::State::Playing);
                                let _ = this.update(cx, |this, cx| {
                                    if this.playback.is_current_session(session_id) {
                                        this.playback.state = if this.show_frame {
                                            PlatState::Loading
                                        } else {
                                            PlatState::Playing
                                        };
                                        cx.notify();
                                    }
                                });
                            }
                        }
                        gst::MessageView::Latency(_) => {
                            if let Ok(bin) = pipeline.clone().dynamic_cast::<gst::Bin>() {
                                let _ = bin.recalculate_latency();
                            }
                        }
                        gst::MessageView::Eos(_) => {
                            let _ = this.update(cx, |this, cx| {
                                if !this.playback.is_current_session(session_id) {
                                    return;
                                }
                                PlayCoreGlobalState::publish(
                                    cx,
                                    PlayCoreStateEvent::PlayBackFished(
                                        this.window_id,
                                        cx.entity_id(),
                                        this.current_player.clone(),
                                    ),
                                );
                                this.reset_pipeline();
                                this.playback.state = PlatState::Paused;
                                cx.notify();
                            });
                            stop_loop = true;
                            break;
                        }
                        _ => {}
                    }
                }

                if stop_loop {
                    break;
                }
                let keep_running = this
                    .update(cx, |this, _| {
                        this.playback.is_current_session(session_id)
                            && this.playback.video_sink.is_some()
                    })
                    .unwrap_or(false);
                if !keep_running {
                    break;
                }
            }
        }));
    }
}
