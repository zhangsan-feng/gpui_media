use crate::{PlatState, PlayCore, PlayCoreMediaType};
use gstreamer as gst;

#[derive(Default)]
pub(crate) struct PlayCoreStreamMetadata {
    media_type: PlayCoreMediaType,
}

impl PlayCore {
    pub(crate) fn stream_metadata(
        streams: impl IntoIterator<Item = gst::Stream>,
    ) -> PlayCoreStreamMetadata {
        let mut has_video = false;
        let mut has_audio = false;

        for stream in streams {
            let stream_type = stream.stream_type();
            has_video |= stream_type.contains(gst::StreamType::VIDEO);
            has_audio |= stream_type.contains(gst::StreamType::AUDIO);
        }

        PlayCoreStreamMetadata {
            media_type: if has_video {
                PlayCoreMediaType::Video
            } else if has_audio {
                PlayCoreMediaType::Audio
            } else {
                PlayCoreMediaType::Unknown
            },
        }
    }

    pub(crate) fn apply_stream_metadata(&mut self, metadata: PlayCoreStreamMetadata) {
        if metadata.media_type != PlayCoreMediaType::Unknown {
            self.media_type = metadata.media_type;
            if metadata.media_type == PlayCoreMediaType::Audio {
                self.finish_audio_loading();
            }
        }
    }

    pub(crate) fn mark_video_present(&mut self) {
        self.media_type = PlayCoreMediaType::Video;
    }

    pub(crate) fn update_codec_metadata(
        &mut self,
        video_codec: Option<String>,
        audio_codec: Option<String>,
        fallback_codec: Option<String>,
    ) {
        if video_codec.is_some() {
            self.media_type = PlayCoreMediaType::Video;
        } else if audio_codec.is_some() && self.media_type == PlayCoreMediaType::Unknown {
            self.media_type = PlayCoreMediaType::Audio;
        }

        if let Some(codec) = video_codec.or(audio_codec).or(fallback_codec) {
            if !codec.is_empty() {
                self.codec = Some(codec);
            }
        }

        if self.media_type == PlayCoreMediaType::Audio {
            self.finish_audio_loading();
        }
    }

    fn finish_audio_loading(&mut self) {
        self.playback.frame_task = None;
        self.playback.loading_timeout_task = None;
        if self.playback.state == PlatState::Loading {
            self.playback.state = PlatState::Playing;
        }
    }
}
