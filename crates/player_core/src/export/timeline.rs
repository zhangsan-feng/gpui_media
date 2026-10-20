use super::PlayCoreTimelineExportRequest;
use crate::transcoder::timeline::{make_joined_source, stage_video_segment};
use crate::transcoder::transcoder::{PlayCoreTranscodeTrim, PlayCoreTranscoder, prepare_output};
use crate::{PlayCoreExport, PlayCoreExportRequest, PlayCoreExportTrim, PlayCoreTranscodeFormat};
use anyhow::{Context, bail};
use std::path::PathBuf;

impl PlayCoreExport {
    pub async fn export_timeline(request: PlayCoreTimelineExportRequest) -> anyhow::Result<()> {
        tokio::task::spawn_blocking(move || Self::export_timeline_blocking(request))
            .await
            .context("拼接导出任务异常")?
    }

    pub fn export_timeline_blocking(request: PlayCoreTimelineExportRequest) -> anyhow::Result<()> {
        if !request.input.is_file() {
            bail!("输入文件不存在: {}", request.input.display());
        }
        if request.segments.is_empty() {
            bail!("时间线为空，暂无可导出的片段");
        }
        for (index, segment) in request.segments.iter().enumerate() {
            if segment.start >= segment.end {
                bail!("第 {} 段的时间范围无效", index + 1);
            }
            if index > 0 && request.segments[index - 1].end > segment.start {
                bail!("保留片段必须按素材时间排列且不能重叠");
            }
        }
        if request.segments.len() == 1 {
            return Self::export_blocking(PlayCoreExportRequest {
                input: request.input,
                output: request.output,
                format: request.format,
                trim: request.segments.first().copied(),
            });
        }
        prepare_output(&request.output)?;
        let directory =
            std::env::temp_dir().join(format!("gpui-medio-join-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).context("创建拼接导出临时目录失败")?;
        let video = matches!(
            request.format,
            PlayCoreTranscodeFormat::Mp4
                | PlayCoreTranscodeFormat::Mkv
                | PlayCoreTranscodeFormat::MOV
        );
        let paths: Vec<PathBuf> = (0..request.segments.len())
            .map(|index| {
                directory.join(format!(
                    "part-{index}.{}",
                    if video { "mkv" } else { "wav" }
                ))
            })
            .collect();
        let result = (|| {
            for (index, (segment, path)) in request.segments.iter().zip(&paths).enumerate() {
                log::info!(
                    "[gst:timeline-export] staging {}/{}: {:?}..{:?}",
                    index + 1,
                    paths.len(),
                    segment.start,
                    segment.end
                );
                if video {
                    stage_video_segment(
                        &request.input,
                        path,
                        PlayCoreTranscodeTrim {
                            start: segment.start,
                            end: segment.end,
                        },
                    )?;
                } else {
                    Self::export_blocking(PlayCoreExportRequest {
                        input: request.input.clone(),
                        output: path.clone(),
                        format: PlayCoreTranscodeFormat::WAV,
                        trim: Some(PlayCoreExportTrim {
                            start: segment.start,
                            end: segment.end,
                        }),
                    })?;
                }
            }
            log::info!(
                "[gst:timeline-export] joining {} segments to {}",
                paths.len(),
                request.output.display()
            );
            let source = make_joined_source(&paths)?;
            PlayCoreTranscoder::transcode_source_blocking(
                source,
                &request.output,
                request.format,
                None,
            )
        })();
        for path in paths {
            if path.exists() {
                if let Err(error) = std::fs::remove_file(&path) {
                    log::warn!(
                        "[gst:timeline-export] temporary segment cleanup failed: {}: {error}",
                        path.display()
                    );
                }
            }
        }
        if let Err(error) = std::fs::remove_dir(&directory) {
            log::warn!(
                "[gst:timeline-export] temporary directory cleanup failed: {}: {error}",
                directory.display()
            );
        }
        result
    }
}
