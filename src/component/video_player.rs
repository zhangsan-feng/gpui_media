use anyhow::Result;
use gpui::prelude::*;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::text::markdown;
use gpui_component::{ElementExt as GpuiElementExt, h_flex, v_flex};
use gstreamer as gst;
use gstreamer::prelude::ElementExt as GstElementExt;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use image::{Frame, RgbaImage};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use url::Url;

use crate::com::rgb_u8;

// pub fn window_center_options(window: &mut Window, w: f32, h: f32) -> WindowOptions {
//     let parent_bounds = window.bounds();
//     let parent_x = parent_bounds.origin.x;
//     let parent_y = parent_bounds.origin.y;

//     let parent_width = parent_bounds.size.width;
//     let parent_height = parent_bounds.size.height;

//     let child_x = parent_x + (parent_width - px(w)) / 2.0;
//     let child_y = parent_y + (parent_height - px(h)) / 2.0;
//     let mut window_options = WindowOptions::default();
//     let window_size = size(px(w), px(h));

//     let bounds = Bounds {
//         origin: Point {
//             x: child_x,
//             y: child_y,
//         },
//         size: window_size,
//     };
//     window_options.window_bounds = Some(WindowBounds::Windowed(bounds));

//     window_options.window_min_size = Some(window_size);
//     window_options.is_resizable = true;
//     window_options.titlebar = Some(TitlebarOptions {
//         title: Some(SharedString::from("")),
//         appears_transparent: false,
//         ..Default::default()
//     });
//     window_options
// }

pub struct VideoPlayer {
    current_player: String,
    player_list: Vec<String>,
    custom_render_width: Option<Bounds<Pixels>>,
    custom_render_video_bounds: Option<Bounds<Pixels>>,

    playbin: Option<gst::Element>,
    appsink: Option<gst_app::AppSink>,
    is_playing: bool,
    duration: Option<Duration>,
    position: Duration,
    video_aspect: f32,
    is_scrubbing: bool,
    scrub_position: Option<Duration>,
    progress_bar_bounds: Option<Bounds<Pixels>>,
    progress_task: Option<Task<()>>,
    frame_task: Option<Task<()>>,
    bus_watch_task: Option<Task<()>>,
    frame_buffer: Arc<Mutex<FrameBuffer>>,
    last_frame_seq: u64,
    render_image: Option<Arc<RenderImage>>,
    frame_thread: Option<thread::JoinHandle<()>>,
    stop_frames: Arc<AtomicBool>,
    last_error: Option<String>,
    bus_watch_started: bool,
}

impl VideoPlayer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let _ = (window, cx);
        let _ = gst::init();
        Self {
            current_player: "".to_string(),
            player_list: vec![],
            custom_render_width: None,
            custom_render_video_bounds: None,
            playbin: None,
            appsink: None,
            is_playing: false,
            duration: None,
            position: Duration::from_secs(0),
            video_aspect: 16.0 / 9.0,
            is_scrubbing: false,
            scrub_position: None,
            progress_bar_bounds: None,
            progress_task: None,
            frame_task: None,
            bus_watch_task: None,
            frame_buffer: Arc::new(Mutex::new(FrameBuffer::default())),
            last_frame_seq: 0,
            render_image: None,
            frame_thread: None,
            stop_frames: Arc::new(AtomicBool::new(false)),
            last_error: None,
            bus_watch_started: false,
        }
    }

    fn ensure_pipeline(&mut self) -> Result<()> {
        if self.playbin.is_some() {
            return Ok(());
        }

        let uri = match self.video_uri() {
            Some(uri) => uri,
            None => return Ok(()),
        };

        let playbin = gst::ElementFactory::make("playbin")
            .name("video-playbin")
            .build()?;

        let caps = gst::Caps::builder("video/x-raw")
            .field("format", "RGBA")
            .build();
        let appsink = gst_app::AppSink::builder()
            .caps(&caps)
            .sync(true)
            .max_buffers(1)
            .drop(true)
            .build();

        playbin.set_property("video-sink", &appsink);
        playbin.set_property("uri", &uri);
        playbin.set_state(gst::State::Paused)?;

        self.appsink = Some(appsink);
        self.playbin = Some(playbin);
        self.start_frame_thread();
        Ok(())
    }

    fn start_frame_thread(&mut self) {
        if self.frame_thread.is_some() {
            return;
        }

        let Some(appsink) = self.appsink.clone() else {
            return;
        };

        let buffer = self.frame_buffer.clone();
        let stop_flag = self.stop_frames.clone();

        self.frame_thread = Some(thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                let sample = appsink.try_pull_sample(gst::ClockTime::from_mseconds(10));
                let Some(sample) = sample else {
                    continue;
                };
                let Some(caps) = sample.caps() else {
                    continue;
                };
                let info = match gst_video::VideoInfo::from_caps(&caps) {
                    Ok(info) => info,
                    Err(_) => continue,
                };
                let width = info.width() as usize;
                let height = info.height() as usize;
                if width == 0 || height == 0 {
                    continue;
                }

                let Some(buffer_ref) = sample.buffer() else {
                    continue;
                };
                let map = match buffer_ref.map_readable() {
                    Ok(map) => map,
                    Err(_) => continue,
                };

                let stride = info.stride()[0] as usize;
                let row_bytes = width * 4;
                let data = map.as_slice();
                if data.len() < stride * height {
                    continue;
                }

                let mut out = vec![0u8; width * height * 4];
                for y in 0..height {
                    let src_start = y * stride;
                    let dst_start = y * row_bytes;
                    let src_row = &data[src_start..src_start + row_bytes];
                    let dst_row = &mut out[dst_start..dst_start + row_bytes];
                    for x in 0..width {
                        let i = x * 4;
                        dst_row[i] = src_row[i + 2];
                        dst_row[i + 1] = src_row[i + 1];
                        dst_row[i + 2] = src_row[i];
                        dst_row[i + 3] = src_row[i + 3];
                    }
                }

                let mut target = buffer.lock().unwrap();
                target.width = width as u32;
                target.height = height as u32;
                target.data = out;
                target.seq = target.seq.wrapping_add(1);
            }
        }));
    }

    fn stop_frame_thread(&mut self) {
        self.stop_frames.store(true, Ordering::Relaxed);
        if let Some(handle) = self.frame_thread.take() {
            let _ = handle.join();
        }
    }

    fn video_uri(&self) -> Option<String> {
        let trimmed = self.current_player.trim();
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
        if self.is_playing {
            self.pause();
        } else {
            self.play(cx);
        }
    }

    fn play(&mut self, cx: &mut Context<Self>) {
        if self.ensure_pipeline().is_err() {
            return;
        }
        if let Some(playbin) = &self.playbin {
            let _ = playbin.set_state(gst::State::Playing);
            self.is_playing = true;
            self.ensure_bus_watch(cx);
            self.ensure_progress_task(cx);
            self.ensure_frame_task(cx);
        }
    }

    fn pause(&mut self) {
        if let Some(playbin) = &self.playbin {
            let _ = playbin.set_state(gst::State::Paused);
        }
        self.is_playing = false;
    }

    fn ensure_bus_watch(&mut self, cx: &mut Context<Self>) {
        if self.bus_watch_started {
            return;
        }
        let Some(playbin) = self.playbin.clone() else {
            return;
        };
        let Some(bus) = playbin.bus() else {
            return;
        };

        self.bus_watch_started = true;
        self.bus_watch_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;

                let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(0)) else {
                    let keep_running = this
                        .update(cx, |this, _| this.playbin.is_some())
                        .unwrap_or(false);
                    if !keep_running {
                        break;
                    }
                    continue;
                };

                let action = match msg.view() {
                    gst::MessageView::Error(err) => {
                        Some((true, Some(format!("{} ({:?})", err.error(), err.debug()))))
                    }
                    gst::MessageView::Eos(_) => Some((true, None)),
                    _ => None,
                };

                if let Some((should_stop, error_text)) = action {
                    let _ = this.update(cx, |this, cx| {
                        if let Some(text) = error_text {
                            this.last_error = Some(text);
                        }
                        this.is_playing = false;
                        cx.notify();
                    });
                    if should_stop {
                        break;
                    }
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
                cx.background_executor()
                    .timer(Duration::from_millis(200))
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

    fn ensure_frame_task(&mut self, cx: &mut Context<Self>) {
        if self.frame_task.is_some() {
            return;
        }
        let buffer = self.frame_buffer.clone();
        self.frame_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(33))
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
        if let Some(playbin) = &self.playbin {
            if let Some(pos) = playbin.query_position::<gst::ClockTime>() {
                self.position = clock_to_duration(pos);
                // println!("[video] pos={}ms", self.position.as_millis());
            }
            let needs_duration = self.duration.map(|d| d.as_nanos() == 0).unwrap_or(true);
            if needs_duration {
                if let Some(total) = playbin.query_duration::<gst::ClockTime>() {
                    let duration = clock_to_duration(total);
                    // println!("[video] duration={}ms", duration.as_millis());
                    if duration.as_nanos() > 0 {
                        self.duration = Some(duration);
                    }
                }
            }
        }

        let should_continue = self.is_playing || self.is_scrubbing;
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
                return self.playbin.is_some();
            }
            (guard.seq, guard.width, guard.height, guard.data.clone())
        };

        if width > 0 && height > 0 {
            if let Some(image) = RgbaImage::from_raw(width, height, data) {
                let frame = Frame::new(image);
                self.render_image = Some(Arc::new(RenderImage::new(vec![frame])));
                self.last_frame_seq = seq;
                self.video_aspect = (width as f32 / height as f32).max(0.01);
                cx.notify();
            }
        }

        let should_continue = self.playbin.is_some();
        if !should_continue {
            self.frame_task = None;
        }
        should_continue
    }

    fn seek_video(&mut self, position: Duration) {
        if self.ensure_pipeline().is_err() {
            return;
        }
        if let Some(playbin) = &self.playbin {
            let nanos = position.as_nanos().min(u64::MAX as u128) as u64;
            let target = gst::ClockTime::from_nseconds(nanos);

            let ok = playbin.seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE, target);
            // println!("[video] seek ok={:?} pos={}ms", ok, position.as_millis());
            self.position = position;
        }
    }

    fn position_from_drag(
        &self,
        position: Point<Pixels>,
        bounds: Bounds<Pixels>,
    ) -> Option<Duration> {
        let total = self.duration?;
        if total.as_nanos() == 0 {
            return None;
        }
        let left = bounds.origin.x.as_f32();
        let width = bounds.size.width.as_f32().max(1.0);
        let ratio = ((position.x.as_f32() - left) / width).clamp(0.0, 1.0);
        let seconds = total.as_secs_f32() * ratio;
        Some(Duration::from_secs_f32(seconds))
    }

    fn format_time(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    fn handle_file_drop(&mut self, paths: &ExternalPaths, cx: &mut Context<Self>) {
        let mut added = Vec::new();
        for path in paths.paths() {
            added.push(path.to_string_lossy().to_string());
        }

        if added.is_empty() {
            return;
        }

        self.current_player = added[0].clone();
        self.player_list.extend(added);
        self.play(cx);

        cx.notify();
    }
}

impl Render for VideoPlayer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let total = self.duration.unwrap_or_else(|| Duration::from_secs(0));
        let display_position = self
            .scrub_position
            .filter(|_| self.is_scrubbing)
            .unwrap_or(self.position);
        let progress_ratio = if total.as_secs_f32() > 0.0 {
            (display_position.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let progress_bar_width = self
            .progress_bar_bounds
            .as_ref()
            .map(|bounds| bounds.size.width.as_f32())
            .unwrap_or(0.0);
        let progress_bar_entity = cx.entity();
        let aspect = self.video_aspect.max(0.01);

        let (video_width, video_height) = {
            let max_w = window.bounds().size.width.as_f32() - 240.;
            let max_h = window.bounds().size.height.as_f32() - 160.;
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

        // println!("video w:{} video h:{}", video_width, video_height);
        v_flex()
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.handle_file_drop(paths, cx);
            }))
            .size_full()
            .p_2()
            .gap_2()
            .child(
                div()
                    .flex_grow()
                    .flex()
                    .justify_center()
                    .items_center()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(0xE2E8F0))
                    // .bg(rgb(0x0F172A))

                    .on_prepaint({
                        let progress_bar_entity = progress_bar_entity.clone();
                        move |bounds: Bounds<Pixels>, _window: &mut Window, cx: &mut App| {
                            let _ = progress_bar_entity.update(cx, |this, cx| {
                                this.custom_render_video_bounds = Some(bounds);
                                // println!("w:{} h:{}", bounds.size.width, bounds.size.height)
                            });
                        }
                    })
                    .child(
                        if let Some(frame) = self.render_image.clone() {
                            img(frame)
                                .w(px(video_width))
                                .h(px(video_height))
                                .object_fit(ObjectFit::Cover)
                                .into_any_element()
                        } else {
                            div().into_any_element()
                        },

                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .p_2()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(0xE2E8F0))
                    .bg(rgb(0xF8FAFC))
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
                                        if let Some(target) =
                                            this.position_from_drag(event.position, bounds)
                                        {
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
                                    if let Some(target) =
                                        this.position_from_drag(event.event.position, event.bounds)
                                    {
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
                            .w_full()
                            .child(
                                div()
                                    .justify_start()
                                    .text_color(rgb_u8(15, 23, 42))
                                    .on_prepaint({
                                        let progress_bar_entity = progress_bar_entity.clone();
                                        move |bounds: Bounds<Pixels>,
                                              _window: &mut Window,
                                              cx: &mut App| {
                                            let _ = progress_bar_entity.update(cx, |this, cx| {
                                                this.custom_render_width = Some(bounds);
                                                // println!("{:?}", bounds.size.width)
                                            });
                                        }
                                    })
                                    .child(
                                        div()
                                            .overflow_x_scrollbar()
                                            .mb_3()
                                            .w(px(self
                                                .custom_render_width
                                                .as_ref()
                                                .map(|bounds| bounds.size.width.as_f32() - 10.)
                                                .unwrap_or(0.0)))
                                            .child(
                                                markdown(if !self.current_player.is_empty() {
                                                    self.current_player.clone()
                                                } else {
                                                    if let Some(player_err) =
                                                        self.last_error.clone()
                                                    {
                                                        player_err.to_string()
                                                    } else {
                                                        "No video loaded".to_string()
                                                    }
                                                })
                                                .selectable(true)
                                                .whitespace_nowrap()
                                                .text_color(rgb(0x94A3B8))
                                                .cursor_text(),
                                            ),
                                    )
                                    .into_any_element(),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .flex_shrink_0()
                                    .child(Self::format_time(display_position))
                                    .child("/")
                                    .child(Self::format_time(total)),
                            ),
                    )
                    .child(
                        h_flex().gap_3().items_center().child(
                            div()
                                .size(px(36.))
                                .rounded_full()
                                .bg(rgb(0x3B82F6))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(white())
                                .cursor_pointer()
                                .id("video_play_button")
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    this.toggle_play(cx);
                                }))
                                .child(if self.is_playing { "Pause" } else { "Play" }),
                        ),
                    ),
            )
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        if let Some(playbin) = &self.playbin {
            let _ = playbin.set_state(gst::State::Null);
        }
        self.stop_frame_thread();
    }
}

#[derive(Clone, Copy)]
struct ProgressDrag;

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
