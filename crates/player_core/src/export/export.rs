use crate::transcoder::transcoder::{
    PlayCoreTranscodeFormat, PlayCoreTranscodeRequest, PlayCoreTranscodeTrim, PlayCoreTranscoder,
    make_uri_decodebin,
};
use anyhow::{Context as AnyhowContext, bail};
use gpui_kit::http_client::Url;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayCoreExportTrim {
    pub start: Duration,
    pub end: Duration,
}

#[derive(Clone, Debug)]
pub struct PlayCoreExportRequest {
    pub input: PathBuf,
    pub output: PathBuf,
    pub format: PlayCoreTranscodeFormat,
    pub trim: Option<PlayCoreExportTrim>,
}

pub struct PlayCoreExport;

impl PlayCoreExport {
    pub async fn export(request: PlayCoreExportRequest) -> anyhow::Result<()> {
        tokio::task::spawn_blocking(move || Self::export_blocking(request))
            .await
            .context("导出任务异常")?
    }

    pub fn export_blocking(request: PlayCoreExportRequest) -> anyhow::Result<()> {
        validate_request(&request)?;

        // FLAC cannot reliably restart its encoder after a flushing preroll seek.
        // Trim to PCM first, then encode that bounded source without another seek.
        if request.format == PlayCoreTranscodeFormat::FLAC && request.trim.is_some() {
            let directory =
                std::env::temp_dir().join(format!("gpui-medio-trim-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&directory).context("创建裁剪临时目录失败")?;
            let intermediate = directory.join("clip.wav");
            log::info!(
                "[gst:export] staging FLAC trim through PCM: {}",
                intermediate.display()
            );
            let result = Self::export_blocking(PlayCoreExportRequest {
                input: request.input,
                output: intermediate.clone(),
                format: PlayCoreTranscodeFormat::WAV,
                trim: request.trim,
            })
            .and_then(|_| {
                PlayCoreTranscoder::transcode_offline_blocking(PlayCoreTranscodeRequest {
                    input: intermediate.clone(),
                    output: request.output,
                    format: PlayCoreTranscodeFormat::FLAC,
                })
            });
            if intermediate.exists() {
                if let Err(error) = std::fs::remove_file(&intermediate) {
                    log::warn!(
                        "[gst:export] temporary PCM cleanup failed: {}: {error}",
                        intermediate.display()
                    );
                }
            }
            if let Err(error) = std::fs::remove_dir(&directory) {
                log::warn!(
                    "[gst:export] temporary directory cleanup failed: {}: {error}",
                    directory.display()
                );
            }
            return result;
        }

        let input_uri = Url::from_file_path(&request.input)
            .map_err(|_| anyhow::anyhow!("无法转换输入文件路径: {}", request.input.display()))?
            .to_string();
        let source = make_uri_decodebin(&input_uri)?;
        let trim = request.trim.map(|trim| PlayCoreTranscodeTrim {
            start: trim.start,
            end: trim.end,
        });
        PlayCoreTranscoder::transcode_source_blocking(source, &request.output, request.format, trim)
    }
}

fn validate_request(request: &PlayCoreExportRequest) -> anyhow::Result<()> {
    if !request.input.is_file() {
        bail!("输入文件不存在: {}", request.input.display());
    }
    if let Some(trim) = request.trim {
        if trim.start >= trim.end {
            bail!("导出裁剪区间无效: start 必须小于 end");
        }
    }
    Ok(())
}
