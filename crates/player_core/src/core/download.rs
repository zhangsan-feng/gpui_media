use super::PlayCoreTranscodeFormat;
use super::export::{
    build_output_branches, connect_decodebin_pad_added, connect_source_setup, prepare_output,
    run_export_pipeline,
};
use anyhow::{Context, bail};
use gstreamer as gst;
use gstreamer::prelude::*;
use reqwest::header::HeaderMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct PlayCoreDownloadRequest {
    pub url: String,
    pub headers: HeaderMap,
    pub output: PathBuf,
    pub format: PlayCoreTranscodeFormat,
}

pub struct PlayCoreDownload;

impl PlayCoreDownload {
    pub async fn download(request: PlayCoreDownloadRequest) -> anyhow::Result<()> {
        tokio::task::spawn_blocking(move || Self::download_blocking(request))
            .await
            .context("下载转码任务异常")?
    }

    pub fn download_blocking(request: PlayCoreDownloadRequest) -> anyhow::Result<()> {
        gst::init().context("初始化 GStreamer 失败")?;
        validate_request(&request)?;
        prepare_output(&request.output)?;

        let pipeline = gst::Pipeline::new();
        let source = gst::ElementFactory::make("uridecodebin")
            .name("download-source")
            .build()
            .context("创建 GStreamer 下载源失败")?;
        source.set_property("uri", &request.url);
        connect_source_setup(&source, request.headers);

        let branches = build_output_branches(&pipeline, &request.output, request.format, None)?;
        connect_decodebin_pad_added(&source, branches);
        pipeline.add(&source)?;

        run_export_pipeline(&pipeline, None)
    }
}

fn validate_request(request: &PlayCoreDownloadRequest) -> anyhow::Result<()> {
    if request.url.trim().is_empty() {
        bail!("下载地址为空");
    }
    if request.output.as_os_str().is_empty() {
        bail!("输出路径为空");
    }
    Ok(())
}
