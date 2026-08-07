use crate::component::window::window_center_settings;
use crate::drive;
use crate::drive::video_player::VideoPlayer;
use crate::state::StateEvent::{TogglePlayVideo, UpdateVideoPlayList};
use crate::state::{GlobalState, StateEvent};
use gpui::Context;
use gpui::http_client::http::header;
use gpui::*;
use gpui_component::input::InputState;
use gpui_component::{Root, VirtualListScrollHandle};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::prelude::{ElementExt, ElementExtManual};
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use image::{Frame, RgbaImage};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Copy)]
pub struct ProgressDrag;

#[derive(Clone, Copy)]
pub struct VolumeDrag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PlatState {
    UnLoading,
    Loading,
    Playing,
    Paused,
    Cache(String),
    Error(String),
}

pub(crate) struct PlaybackRuntime {
    pub(crate) session_id: u64,
    pub(crate) state: PlatState,
    pub(crate) pipeline: Option<gst::Element>,
    pub(crate) video_sink: Option<gst_app::AppSink>,
    pub(crate) progress_task: Option<Task<()>>,
    pub(crate) frame_task: Option<Task<()>>,
    pub(crate) bus_watch_task: Option<Task<()>>,
    pub(crate) loading_timeout_task: Option<Task<()>>,
    pub(crate) bus_watch_started: bool,
}

impl Default for PlaybackRuntime {
    fn default() -> Self {
        Self {
            session_id: 0,
            state: PlatState::UnLoading,
            pipeline: None,
            video_sink: None,
            progress_task: None,
            frame_task: None,
            bus_watch_task: None,
            loading_timeout_task: None,
            bus_watch_started: false,
        }
    }
}

impl PlaybackRuntime {
    pub(crate) fn invalidate_session(&mut self) -> u64 {
        self.session_id = self.session_id.wrapping_add(1);
        self.session_id
    }

    pub(crate) fn is_current_session(&self, session_id: u64) -> bool {
        self.session_id == session_id
    }
}

#[derive(Default)]
struct FrameBuffer {
    width: u32,
    height: u32,
    frame_rate: f64,
    data: Vec<u8>,
    seq: u64,
}

struct PresentedFrame {
    width: u32,
    height: u32,
    frame_rate: f64,
}

pub(crate) struct FramePipeline {
    latest_frame: Arc<Mutex<FrameBuffer>>,
    last_presented_sequence: u64,
    current_image: Option<Arc<RenderImage>>,
    retired_images: Vec<Arc<RenderImage>>,
}

impl Default for FramePipeline {
    fn default() -> Self {
        Self {
            latest_frame: Arc::new(Mutex::new(FrameBuffer::default())),
            last_presented_sequence: 0,
            current_image: None,
            retired_images: Vec::new(),
        }
    }
}

impl FramePipeline {
    fn latest_frame(&self) -> Arc<Mutex<FrameBuffer>> {
        self.latest_frame.clone()
    }

    pub(crate) fn current_image(&self) -> Option<Arc<RenderImage>> {
        self.current_image.clone()
    }

    pub(crate) fn reset(&mut self) {
        if let Some(image) = self.current_image.take() {
            self.retired_images.push(image);
        }
        self.latest_frame = Arc::new(Mutex::new(FrameBuffer::default()));
        self.last_presented_sequence = 0;
    }

    fn submit_latest_frame(&mut self) -> Option<PresentedFrame> {
        if !self.retired_images.is_empty() {
            return None;
        }

        let (seq, width, height, frame_rate, data) = {
            let frame = self.latest_frame.lock().unwrap();
            if frame.seq == self.last_presented_sequence || frame.width == 0 || frame.height == 0 {
                return None;
            }
            (
                frame.seq,
                frame.width,
                frame.height,
                frame.frame_rate,
                frame.data.clone(),
            )
        };

        let image = RgbaImage::from_raw(width, height, data)?;
        let image = Arc::new(RenderImage::new(vec![Frame::new(image)]));
        if let Some(old) = self.current_image.replace(image) {
            self.retired_images.push(old);
        }
        self.last_presented_sequence = seq;

        Some(PresentedFrame {
            width,
            height,
            frame_rate,
        })
    }

    pub(crate) fn drain_retired_images(&mut self, window: &mut Window) {
        for image in self.retired_images.drain(..) {
            let _ = window.drop_image(image);
        }
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        if let Some(playbin) = &self.playback.pipeline {
            let _ = playbin.set_state(gst::State::Null);
        }
    }
}

impl VideoPlayer {
    pub(crate) fn resume(&mut self) -> bool {
        let Some(playbin) = &self.playback.pipeline else {
            return false;
        };
        let _ = playbin.set_state(gst::State::Playing);
        self.playback.state = PlatState::Playing;
        true
    }

    pub(crate) fn pause_pipeline(&mut self) {
        if let Some(playbin) = &self.playback.pipeline {
            let _ = playbin.set_state(gst::State::Paused);
        }
        self.playback.state = PlatState::Paused;
    }

    pub(crate) fn seek(&mut self, position: Duration) {
        if let Some(playbin) = &self.playback.pipeline {
            let nanos = position.as_nanos().min(u64::MAX as u128) as u64;
            let target = gst::ClockTime::from_nseconds(nanos);
            let _ = playbin.seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE, target);
            self.position = position;
        }
    }

    pub(crate) fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(playbin) = &self.playback.pipeline {
            playbin.set_property("volume", &(self.volume as f64));
        }
    }

    fn reset_playback(&mut self) {
        self.playback.invalidate_session();
        if let Some(playbin) = &self.playback.pipeline {
            let _ = playbin.set_state(gst::State::Null);
        }
        self.playback.pipeline = None;
        self.playback.video_sink = None;
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
        self.frames.reset();
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
        if self.playback.pipeline.is_some() {
            return Ok(());
        }

        let playbin = gst::ElementFactory::make("playbin3")
            .name("video-playbin")
            .build()?;
        let request_headers = self.current_player.headers.clone();
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
                        let fps = info.fps();
                        let frame_rate = if fps.denom() > 0 {
                            fps.numer() as f64 / fps.denom() as f64
                        } else {
                            0.0
                        };

                        // println!("video width: {}, height: {}", width, height);
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
                        target.frame_rate = frame_rate;
                        target.data = out;
                        target.seq = target.seq.wrapping_add(1);

                        Ok(gst::FlowSuccess::Ok)
                    })
                    .build(),
            )
            .build();

        playbin.set_property("video-sink", &appsink);
        playbin.set_property("volume", &(self.volume as f64));
        playbin.set_property("uri", &self.current_player.source);
        playbin.set_state(gst::State::Paused)?;

        self.playback.video_sink = Some(appsink);
        self.playback.pipeline = Some(playbin);

        Ok(())
    }

    pub(crate) fn reset_pipeline(&mut self) {
        self.reset_playback();
        self.is_dragging_progress_bar = false;
        self.pending_seek_position = None;
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

    pub(crate) fn start_loading_timeout_task(&mut self, cx: &mut Context<Self>) {
        if self.playback.loading_timeout_task.is_some() {
            return;
        }

        let source = self.current_player.source.clone();
        let session_id = self.playback.session_id;

        if source.starts_with("file://") {
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
                let still_loading_same_source = this.current_player.source == source
                    && this.playback.pipeline.is_some()
                    && this.frames.current_image().is_none();

                this.playback.loading_timeout_task = None;
                if still_loading_same_source {
                    log::info!("[video:loading-timeout] source={source}");
                    this.reset_pipeline();
                    this.playback.state = PlatState::Error("加载视频源超时".to_string());
                    cx.notify();
                }
            });
        }));
    }

    // 监听总线消息
    pub(crate) fn start_event_bus(&mut self, cx: &mut Context<Self>) {
        if self.playback.bus_watch_started {
            return;
        }
        let Some(playbin) = self.playback.pipeline.clone() else {
            return;
        };
        let Some(bus) = playbin.bus() else {
            return;
        };

        let is_local_file = self.current_player.source.starts_with("file://");
        let session_id = self.playback.session_id;

        self.playback.bus_watch_started = true;
        self.playback.bus_watch_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;

                let session_is_current = this
                    .update(cx, |this, _| this.playback.is_current_session(session_id))
                    .unwrap_or(false);
                if !session_is_current {
                    break;
                }

                let mut stop_loop = false;
                while let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(0)) {
                    match msg.view() {
                        // 播放异常
                        gst::MessageView::Error(err) => {
                            log::info!(
                                "[gst:error] source={} error={} debug={:?}",
                                msg.src()
                                    .map(|src| src.path_string())
                                    .unwrap_or_else(|| "unknown".into()),
                                err.error(),
                                err.debug()
                            );
                            let _ = this.update(cx, |this, cx| {
                                if !this.playback.is_current_session(session_id) {
                                    return;
                                }
                                this.reset_pipeline();
                                this.playback.state = PlatState::Error("播放失败".to_string());
                                log::info!("{}", format!("{} ({:?})", err.error(), err.debug()));
                                cx.notify();
                            });
                            stop_loop = true;
                            break;
                        }

                        // 警告信息
                        gst::MessageView::Warning(warn) => {
                            log::info!(
                                "[gst:warning] source={} warning={} debug={:?}",
                                msg.src()
                                    .map(|src| src.path_string())
                                    .unwrap_or_else(|| "unknown".into()),
                                warn.error(),
                                warn.debug()
                            );
                        }

                        // 播放的缓冲
                        gst::MessageView::Buffering(buffering) if !is_local_file => {
                            let percent = buffering.percent();
                            log::info!("[gst:buffering] {percent}%");

                            if percent < 100 {
                                let _ = playbin.set_state(gst::State::Paused);
                                let _ = this.update(cx, |this, cx| {
                                    if !this.playback.is_current_session(session_id) {
                                        return;
                                    }
                                    this.playback.state =
                                        PlatState::Cache(format!("缓冲中 {percent}%"));
                                    cx.notify();
                                });
                            } else {
                                let _ = playbin.set_state(gst::State::Playing);
                                let _ = this.update(cx, |this, cx| {
                                    if !this.playback.is_current_session(session_id) {
                                        return;
                                    }
                                    this.playback.state = PlatState::Loading;
                                    cx.notify();
                                });
                            }
                        }

                        // 监听播放状态
                        gst::MessageView::StateChanged(state) => {
                            if msg
                                .src()
                                .map(|src| src.name() == "video-playbin")
                                .unwrap_or(false)
                            {
                                log::info!(
                                    "[gst:state] {:?} -> {:?} pending={:?}",
                                    state.old(),
                                    state.current(),
                                    state.pending()
                                );
                            }
                        }

                        // 同步视频和音频轨道
                        gst::MessageView::Latency(_) => {
                            log::info!("recalculating latency");
                            if let Ok(bin) = playbin.clone().dynamic_cast::<gst::Bin>() {
                                let _ = bin.recalculate_latency();
                            }
                        }

                        //  组件内部的消息
                        gst::MessageView::Element(element) => {
                            if let Some(structure) = element.structure() {
                                let source = msg
                                    .src()
                                    .map(|src| src.name().to_string())
                                    .unwrap_or_else(|| "unknown".into());
                                log::info!("[gst:element] {} from {}", structure.name(), source);
                            }
                        }

                        // 播放结束 读取不到 视频流的数据
                        gst::MessageView::Eos(_) => {
                            log::info!("[gst:eos]");
                            let _ = this.update(cx, |this, cx| {
                                if !this.playback.is_current_session(session_id) {
                                    return;
                                }
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

    // 刷新gpui 的进度条
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
                let should_continue = this
                    .update(cx, |this, cx| this.update_progress(session_id, cx))
                    .unwrap_or(false);
                if !should_continue {
                    break;
                }
            }
        }));
    }

    //  刷新视频的帧
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
                let should_continue = this
                    .update(cx, |this, cx| this.update_frame(session_id, cx))
                    .unwrap_or(false);
                if !should_continue {
                    break;
                }
            }
        }));
    }

    fn update_progress(&mut self, session_id: u64, cx: &mut Context<Self>) -> bool {
        if !self.playback.is_current_session(session_id) {
            return false;
        }

        if let Some(playbin) = &self.playback.pipeline {
            if let Some(pos) = playbin.query_position::<gst::ClockTime>() {
                self.position = self.clock_to_duration(pos);
                // println!("[video] pos={}ms", self.video_player_duration.as_millis());
            }
            let needs_duration = self
                .total_duration
                .map(|d| d.as_nanos() == 0)
                .unwrap_or(true);
            if needs_duration {
                if let Some(total) = playbin.query_duration::<gst::ClockTime>() {
                    let duration = self.clock_to_duration(total);
                    // println!("[video] duration={}ms", duration.as_millis());
                    if duration.as_nanos() > 0 {
                        self.total_duration = Some(duration);
                    }
                }
            }
        }

        // Loading/Cache 阶段也要查询 duration，否则任务会在首帧到达前退出。
        let should_continue = matches!(
            self.playback.state,
            PlatState::Loading | PlatState::Playing | PlatState::Cache(_)
        ) || self.is_dragging_progress_bar;
        if !should_continue {
            self.playback.progress_task = None;
        }
        cx.notify();
        should_continue
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

        let should_continue = self.playback.pipeline.is_some();
        if !should_continue {
            self.playback.frame_task = None;
        }
        should_continue
    }
    pub(crate) fn get_progress_position(
        &self,
        position: Point<Pixels>,
        bounds: Bounds<Pixels>,
    ) -> Option<Duration> {
        let total = self.total_duration?;
        if total.as_nanos() == 0 {
            return None;
        }
        let left = bounds.origin.x.as_f32();
        let width = bounds.size.width.as_f32().max(1.0);
        let ratio = ((position.x.as_f32() - left) / width).clamp(0.0, 1.0);
        let seconds = total.as_secs_f32() * ratio;
        Some(Duration::from_secs_f32(seconds))
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
}
