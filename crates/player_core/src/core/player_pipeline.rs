use crate::{PlayCore, PlayCoreMediaType};
use gpui::Context;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;

impl Drop for PlayCore {
    fn drop(&mut self) {
        if let Some(pipeline) = &self.playback.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
    }
}

impl PlayCore {
    pub(crate) fn set_pipeline(&mut self, cx: &mut Context<Self>) -> anyhow::Result<()> {
        if self.playback.pipeline.is_some() {
            return Ok(());
        }

        let playbin = gst::ElementFactory::make("playbin3")
            .name("video-playbin")
            .build()?;
        let mut headers = self.current_player.headers.clone();
        if !headers.contains_key(reqwest::header::USER_AGENT) {
            headers.insert(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0.0.0 Safari/537.36",
                ),
            );
        }
        log::info!(
            "[gst:play] uri={} headers={:?}",
            self.current_player.url,
            headers.keys().map(|name| name.as_str()).collect::<Vec<_>>()
        );
        playbin.connect("source-setup", false, move |values| {
            let Some(source) = values
                .get(1)
                .and_then(|value| value.get::<gst::Element>().ok())
            else {
                return None;
            };

            if source.find_property("extra-headers").is_none() {
                return None;
            }

            let mut extra_headers = gst::Structure::builder("extra-headers");
            let mut header_count = 0;
            let mut applied_headers = Vec::new();
            for (name, value) in &headers {
                let Ok(value) = value.to_str() else {
                    continue;
                };
                if name == reqwest::header::USER_AGENT {
                    if source.find_property("user-agent").is_some() {
                        source.set_property("user-agent", value);
                        applied_headers.push(name.as_str());
                    }
                    continue;
                }
                extra_headers = extra_headers.field(name.as_str(), value.to_owned());
                header_count += 1;
                applied_headers.push(name.as_str());
            }

            if header_count > 0 {
                source.set_property("extra-headers", extra_headers.build());
            }
            log::info!(
                "[gst:source-setup] source={} headers={:?}",
                source.name(),
                applied_headers
            );
            None
        });

        let caps = gst::Caps::builder("video/x-raw")
            .field("format", "BGRA")
            .build();
        let buffer_clone = self.frames.latest_frame();
        let appsink = gst_app::AppSink::builder()
            .caps(&caps)
            .sync(true)
            .max_buffers(8)
            .drop(true)
            .callbacks(
                gst_app::AppSinkCallbacks::builder()
                    .new_sample(move |appsink| {
                        let sample = match appsink.pull_sample() {
                            Ok(sample) => sample,
                            Err(_) => return Ok(gst::FlowSuccess::Ok),
                        };
                        let caps = match sample.caps() {
                            Some(caps) => caps,
                            None => return Ok(gst::FlowSuccess::Ok),
                        };
                        let info = match gst_video::VideoInfo::from_caps(&caps) {
                            Ok(info) => info,
                            Err(_) => return Ok(gst::FlowSuccess::Ok),
                        };
                        let width = info.width() as usize;
                        let height = info.height() as usize;
                        if width == 0 || height == 0 {
                            return Ok(gst::FlowSuccess::Ok);
                        }
                        let fps = info.fps();
                        let frame_rate = if fps.denom() > 0 {
                            fps.numer() as f64 / fps.denom() as f64
                        } else {
                            0.0
                        };
                        let Some(buffer) = sample.buffer() else {
                            return Ok(gst::FlowSuccess::Ok);
                        };
                        let Ok(map) = buffer.map_readable() else {
                            return Ok(gst::FlowSuccess::Ok);
                        };
                        let stride = info.stride()[0] as usize;
                        let row_bytes = width * 4;
                        let data = map.as_slice();
                        if data.len() < stride * height {
                            return Ok(gst::FlowSuccess::Ok);
                        }

                        let mut out = vec![0u8; width * height * 4];
                        if stride == row_bytes {
                            out.copy_from_slice(&data[..row_bytes * height]);
                        } else {
                            for y in 0..height {
                                let src_start = y * stride;
                                let dst_start = y * row_bytes;
                                out[dst_start..dst_start + row_bytes]
                                    .copy_from_slice(&data[src_start..src_start + row_bytes]);
                            }
                        }

                        let mut target = buffer_clone.lock().unwrap();
                        target.width = width as u32;
                        target.height = height as u32;
                        target.frame_rate = frame_rate;
                        target.data = out;
                        target.seq = target.seq.wrapping_add(1);
                        Ok(gst::FlowSuccess::Ok)
                    })
                    .build(),
            )
            .build();

        if let Some(video_filter) = self.build_video_filter() {
            playbin.set_property("video-filter", &video_filter);
            self.playback.video_filter = Some(video_filter);
        }
        playbin.set_property("video-sink", &appsink);
        playbin.set_property("volume", &(self.volume as f64));
        playbin.set_property("uri", &self.current_player.url);
        playbin.set_state(gst::State::Paused)?;

        self.playback.pipeline = Some(playbin);
        self.start_event_bus(cx);
        self.start_loading_timeout_task(cx);
        Ok(())
    }

    pub(crate) fn set_playing(&self) -> bool {
        self.playback
            .pipeline
            .as_ref()
            .map(|pipeline| pipeline.set_state(gst::State::Playing).is_ok())
            .unwrap_or(false)
    }

    pub(crate) fn set_paused(&mut self) {
        if let Some(pipeline) = &self.playback.pipeline {
            let _ = pipeline.set_state(gst::State::Paused);
        }
        self.playback.loading_timeout_task = None;
    }

    pub(crate) fn maintain_pipeline_tasks(&mut self, cx: &mut Context<Self>) {
        self.start_loading_timeout_task(cx);
        if self.media_type == PlayCoreMediaType::Video {
            self.start_frame_task(cx);
        }
    }

    pub(crate) fn mark_loading_progress(&mut self) {
        self.playback.loading_progress = self.playback.loading_progress.wrapping_add(1);
    }

    pub(crate) fn mark_buffering_progress(&mut self, percent: i32) {
        if self.playback.last_buffering_percent == Some(percent) {
            return;
        }
        self.playback.last_buffering_percent = Some(percent);
        self.mark_loading_progress();
    }

    pub(crate) fn reset_pipeline_state(&mut self) {
        if let Some(pipeline) = &self.playback.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
        self.playback.pipeline = None;
        self.playback.video_filter = None;
    }

    pub(crate) fn seek_pipeline(&mut self, position: std::time::Duration) -> bool {
        let Some(pipeline) = &self.playback.pipeline else {
            return false;
        };
        let target =
            gst::ClockTime::from_nseconds(position.as_nanos().min(u64::MAX as u128) as u64);
        let seeked = pipeline
            .seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE, target)
            .is_ok();
        seeked
    }

    pub(crate) fn seek_segment_pipeline(
        &self,
        start: std::time::Duration,
        end: std::time::Duration,
    ) -> bool {
        let Some(pipeline) = &self.playback.pipeline else {
            return false;
        };
        let start = gst::ClockTime::from_nseconds(start.as_nanos().min(u64::MAX as u128) as u64);
        let end = gst::ClockTime::from_nseconds(end.as_nanos().min(u64::MAX as u128) as u64);
        pipeline
            .seek(
                1.0,
                gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE | gst::SeekFlags::SEGMENT,
                gst::SeekType::Set,
                start,
                gst::SeekType::Set,
                end,
            )
            .is_ok()
    }

    pub(crate) fn set_pipeline_volume(&self, volume: f32) {
        if let Some(pipeline) = &self.playback.pipeline {
            pipeline.set_property("volume", &(volume.clamp(0.0, 1.0) as f64));
        }
    }
}
