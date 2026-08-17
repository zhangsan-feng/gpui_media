use crate::PlayCoreTranscodeFormat;
use crate::transcoder::transcoder::{PlayCoreTranscoder, make_uri_decodebin};
use anyhow::{Context, bail};
use gstreamer as gst;
use gstreamer::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
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
        validate_request(&request)?;
        let source = make_uri_decodebin(&request.url)?;
        connect_source_setup(&source, request.headers);
        PlayCoreTranscoder::transcode_source_blocking(source, &request.output, request.format, None)
    }
}

pub(crate) fn connect_source_setup(source: &gst::Element, mut headers: HeaderMap) {
    if !headers.contains_key(USER_AGENT) {
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/131.0.0.0 Safari/537.36",
            ),
        );
    }

    source.connect("source-setup", false, move |values| {
        let Some(source) = values
            .get(1)
            .and_then(|value| value.get::<gst::Element>().ok())
        else {
            return None;
        };

        let has_extra_headers = source.find_property("extra-headers").is_some();
        let has_user_agent = source.find_property("user-agent").is_some();
        if !has_extra_headers && !has_user_agent {
            return None;
        }

        let mut extra_headers = gst::Structure::builder("extra-headers");
        let mut header_count = 0;
        for (name, value) in &headers {
            let Ok(value) = value.to_str() else {
                continue;
            };
            if name == USER_AGENT {
                if has_user_agent {
                    source.set_property("user-agent", value);
                }
                continue;
            }
            if has_extra_headers {
                extra_headers = extra_headers.field(name.as_str(), value.to_owned());
                header_count += 1;
            }
        }

        if has_extra_headers && header_count > 0 {
            source.set_property("extra-headers", extra_headers.build());
        }
        None
    });
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
