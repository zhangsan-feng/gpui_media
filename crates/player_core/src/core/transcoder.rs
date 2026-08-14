use super::export::{
    PlayCoreExport, PlayCoreExportRequest, build_output_branches, connect_decodebin_pad_added,
    prepare_output, run_pipeline_until_input_end,
};
use anyhow::Context as AnyhowContext;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use std::path::PathBuf;
use std::thread;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayCoreTranscodeFormat {
    Mp4,
    Mkv,
    MOV,
    Mp3,
    FLAC,
    WAV,
}

#[derive(Clone, Debug)]
pub struct PlayCoreTranscodeRequest {
    pub input: PathBuf,
    pub output: PathBuf,
    pub format: PlayCoreTranscodeFormat,
}

#[derive(Clone, Debug)]
pub struct PlayCoreRealtimeTranscodeRequest {
    pub output: PathBuf,
    pub format: PlayCoreTranscodeFormat,
    pub input_mime: Option<String>,
}

pub struct PlayCoreTranscoder;

pub struct PlayCoreTranscodeSession {
    sender: mpsc::Sender<Vec<u8>>,
    result: oneshot::Receiver<anyhow::Result<()>>,
}

impl PlayCoreTranscoder {
    pub async fn transcode_offline(request: PlayCoreTranscodeRequest) -> anyhow::Result<()> {
        PlayCoreExport::export(PlayCoreExportRequest {
            input: request.input,
            output: request.output,
            format: request.format,
            trim: None,
            filters: None,
        })
        .await
    }

    pub fn transcode_offline_blocking(request: PlayCoreTranscodeRequest) -> anyhow::Result<()> {
        PlayCoreExport::export_blocking(PlayCoreExportRequest {
            input: request.input,
            output: request.output,
            format: request.format,
            trim: None,
            filters: None,
        })
    }

    pub fn start_realtime(
        request: PlayCoreRealtimeTranscodeRequest,
    ) -> anyhow::Result<PlayCoreTranscodeSession> {
        gst::init().context("初始化 GStreamer 失败")?;
        prepare_output(&request.output)?;

        let (sender, receiver) = mpsc::channel(8);
        let (result_sender, result) = oneshot::channel();
        thread::Builder::new()
            .name("player-core-realtime-transcode".to_string())
            .spawn(move || {
                let result = transcode_realtime_blocking(request, receiver);
                let _ = result_sender.send(result);
            })
            .context("启动实时转码线程失败")?;

        Ok(PlayCoreTranscodeSession { sender, result })
    }
}

impl PlayCoreTranscodeSession {
    pub async fn write_chunk(&self, chunk: impl Into<Vec<u8>>) -> anyhow::Result<()> {
        self.sender
            .send(chunk.into())
            .await
            .map_err(|_| anyhow::anyhow!("实时转码管线已结束"))
    }

    pub async fn finish(self) -> anyhow::Result<()> {
        let Self { sender, result } = self;
        drop(sender);
        result.await.context("实时转码线程异常")?
    }
}

fn transcode_realtime_blocking(
    request: PlayCoreRealtimeTranscodeRequest,
    mut receiver: mpsc::Receiver<Vec<u8>>,
) -> anyhow::Result<()> {
    gst::init().context("初始化 GStreamer 失败")?;

    let pipeline = gst::Pipeline::new();
    let appsrc = gst_app::AppSrc::builder()
        .stream_type(gst_app::AppStreamType::Stream)
        .format(gst::Format::Bytes)
        .is_live(false)
        .block(true)
        .max_bytes(4 * 1024 * 1024)
        .automatic_eos(false)
        .build();
    if let Some(mime) = request.input_mime.as_deref() {
        appsrc.set_caps(Some(&gst::Caps::builder(mime).build()));
    }

    let decodebin = gst::ElementFactory::make("decodebin").build()?;
    let appsrc_element = appsrc.upcast_ref::<gst::Element>();
    pipeline.add(appsrc_element)?;
    pipeline.add(&decodebin)?;
    appsrc_element.link(&decodebin)?;

    let branches = build_output_branches(&pipeline, &request.output, request.format, None)?;
    connect_decodebin_pad_added(&decodebin, branches);
    run_pipeline_until_input_end(&pipeline, &appsrc, &mut receiver)
}
