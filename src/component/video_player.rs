use crate::com::rgb_u8;
use anyhow::Result;
use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::popover::Popover;
use gpui_component::scroll::{ScrollableElement, Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::text::markdown;
use gpui_component::{
    Anchor, ElementExt as GpuiElementExt, StyledExt, VirtualListScrollHandle, h_flex, v_flex,
    v_virtual_list,
};
use gstreamer as gst;
use gstreamer::prelude::ElementExt as GstElementExt;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use image::{Frame, RgbaImage};
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use std::time::Duration;
use url::Url;



pub struct VideoPlayer {
    current_player_video: String,
    player_list: Vec<String>,
    vm_scroll_handle: VirtualListScrollHandle,
    video_player_volume: f32,
    video_frame_pipline: Option<gst::Element>,
    video_frame_data: Option<gst_app::AppSink>,
    is_player: bool,
    video_total_duration: Option<Duration>,
    video_player_duration: Duration,
    video_aspect: f32,
    is_scrubbing: bool,
    scrub_position: Option<Duration>,
    progress_bar_bounds: Option<Bounds<Pixels>>,
    volume_bar_bounds: Option<Bounds<Pixels>>,
    progress_task: Option<Task<()>>,
    frame_task: Option<Task<()>>,
    bus_watch_task: Option<Task<()>>,
    frame_buffer: Arc<Mutex<FrameBuffer>>,
    last_frame_seq: u64,
    render_image: Option<Arc<RenderImage>>,
    // frame_thread: Option<thread::JoinHandle<()>>,
    stop_frames: Arc<AtomicBool>,
    last_error: Option<String>,
    bus_watch_started: bool,
    pending_drop_images: Vec<Arc<RenderImage>>,
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        if let Some(playbin) = &self.video_frame_pipline {
            let _ = playbin.set_state(gst::State::Null);
        }
        self.stop_frame_thread();
    }
}

#[derive(Clone, Copy)]
struct ProgressDrag;

#[derive(Clone, Copy)]
struct VolumeDrag;

#[derive(Default)]
struct FrameBuffer {
    width: u32,
    height: u32,
    data: Vec<u8>,
    seq: u64,
}

fn clock_to_duration(clock: gst::ClockTime) -> Duration {
    Duration::from_nanos(clock.nseconds())
}


impl VideoPlayer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let _ = (window, cx);
        let _ = gst::init();
        Self {
            current_player_video: "".to_string(),
            player_list: vec![],
            vm_scroll_handle: VirtualListScrollHandle::new(),
            video_player_volume: 0.6,
            video_frame_pipline: None,
            video_frame_data: None,
            video_player_duration: Duration::from_secs(0),

            is_player: false,
            video_total_duration: None,
            video_aspect: 16.0 / 9.0,
            is_scrubbing: false,
            scrub_position: None,
            progress_bar_bounds: None,
            volume_bar_bounds: None,
            progress_task: None,
            frame_task: None,
            bus_watch_task: None,
            frame_buffer: Arc::new(Mutex::new(FrameBuffer::default())),
            last_frame_seq: 0,
            render_image: None,
            // frame_thread: None,
            stop_frames: Arc::new(AtomicBool::new(false)),
            last_error: None,
            bus_watch_started: false,
            pending_drop_images: Vec::new(),
        }
    }

    fn ensure_pipeline(&mut self) -> Result<()> {
        if self.video_frame_pipline.is_some() {
            return Ok(());
        }

        let uri = match self.video_uri() {
            Some(uri) => uri,
            None => return Ok(()),
        };

        let playbin = gst::ElementFactory::make("playbin").name("video-playbin").build()?;
        let caps = gst::Caps::builder("video/x-raw").field("format", "BGRA").build();
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
                                out[dst_start..dst_start + row_bytes].copy_from_slice(&data[src_start..src_start + row_bytes]);
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
        playbin.set_property("uri", &uri);
        playbin.set_state(gst::State::Paused)?;

        self.video_frame_data = Some(appsink);
        self.video_frame_pipline = Some(playbin);

        Ok(())
    }


    fn stop_frame_thread(&mut self) {
        self.stop_frames.store(true, Ordering::Relaxed);
    }

    fn video_uri(&self) -> Option<String> {
        let trimmed = self.current_player_video.trim();
        if trimmed.is_empty() {
            return None;
        }
        if trimmed.contains("://") {
            return Some(trimmed.to_string());
        }
        let path = Path::new(trimmed);
        let canonical = path.canonicalize().ok()?;
        let url = Url::from_file_path(canonical).ok()?;
        Some(url.to_string())
    }

    fn toggle_play(&mut self, cx: &mut Context<Self>) {
        if self.is_player {
            self.pause();
        } else {
            self.play(cx);
        }
    }

    fn play(&mut self, cx: &mut Context<Self>) {
        if self.ensure_pipeline().is_err() {
            return;
        }
        if let Some(playbin) = &self.video_frame_pipline {
            let _ = playbin.set_state(gst::State::Playing);
            self.is_player = true;
            self.ensure_bus_watch(cx);
            self.ensure_progress_task(cx);
            self.ensure_frame_task(cx);
        }
    }

    fn pause(&mut self) {
        if let Some(playbin) = &self.video_frame_pipline {
            let _ = playbin.set_state(gst::State::Paused);
        }
        self.is_player = false;
    }

    fn ensure_bus_watch(&mut self, cx: &mut Context<Self>) {
        if self.bus_watch_started {
            return;
        }
        let Some(playbin) = self.video_frame_pipline.clone() else {
            return;
        };
        let Some(bus) = playbin.bus() else {
            return;
        };

        self.bus_watch_started = true;
        self.bus_watch_task = Some(cx.spawn(async move |this, cx| {
            loop {
                // 监听总线消息
                cx.background_executor().timer(Duration::from_millis(1500)).await;

                let mut stop_loop = false;
                while let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(0)) {
                    match msg.view() {
                        gst::MessageView::Error(err) => {
                            let _ = this.update(cx, |this, cx| {
                                this.last_error = Some(format!("{} ({:?})", err.error(), err.debug()));
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

    fn ensure_progress_task(&mut self, cx: &mut Context<Self>) {
        if self.progress_task.is_some() {
            return;
        }
        self.progress_task = Some(cx.spawn(async move |this, cx| {
            loop {
                // 视频刷新率 每秒刷新多少帧的图片
                cx.background_executor().timer(Duration::from_millis(30)).await;
                let should_continue = this.update(cx, |this, cx| this.update_progress(cx)).unwrap_or(false);
                if !should_continue {
                    break;
                }
            }
        }));
    }

    fn ensure_frame_task(&mut self, cx: &mut Context<Self>) {
        if self.frame_task.is_some() {
            return;
        }
        let buffer = self.frame_buffer.clone();
        self.frame_task = Some(cx.spawn(async move |this, cx| {
            loop {
                // 刷新gpui 的进度条 每秒刷新多少次
                cx.background_executor().timer(Duration::from_millis(30)).await;
                let should_continue = this.update(cx, |this, cx| this.update_frame(&buffer, cx)).unwrap_or(false);
                if !should_continue {
                    break;
                }
            }
        }));
    }

    fn update_progress(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(playbin) = &self.video_frame_pipline {
            if let Some(pos) = playbin.query_position::<gst::ClockTime>() {
                self.video_player_duration = clock_to_duration(pos);
                // println!("[video] pos={}ms", self.video_player_duration.as_millis());
            }
            let needs_duration = self.video_total_duration.map(|d| d.as_nanos() == 0).unwrap_or(true);
            if needs_duration {
                if let Some(total) = playbin.query_duration::<gst::ClockTime>() {
                    let duration = clock_to_duration(total);
                    // println!("[video] duration={}ms", duration.as_millis());
                    if duration.as_nanos() > 0 {
                        self.video_total_duration = Some(duration);
                    }
                }
            }
        }

        let should_continue = self.is_player || self.is_scrubbing;
        if !should_continue {
            self.progress_task = None;
        }
        cx.notify();
        should_continue
    }

    fn update_frame(&mut self, buffer: &Arc<Mutex<FrameBuffer>>, cx: &mut Context<Self>) -> bool {
        let (seq, width, height, data) = {
            let guard = buffer.lock().unwrap();
            if guard.seq == self.last_frame_seq {
                return self.video_frame_pipline.is_some();
            }
            (guard.seq, guard.width, guard.height, guard.data.clone())
        };

        if width > 0 && height > 0 {
            if let Some(image) = RgbaImage::from_raw(width, height, data) {
                let frame = Frame::new(image);
                let new_image = Arc::new(RenderImage::new(vec![frame]));
                if let Some(old) = self.render_image.replace(new_image) {
                    self.pending_drop_images.push(old);
                }
                self.last_frame_seq = seq;
                self.video_aspect = (width as f32 / height as f32).max(0.01);
                cx.notify();
            }
        }

        let should_continue = self.video_frame_pipline.is_some();
        if !should_continue {
            self.frame_task = None;
        }
        should_continue
    }

    fn seek_video(&mut self, position: Duration) {
        if self.ensure_pipeline().is_err() {
            return;
        }
        if let Some(playbin) = &self.video_frame_pipline {
            let nanos = position.as_nanos().min(u64::MAX as u128) as u64;
            let target = gst::ClockTime::from_nseconds(nanos);

            let ok = playbin.seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE, target);
            println!("[video] seek ok={:?} pos={}ms", ok, position.as_millis());
            self.video_player_duration = position;
        }
    }

    fn position_from_drag( &self, position: Point<Pixels>, bounds: Bounds<Pixels>) -> Option<Duration> {
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

    fn volume_from_position(&self, position: Point<Pixels>, bounds: Bounds<Pixels>) -> f32 {
        let left = bounds.origin.x.as_f32();
        let width = bounds.size.width.as_f32().max(1.0);
        ((position.x.as_f32() - left) / width).clamp(0.0, 1.0)
    }

    fn format_time(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    fn set_volume(&mut self, volume: f32) {
        self.video_player_volume = volume.clamp(0.0, 1.0);
        if let Some(playbin) = &self.video_frame_pipline {
            playbin.set_property("volume", &(self.video_player_volume as f64));
        }
    }

    fn handle_file_drop(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        let mut added = Vec::new();
        for path in paths.paths() {
            if self.player_list.iter().any(|item| item == path) {
                continue;
            }
            added.push(path.to_string_lossy().to_string());
        }

        if added.is_empty() {
            return;
        }

        self.current_player_video = added[0].clone();
        self.player_list.extend(added);
        self.reset_pipeline();
        self.play(cx);

        cx.notify();
    }

    fn drop_video_frame(&mut self, window: &mut Window) {
        if self.pending_drop_images.is_empty() {
            return;
        }
        for image in self.pending_drop_images.drain(..) {
            let _ = window.drop_image(image);
        }
    }

    fn reset_pipeline(&mut self) {
        if let Some(playbin) = &self.video_frame_pipline {
            let _ = playbin.set_state(gst::State::Null);
        }
        self.video_frame_pipline = None;
        self.video_frame_data = None;
        self.is_player = false;
        self.video_total_duration = None;
        self.video_player_duration = Duration::from_secs(0);
        self.is_scrubbing = false;
        self.scrub_position = None;
        self.last_error = None;
        self.bus_watch_started = false;
        self.progress_task = None;
        self.frame_task = None;
        self.bus_watch_task = None;
        self.last_frame_seq = 0;
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

    fn switch_to_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.player_list.len() {
            return;
        }
        let next = self.player_list[index].clone();
        if next.is_empty() {
            return;
        }
        self.current_player_video = next;
        self.reset_pipeline();
        self.play(cx);
    }

    fn prev_video(&mut self, cx: &mut Context<Self>) {
        let len = self.player_list.len();
        if len == 0 {
            return;
        }
        let current = self.player_list.iter().position(|item| item == &self.current_player_video);
        let index = match current {
            Some(i) if i > 0 => i - 1,
            Some(_) => len - 1,
            None => len - 1,
        };
        self.switch_to_index(index, cx);
    }

    fn next_video(&mut self, cx: &mut Context<Self>) {
        let len = self.player_list.len();
        if len == 0 {
            return;
        }
        let current = self.player_list.iter().position(|item| item == &self.current_player_video);
        let index = match current {
            Some(i) if i + 1 < len => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.switch_to_index(index, cx);
    }

    fn player_list_vm(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_virtual_list(
            cx.entity().clone(),
            "video-player-vm-list",
            Rc::new(
                self.player_list
                    .iter()
                    .map(|_| size(px(100.), px(40.)))
                    .collect(),
            ),
            |view, visible_range, _, cx| {
                visible_range
                    .map(|index| {
                        let data = view.player_list[index].clone();
                        div()
                            .flex()
                            .justify_between()
                            .w_full()
                            .pr_2()
                            .child(div().gap_2().justify_between().h_flex().child(data.clone()))
                            .child(if view.current_player_video == data {
                                div().child("正在播放").into_any_element()
                            } else {
                                Button::new(("music-play-index-", index))
                                    .label("播放")
                                    .on_click({
                                        let c = data.clone();
                                        cx.listener(move |this, _, _, cx| {
                                            let c = c.clone();
                                            this.current_player_video = c;
                                            this.reset_pipeline();
                                            this.play(cx);
                                        })
                                    })
                                    .into_any_element()
                            })
                    })
                    .collect()
            },
        )
        .track_scroll(&self.vm_scroll_handle)
    }

    fn player_list_ui(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Popover::new("video-player-open-popover")
            .anchor(Anchor::BottomRight)
            .trigger(Button::new("show-form").label("播放列表").outline())
            .child(
                div()
                    .h(px(600.))
                    .w(px(800.))
                    .child(
                        v_flex()
                            .gap_2()
                            .p_4()
                            .size_full()
                            .child(self.player_list_vm(window, cx))
                            .child(
                                Scrollbar::vertical(&self.vm_scroll_handle)
                                    .scrollbar_show(ScrollbarShow::Always)
                                    .axis(ScrollbarAxis::Vertical),
                            ),
                    )
                    .with_animation(
                        "video-player-open-popover-animation",
                        Animation::new(Duration::from_millis(550)).with_easing(ease_in_out),
                        |el, delta| el.opacity(0.2 + 0.8 * delta).h(px(8. + 592. * delta)),
                    ),
            )
    }

    fn player_volume_control_ui( &self,window: &mut Window, cx: &mut Context<Self>,) -> impl IntoElement {
        let volume_ratio = self.video_player_volume.clamp(0.0, 1.0);
        let volume_bar_width = 150.0;

        h_flex().child(
            h_flex()
                .w(px(220.))
                .gap_2()
                .items_center()
                .ml_auto()
                .child(img("icon/icons8-voice-100.png").size(px(24.)))
                .child(
                    div()
                        .w(px(35.))
                        .text_size(px(11.))
                        .text_color(rgb_u8(100, 116, 139))
                        .child(format!("{:.0}%", volume_ratio * 100.0)),
                )
                .child(
                    div()
                        .h(px(8.))
                        .w(px(volume_bar_width))
                        .rounded_full()
                        .bg(rgb_u8(226, 232, 240))
                        .cursor_pointer()
                        .on_prepaint({
                            let volume_bar_entity = cx.entity();
                            move |bounds: Bounds<Pixels>, _window: &mut Window, cx: &mut App| {
                                let _ = volume_bar_entity.update(cx, |this, _cx| {
                                    this.volume_bar_bounds = Some(bounds);
                                });
                            }
                        })
                        .id("music_volume_bar")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                                if let Some(bounds) = this.volume_bar_bounds {
                                    let ratio = this.volume_from_position(event.position, bounds);
                                    this.set_volume(ratio);
                                }
                            }),
                        )
                        .on_drag(VolumeDrag, |_value, _offset, _window, cx| cx.new(|_| Empty))
                        .on_drag_move::<VolumeDrag>(cx.listener(
                            |this, event: &DragMoveEvent<VolumeDrag>, _window, _cx| {
                                let left = event.bounds.origin.x.as_f32();
                                let width = event.bounds.size.width.as_f32().max(1.0);
                                let ratio = ((event.event.position.x.as_f32() - left) / width).clamp(0.0, 1.0);
                                this.set_volume(ratio);
                            },
                        ))
                        .child(
                            div()
                                .h(px(8.))
                                .w(px(volume_bar_width * volume_ratio))
                                .rounded_full()
                                .bg(rgb_u8(148, 163, 184)),
                        ),
                ),
        )
    }

    fn player_progress_control_ui( &self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let total = self.video_total_duration.unwrap_or_else(|| Duration::from_secs(0));
        let display_position = self.scrub_position.filter(|_| self.is_scrubbing).unwrap_or(self.video_player_duration);
        let progress_ratio = if total.as_secs_f32() > 0.0 {
            (display_position.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let progress_bar_width = self.progress_bar_bounds.as_ref().map(|bounds| bounds.size.width.as_f32()).unwrap_or(0.0);
        let progress_bar_entity = cx.entity();

        v_flex()
            .child(
                div()
                    .h(px(8.))
                    .w_full()
                    .rounded_full()
                    .bg(rgb(0xE2E8F0))
                    .cursor_pointer()
                    .on_prepaint({
                        let progress_bar_entity = progress_bar_entity.clone();
                        move |bounds: Bounds<Pixels>, _window: &mut Window, cx: &mut App| {
                            let _ = progress_bar_entity.update(cx, |this, _cx| {
                                this.progress_bar_bounds = Some(bounds);
                            });
                        }
                    })
                    .id("video_progress_bar")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                            if let Some(bounds) = this.progress_bar_bounds {
                                if let Some(target) = this.position_from_drag(event.position, bounds){
                                    this.seek_video(target);
                                    this.is_scrubbing = false;
                                    this.scrub_position = None;
                                }
                            }
                        }),
                    )
                    .on_drag(ProgressDrag, |_value, _offset, _window, cx: &mut App| {
                        cx.new(|_| Empty)
                    })
                    .on_drag_move::<ProgressDrag>(cx.listener(
                        |this, event: &DragMoveEvent<ProgressDrag>, _window, _cx| {
                            if let Some(target) = this.position_from_drag(event.event.position, event.bounds){
                                this.is_scrubbing = true;
                                this.scrub_position = Some(target);
                            }
                        },
                    ))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, _cx| {
                            if this.is_scrubbing {
                                if let Some(target) = this.scrub_position.take() {
                                    this.seek_video(target);
                                }
                                this.is_scrubbing = false;
                            }
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, _cx| {
                            if this.is_scrubbing {
                                if let Some(target) = this.scrub_position.take() {
                                    this.seek_video(target);
                                }
                                this.is_scrubbing = false;
                            }
                        }),
                    )
                    .child(
                        div()
                            .h(px(8.))
                            .w(px(progress_bar_width * progress_ratio))
                            .rounded_full()
                            .bg(rgb(0x3B82F6)),
                    ),
            )
            .child(
                h_flex()
                    .text_size(px(12.))
                    .justify_between()
                    .w_full()
                    .child(
                        div()
                            .w(px(window.bounds().size.width.as_f32().clone() / 2.))
                            .text_color(rgb_u8(15, 23, 42))
                            .overflow_x_scrollbar()
                            .mb_3()
                            .child(
                                markdown(if let Some(player_err) = self.last_error.clone() {
                                    player_err.to_string()
                                } else {
                                    if !self.current_player_video.is_empty() {
                                        self.current_player_video.clone()
                                    } else {
                                        "没有加载视频来源".to_string()
                                    }
                                })
                                .selectable(true)
                                .whitespace_nowrap()
                                .text_color(rgb(0x94A3B8))
                                .cursor_text(),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Self::format_time(display_position))
                            .child("/")
                            .child(Self::format_time(total)),
                    ),
            )
    }
    fn player_controll_ui(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .child(
                div()
                    .size(px(28.))
                    .rounded_full()
                    .bg(rgb_u8(241, 245, 249))
                    .border_1()
                    .border_color(rgb_u8(203, 213, 225))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(12.))
                    .text_color(rgb_u8(15, 23, 42))
                    .id("music_prev_button")
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.prev_video(cx);
                    }))
                    .child("<"),
            )
            .child(
                div()
                    .size(px(36.))
                    .rounded_full()
                    .bg(rgb_u8(59, 130, 246))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(14.))
                    .text_color(white())
                    .cursor_pointer()
                    .id("music_play_button")
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.toggle_play(cx);
                    }))
                    .child(
                        if self.is_player {
                            div().child("■")
                        } else {
                            div().child("▶")
                        }
                    ),
            )
            .child(
                div()
                    .size(px(28.))
                    .rounded_full()
                    .bg(rgb_u8(241, 245, 249))
                    .border_1()
                    .border_color(rgb_u8(203, 213, 225))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(12.))
                    .text_color(rgb_u8(15, 23, 42))
                    .cursor_pointer()
                    .id("music_nest_button")
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.next_video(cx);
                    }))
                    .child(">"),
            )
    }

    fn video_frame_ui(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let aspect = self.video_aspect.max(0.01);
        let (video_width, video_height) = {
            let max_w = window.bounds().size.width.as_f32() - 240.;
            let max_h = window.bounds().size.height.as_f32() - 140.;
            let max_w = max_w.max(1.0);
            let max_h = max_h.max(1.0);
            let width_for_height = max_h * aspect;
            if width_for_height <= max_w {
                (width_for_height, max_h)
            } else {
                let height_for_width = max_w / aspect;
                (max_w, height_for_width)
            }
        };
        div()
            .flex_grow()
            .flex()
            .justify_center()
            .items_center()
            .rounded_md()
            .border_1()
            .border_color(rgb(0xE2E8F0))
            // .bg(rgb(0x0F172A))
            .child(if let Some(frame) = self.render_image.clone() {
                img(frame)
                    .w(px(video_width))
                    .h(px(video_height))
                    .object_fit(ObjectFit::Cover)
                    .into_any_element()
            } else {
                div().into_any_element()
            })
    }
}

impl Render for VideoPlayer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.drop_video_frame(window);

        v_flex()
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            .size_full()
            .p_2()
            .gap_2()
            .child(self.video_frame_ui(window, cx))
            .child(
                v_flex()
                    .gap_2()
                    .p_2()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(0xE2E8F0))
                    .child(self.player_progress_control_ui(window, cx))
                    .child(
                        h_flex()
                            .w_full()
                            .gap_4()
                            .justify_center()
                            .items_center()
                            .child(self.player_list_ui(window, cx))
                            .child(self.player_controll_ui(window, cx))
                            .child(self.player_volume_control_ui(window, cx)),
                    )
            )
    }
}
