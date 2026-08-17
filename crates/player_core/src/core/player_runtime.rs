use gpui::*;
use gstreamer as gst;
use image::{Frame, RgbaImage};
use std::sync::{Arc, Mutex};

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
    pub(crate) video_filter: Option<gst::Element>,
    pub(crate) progress_task: Option<Task<()>>,
    pub(crate) frame_task: Option<Task<()>>,
    pub(crate) bus_watch_task: Option<Task<()>>,
    pub(crate) loading_timeout_task: Option<Task<()>>,
    pub(crate) loading_progress: u64,
    pub(crate) last_buffering_percent: Option<i32>,
    pub(crate) bus_watch_started: bool,
}

impl Default for PlaybackRuntime {
    fn default() -> Self {
        Self {
            session_id: 0,
            state: PlatState::UnLoading,
            pipeline: None,
            video_filter: None,
            progress_task: None,
            frame_task: None,
            bus_watch_task: None,
            loading_timeout_task: None,
            loading_progress: 0,
            last_buffering_percent: None,
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
pub(crate) struct FrameBuffer {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) frame_rate: f64,
    pub(crate) data: Vec<u8>,
    pub(crate) seq: u64,
}

pub(crate) struct PresentedFrame {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) frame_rate: f64,
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
    pub(crate) fn latest_frame(&self) -> Arc<Mutex<FrameBuffer>> {
        self.latest_frame.clone()
    }

    pub fn current_image(&self) -> Option<Arc<RenderImage>> {
        self.current_image.clone()
    }

    pub(crate) fn reset(&mut self) {
        if let Some(image) = self.current_image.take() {
            self.retired_images.push(image);
        }
        self.latest_frame = Arc::new(Mutex::new(FrameBuffer::default()));
        self.last_presented_sequence = 0;
    }

    pub(crate) fn submit_latest_frame(&mut self) -> Option<PresentedFrame> {
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

    pub(crate) fn release_images(&mut self, cx: &mut App) {
        if let Some(image) = self.current_image.take() {
            cx.drop_image(image, None);
        }
        for image in self.retired_images.drain(..) {
            cx.drop_image(image, None);
        }
    }
}
