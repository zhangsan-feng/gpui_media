use crate::{PlayCore, PlayCoreFilterKind};
use gstreamer as gst;
use gstreamer::prelude::*;

impl PlayCoreFilterKind {
    pub(crate) fn clamp_value(self, value: f32) -> f32 {
        match self {
            Self::Brightness | Self::Hue => value.clamp(-1.0, 1.0),
            Self::Contrast | Self::Saturation => value.clamp(0.0, 2.0),
        }
    }

    fn property_name(self) -> &'static str {
        match self {
            Self::Brightness => "brightness",
            Self::Contrast => "contrast",
            Self::Saturation => "saturation",
            Self::Hue => "hue",
        }
    }
}

impl PlayCore {
    pub(crate) fn set_filter_value(&mut self, filter: PlayCoreFilterKind, value: f32) -> f32 {
        let value = filter.clamp_value(value);
        match filter {
            PlayCoreFilterKind::Brightness => self.filter_state.brightness = value,
            PlayCoreFilterKind::Contrast => self.filter_state.contrast = value,
            PlayCoreFilterKind::Saturation => self.filter_state.saturation = value,
            PlayCoreFilterKind::Hue => self.filter_state.hue = value,
        }
        self.apply_filter_value(filter, value);
        value
    }

    pub(crate) fn apply_filter_state(&self) {
        self.apply_filter_value(PlayCoreFilterKind::Brightness, self.filter_state.brightness);
        self.apply_filter_value(PlayCoreFilterKind::Contrast, self.filter_state.contrast);
        self.apply_filter_value(PlayCoreFilterKind::Saturation, self.filter_state.saturation);
        self.apply_filter_value(PlayCoreFilterKind::Hue, self.filter_state.hue);
    }

    fn apply_filter_value(&self, filter: PlayCoreFilterKind, value: f32) {
        if let Some(video_filter) = &self.playback.video_filter {
            video_filter.set_property(filter.property_name(), &(value as f64));
        }
    }

    pub(crate) fn build_video_filter(&self) -> Option<gst::Element> {
        let video_filter = match gst::ElementFactory::make("videobalance")
            .name("video-balance")
            .build()
        {
            Ok(video_filter) => video_filter,
            Err(error) => {
                log::warn!("[gst:filter] videobalance unavailable: {error}");
                return None;
            }
        };

        video_filter.set_property("brightness", &(self.filter_state.brightness as f64));
        video_filter.set_property("contrast", &(self.filter_state.contrast as f64));
        video_filter.set_property("saturation", &(self.filter_state.saturation as f64));
        video_filter.set_property("hue", &(self.filter_state.hue as f64));
        Some(video_filter)
    }
}
