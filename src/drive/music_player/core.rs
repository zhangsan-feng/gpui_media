use crate::com::window_center_options;
use crate::drive;
use crate::drive::music_player::MusicPlayer;
use crate::drive::video_player::VideoPlayer;
use crate::state::StateEvent::{TogglePlayMusic, UpdateMusicPlatyList};
use crate::state::{GlobalState, StateEvent};
use anyhow::anyhow;
use gpui::http_client::Url;
use gpui::prelude::*;
use gpui::*;
use gpui_component::input::InputState;
use gpui_component::*;
use gstreamer as gst;
use gstreamer::prelude::ElementExt as GstElementExt;
use gstreamer::prelude::*;
use log::error;
use std::path::Path;
use std::time::Duration;

impl MusicPlayer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> MusicPlayer {
        let _ = window;
        let mut s = MusicPlayer {
            current_player: drive::NetworkStatic::default(),
            player_list: vec![],
            vm_scroll_handle: VirtualListScrollHandle::new(),
            is_player: false,
            play_err: None,
            audio_pipeline: None,
            total_duration: None,
            current_position: Duration::from_secs(0),
            is_scrubbing: false,
            scrub_position: None,
            volume: 0.6,
            progress_task: None,
            duration_task: None,
            progress_bar_bounds: None,
            volume_bar_bounds: None,
        };
        s.init_subscribe(cx);
        s
    }

    fn init_subscribe(&mut self, cx: &mut Context<Self>) {
        let state_handler = cx.global::<GlobalState>().0.clone();

        cx.subscribe(
            &state_handler,
            |this: &mut Self, _model, event: &StateEvent, cx| match event {
                TogglePlayMusic(data) => {
                    // println!("{:?}", data.music_source);
                    this.current_player = data.clone();
                    this.play_current_music(cx);
                    cx.notify();
                }
                UpdateMusicPlatyList(data) => {
                    this.player_list = data.clone();
                }
                _ => {}
            },
        )
        .detach();
    }

    pub(crate) fn format_time(&self, duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub(crate) fn set_pipeline(&mut self, _cx: &mut Context<Self>) -> anyhow::Result<()> {
        let _ = gst::init();
        if let Some(playbin) = &self.audio_pipeline {
            let _ = playbin.set_state(gst::State::Null);
        }
        self.audio_pipeline = None;

        let source = if Path::new(&self.current_player.source).is_file() {
            let canonical = Path::new(&self.current_player.source).canonicalize()?;
            Url::from_file_path(canonical)
                .map_err(|_| anyhow!("invalid local file path"))?
                .to_string()
        } else {
            self.current_player.source.clone()
        };

        let playbin = gst::ElementFactory::make("playbin")
            .name("music-playbin")
            .build()?;
        playbin.set_property("uri", &source);
        playbin.set_property("volume", &(self.volume as f64));
        let _ = playbin.set_state(gst::State::Paused);

        let mut total_duration = None;
        for _ in 0..10 {
            if let Some(total) = playbin.query_duration::<gst::ClockTime>() {
                if total.nseconds() > 0 {
                    total_duration = Some(Duration::from_nanos(total.nseconds()));
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(30));
        }

        self.total_duration = total_duration.or_else(|| {
            playbin
                .query_duration::<gst::ClockTime>()
                .map(|d| Duration::from_nanos(d.nseconds()))
        });
        self.audio_pipeline = Some(playbin);
        Ok(())
    }

    pub(crate) fn reset_pipeline(&mut self, _cx: &mut Context<Self>) -> anyhow::Result<()> {
        self.pause();
        if let Some(playbin) = &self.audio_pipeline {
            let _ = playbin.set_state(gst::State::Null);
        }
        self.audio_pipeline = None;
        self.duration_task = None;
        self.current_position = Duration::from_secs(0);
        self.scrub_position = None;
        self.is_scrubbing = false;
        self.total_duration = None;
        Ok(())
    }

    pub(crate) fn start_progress_task(&mut self, cx: &mut Context<Self>) {
        if self.progress_task.is_some() {
            return;
        }
        self.progress_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(200))
                    .await;
                let should_continue = this
                    .update(cx, |this, cx| this.update_audio_progress(cx))
                    .unwrap_or(false);
                if !should_continue {
                    break;
                }
            }
        }));
    }

    pub(crate) fn seek_audio_progress(&mut self, position: Duration, cx: &mut Context<Self>) {
        if self.audio_pipeline.is_none() {
            if let Err(e) = self.set_pipeline(cx) {
                self.play_err = Some(e.to_string());
                error!("set_pipeline failed in seek_audio_progress: {}", e);
                return;
            }
        }
        if let Some(playbin) = &self.audio_pipeline {
            let nanos = position.as_nanos().min(u64::MAX as u128) as u64;
            let target = gst::ClockTime::from_nseconds(nanos);
            let _ = playbin.seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE, target);
            self.current_position = position;
            if self.is_player {
                let _ = playbin.set_state(gst::State::Playing);
                self.start_progress_task(cx);
            } else {
                let _ = playbin.set_state(gst::State::Paused);
            }
        }
    }

    pub(crate) fn set_volume_size(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(playbin) = &self.audio_pipeline {
            playbin.set_property("volume", &(self.volume as f64));
        }
    }

    pub(crate) fn update_audio_progress(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(playbin) = self.audio_pipeline.clone() {
            if let Some(pos) = playbin.query_position::<gst::ClockTime>() {
                self.current_position = Duration::from_nanos(pos.nseconds());
            }
            if self.total_duration.is_none() {
                if let Some(total) = playbin.query_duration::<gst::ClockTime>() {
                    self.total_duration = Some(Duration::from_nanos(total.nseconds()));
                }
            }
            if let Some(bus) = playbin.bus() {
                while let Some(msg) = bus.timed_pop(gst::ClockTime::from_mseconds(0)) {
                    match msg.view() {
                        gst::MessageView::Eos(_) => {
                            self.is_player = false;
                            self.next_music(cx);
                            break;
                        }
                        gst::MessageView::Error(err) => {
                            self.is_player = false;
                            self.play_err = Some(err.error().to_string());
                        }
                        _ => {}
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
