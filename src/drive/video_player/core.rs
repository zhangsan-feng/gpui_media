use crate::drive;
use crate::drive::video_player::{FrameBuffer, VideoPlayer};
use crate::state::StateEvent::{TogglePlayVideo, UpdateVideoPlayList};
use crate::state::{GlobalState, StateEvent};
use gpui::http_client::http::header;
use gpui::*;
use gpui::{Context, RenderImage};
use gpui_component::VirtualListScrollHandle;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::prelude::{ElementExt, ElementExtManual};
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use image::{Frame, RgbaImage};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

impl VideoPlayer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let headers = header::HeaderMap::new();

        let window_id = window.window_handle().window_id();
        let _ = gst::init();
        let mut s = Self {
            current_player: drive::NetworkStatic::default(),
            player_list: Vec::from([]),
            video_request_headers: headers,
            vm_scroll_handle: VirtualListScrollHandle::new(),
            video_player_volume: 0.6,
            video_frame_pipeline: None,
            video_frame_data: None,
            video_player_duration: Duration::from_secs(0),
            is_player: false,
            video_total_duration: None,
            video_frame_size: 16.0 / 9.0,
            is_dragging_progress_bar: false,
            pending_seek_position: None,
            progress_bar_bounds: None,
            volume_bar_bounds: None,
            progress_task: None,
            frame_task: None,
            bus_watch_task: None,
            frame_buffer: Arc::new(Mutex::new(FrameBuffer::default())),
            last_rendered_frame_sequence: 0,
            render_image: None,
            // frame_thread: None,
            stop_frames: Arc::new(AtomicBool::new(false)),
            last_error: None,
            bus_watch_started: false,
            pending_drop_images: Vec::new(),
        };
        s.init_subscribe(window_id, cx);
        s
    }

    fn init_subscribe(&mut self, window_id:WindowId, cx: &mut Context<Self>) {
        let state_handler = cx.global::<GlobalState>().0.clone();
        cx.subscribe(
            &state_handler,
            move |this: &mut Self, _model, event: &StateEvent, _cx| match event {
                // ############################################################################# 跨组件传递数据
                TogglePlayVideo(event_window_id, data) => {
                    if event_window_id.as_u64() == window_id.as_u64() {
                        this.current_player = data.clone();
                    }
                }
                UpdateVideoPlayList(event_window_id, data) => {
                    if event_window_id.as_u64() == window_id.as_u64() {
                        this.player_list = data.clone();
                    }
                }
                _ => {}
                // ############################################################################# 跨组件传递数据
            },
        )
        .detach();
    }

    fn clock_to_duration(&self, clock: gst::ClockTime) -> Duration {
        Duration::from_nanos(clock.nseconds())
    }

    pub(crate) fn format_time(&self, duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub(crate) fn set_pipeline(&mut self) -> anyhow::Result<()> {
        if self.video_frame_pipeline.is_some() {
            return Ok(());
        }

        let playbin = gst::ElementFactory::make("playbin")
            .name("video-playbin")
            .build()?;
        let request_headers = self.video_request_headers.clone();
        playbin.connect("source-setup", false, move |values| {
            let Some(source) = values
                .get(1)
                .and_then(|value| value.get::<gst::Element>().ok())
            else {
                return None;
            };

            if !request_headers.is_empty() && source.find_property("extra-headers").is_some() {
                source.set_property(
                    "extra-headers",
                    VideoPlayer::build_extra_headers(&request_headers),
                );
            }
            None
        });
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", "BGRA")
            .build();
        let buffer_clone = self.frame_buffer.clone();

        let appsink = gst_app::AppSink::builder()
            .caps(&caps)
            .sync(true)
            .max_buffers(3)
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

                        let buffer_ref = match sample.buffer() {
                            Some(buffer) => buffer,
                            None => return Ok(gst::FlowSuccess::Ok),
                        };

                        let map = match buffer_ref.map_readable() {
                            Ok(map) => map,
                            Err(_) => return Ok(gst::FlowSuccess::Ok),
                        };

                        let stride = info.stride()[0] as usize;
                        let row_bytes = width * 4;
                        let data = map.as_slice();
                        if data.len() < stride * height {
                            return Ok(gst::FlowSuccess::Ok);
                        }

                        let mut out = vec![0u8; width * height * 4];
                        if stride == row_bytes {
                            // 极速内存拷贝，代替原本繁重的嵌套双重循环
                            out.copy_from_slice(&data[..row_bytes * height]);
                        } else {
                            // 如果跨步（stride）和真实宽度不匹配，则逐行对齐拷贝
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
                        target.data = out;
                        target.seq = target.seq.wrapping_add(1);

                        Ok(gst::FlowSuccess::Ok)
                    })
                    .build(),
            )
            .build();

        playbin.set_property("video-sink", &appsink);
        playbin.set_property("volume", &(self.video_player_volume as f64));
        playbin.set_property("uri", &self.current_player.source);
        playbin.set_state(gst::State::Paused)?;

        self.video_frame_data = Some(appsink);
        self.video_frame_pipeline = Some(playbin);

        Ok(())
    }

    pub(crate) fn reset_pipeline(&mut self) {
        if let Some(playbin) = &self.video_frame_pipeline {
            let _ = playbin.set_state(gst::State::Null);
        }
        self.video_frame_pipeline = None;
        self.video_frame_data = None;
        self.is_player = false;
        self.video_total_duration = None;
        self.video_player_duration = Duration::from_secs(0);
        self.is_dragging_progress_bar = false;
        self.pending_seek_position = None;
        self.last_error = None;
        self.bus_watch_started = false;
        self.progress_task = None;
        self.frame_task = None;
        self.bus_watch_task = None;
        self.last_rendered_frame_sequence = 0;
        if let Some(old) = self.render_image.take() {
            self.pending_drop_images.push(old);
        }
        {
            let mut buffer = self.frame_buffer.lock().unwrap();
            buffer.width = 0;
            buffer.height = 0;
            buffer.data.clear();
            buffer.seq = 0;
        }
        self.stop_frame_thread();
        self.stop_frames.store(false, Ordering::Relaxed);
    }

    fn build_extra_headers(headers: &header::HeaderMap) -> gst::Structure {
        let mut structure = gst::Structure::new_empty("extra-headers");
        for (name, value) in headers {
            let key = name.as_str().trim();
            if key.is_empty() {
                continue;
            }
            if let Ok(value) = value.to_str() {
                structure.set(key, value.trim());
            }
        }
        structure
    }

    pub(crate) fn set_video_request_headers(&mut self, headers: header::HeaderMap) {
        self.video_request_headers = headers;
        self.reset_pipeline();
    }

    pub(crate) fn stop_frame_thread(&mut self) {
        self.stop_frames.store(true, Ordering::Relaxed);
    }

    pub(crate) fn start_event_bus(&mut self, cx: &mut Context<Self>) {
        if self.bus_watch_started {
            return;
        }
        let Some(playbin) = self.video_frame_pipeline.clone() else {
            return;
        };
        let Some(bus) = playbin.bus() else {
            return;
        };

        self.bus_watch_started = true;
        self.bus_watch_task = Some(cx.spawn(async move |this, cx| {
            loop {
                // 监听总线消息
                cx.background_executor()
                    .timer(Duration::from_millis(1500))
                    .await;

                let mut stop_loop = false;
                while let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(0)) {
                    match msg.view() {
                        gst::MessageView::Error(err) => {
                            let _ = this.update(cx, |this, cx| {
                                this.last_error =
                                    Some(format!("{} ({:?})", err.error(), err.debug()));
                                this.is_player = false;
                                cx.notify();
                            });
                            stop_loop = true;
                            break;
                        }
                        gst::MessageView::Eos(_) => {
                            let _ = this.update(cx, |this, cx| {
                                this.next_video(cx);
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
                    .update(cx, |this, _| this.video_frame_data.is_some())
                    .unwrap_or(false);
                if !keep_running {
                    break;
                }
            }
        }));
    }

    pub(crate) fn start_progress_task(&mut self, cx: &mut Context<Self>) {
        if self.progress_task.is_some() {
            return;
        }
        self.progress_task = Some(cx.spawn(async move |this, cx| {
            loop {
                // 刷新gpui 的进度条 每秒刷新多少次
                cx.background_executor()
                    .timer(Duration::from_millis(30))
                    .await;
                let should_continue = this
                    .update(cx, |this, cx| this.update_progress(cx))
                    .unwrap_or(false);
                if !should_continue {
                    break;
                }
            }
        }));
    }

    pub(crate) fn start_frame_task(&mut self, cx: &mut Context<Self>) {
        if self.frame_task.is_some() {
            return;
        }
        let buffer = self.frame_buffer.clone();
        self.frame_task = Some(cx.spawn(async move |this, cx| {
            loop {
                //  视频刷新率 每秒刷新多少帧的图片
                cx.background_executor()
                    .timer(Duration::from_millis(30))
                    .await;
                let should_continue = this
                    .update(cx, |this, cx| this.update_frame(&buffer, cx))
                    .unwrap_or(false);
                if !should_continue {
                    break;
                }
            }
        }));
    }

    fn update_progress(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(playbin) = &self.video_frame_pipeline {
            if let Some(pos) = playbin.query_position::<gst::ClockTime>() {
                self.video_player_duration = self.clock_to_duration(pos);
                // println!("[video] pos={}ms", self.video_player_duration.as_millis());
            }
            let needs_duration = self
                .video_total_duration
                .map(|d| d.as_nanos() == 0)
                .unwrap_or(true);
            if needs_duration {
                if let Some(total) = playbin.query_duration::<gst::ClockTime>() {
                    let duration = self.clock_to_duration(total);
                    // println!("[video] duration={}ms", duration.as_millis());
                    if duration.as_nanos() > 0 {
                        self.video_total_duration = Some(duration);
                    }
                }
            }
        }

        let should_continue = self.is_player || self.is_dragging_progress_bar;
        if !should_continue {
            self.progress_task = None;
        }
        cx.notify();
        should_continue
    }

    fn update_frame(&mut self, buffer: &Arc<Mutex<FrameBuffer>>, cx: &mut Context<Self>) -> bool {
        let (seq, width, height, data) = {
            let guard = buffer.lock().unwrap();
            if guard.seq == self.last_rendered_frame_sequence {
                return self.video_frame_pipeline.is_some();
            }
            if guard.width == 0 || guard.height == 0 {
                return self.video_frame_pipeline.is_some();
            }
            (guard.seq, guard.width, guard.height, guard.data.clone())
        };

        if let Some(image) = RgbaImage::from_raw(width, height, data) {
            let frame = Frame::new(image);
            let new_image = Arc::new(RenderImage::new(vec![frame]));
            if let Some(old) = self.render_image.replace(new_image) {
                self.pending_drop_images.push(old);
            }
            self.last_rendered_frame_sequence = seq;
            self.video_frame_size = (width as f32 / height as f32).max(0.01);
            cx.notify();
        }

        let should_continue = self.video_frame_pipeline.is_some();
        if !should_continue {
            self.frame_task = None;
        }
        should_continue
    }

    pub(crate) fn free_video_frame(&mut self, window: &mut Window) {
        if self.pending_drop_images.is_empty() {
            return;
        }
        for image in self.pending_drop_images.drain(..) {
            let _ = window.drop_image(image);
        }
    }
    pub(crate) fn get_progress_position(
        &self,
        position: Point<Pixels>,
        bounds: Bounds<Pixels>,
    ) -> Option<Duration> {
        let total = self.video_total_duration?;
        if total.as_nanos() == 0 {
            return None;
        }
        let left = bounds.origin.x.as_f32();
        let width = bounds.size.width.as_f32().max(1.0);
        let ratio = ((position.x.as_f32() - left) / width).clamp(0.0, 1.0);
        let seconds = total.as_secs_f32() * ratio;
        Some(Duration::from_secs_f32(seconds))
    }

    pub(crate) fn seek_video_progress(&mut self, position: Duration) {
        if let Some(playbin) = &self.video_frame_pipeline {
            let nanos = position.as_nanos().min(u64::MAX as u128) as u64;
            let target = gst::ClockTime::from_nseconds(nanos);

            let ok = playbin.seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE, target);
            println!("[video] seek ok={:?} pos={}ms", ok, position.as_millis());
            self.video_player_duration = position;
        }
    }

    pub(crate) fn get_volume_position(
        &self,
        position: Point<Pixels>,
        bounds: Bounds<Pixels>,
    ) -> f32 {
        let left = bounds.origin.x.as_f32();
        let width = bounds.size.width.as_f32().max(1.0);
        ((position.x.as_f32() - left) / width).clamp(0.0, 1.0)
    }

    pub(crate) fn set_volume_size(&mut self, volume: f32) {
        self.video_player_volume = volume.clamp(0.0, 1.0);
        if let Some(playbin) = &self.video_frame_pipeline {
            playbin.set_property("volume", &(self.video_player_volume as f64));
        }
    }
}
