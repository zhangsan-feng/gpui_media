use super::super::{ExportStatus, MediaWorkspace};
use gpui_kit::{AsyncApp, Context, PathPromptOptions};
use player_core::{
    PlayCoreDownload, PlayCoreDownloadRequest, PlayCoreExport, PlayCoreExportTrim,
    PlayCoreTimelineExportRequest, PlayCoreTranscodeFormat, PlayCoreTranscodeRequest,
    PlayCoreTranscoder, PlayStatic,
};
use std::path::{Path, PathBuf};

impl MediaWorkspace {
    pub(in crate::gui::home::media_workspace) fn export_media(
        &mut self,
        format: PlayCoreTranscodeFormat,
        cx: &mut Context<Self>,
    ) {
        self.export_source(format, None, cx);
    }

    pub(in crate::gui::home::media_workspace) fn export_clip(
        &mut self,
        format: PlayCoreTranscodeFormat,
        cx: &mut Context<Self>,
    ) {
        let Some(_) = self.clip_range(cx) else {
            return;
        };
        if !self
            .play_core
            .read(cx)
            ._view_state()
            .player
            .url
            .starts_with("file://")
        {
            self.clip.error = Some("裁剪导出需要本地素材，请先下载并导入媒体文件".into());
            cx.notify();
            return;
        }
        let segments = self
            .clip
            .timeline
            .as_ref()
            .map(|timeline| timeline.ranges().to_vec())
            .unwrap_or_default();
        self.export_source(format, Some(segments), cx);
    }

    fn export_source(
        &mut self,
        format: PlayCoreTranscodeFormat,
        trim: Option<Vec<PlayCoreExportTrim>>,
        cx: &mut Context<Self>,
    ) {
        let source = self.play_core.read(cx)._view_state().player;
        if source.url.trim().is_empty() {
            self.export_status = Some(ExportStatus::Error("暂无可导出的媒体".to_string()));
            cx.notify();
            return;
        }
        if self.export_in_progress {
            return;
        }

        let suggested_name = suggested_export_name(&source.title, format);
        let suggested_name = if trim.is_some() {
            let stem = Path::new(&suggested_name)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            format!("{stem}_edited.{}", format.extension())
        } else {
            suggested_name
        };
        let save_dialog = cx.prompt_for_new_path(&PathBuf::default(), Some(&suggested_name));
        let entity = cx.entity().clone();
        let mut cx_async = cx.to_async().clone();
        let format_label = export_format_label(format).to_string();

        self.export_in_progress = true;
        self.export_status = Some(ExportStatus::Working(format!(
            "正在准备导出 {format_label}..."
        )));
        cx.notify();

        cx.spawn(move |_, _: &mut AsyncApp| async move {
            let path = match save_dialog.await {
                Ok(Ok(Some(path))) => path,
                Ok(Ok(None)) => {
                    let _ = entity.update(&mut cx_async, |this, cx| {
                        this.export_in_progress = false;
                        this.export_status = None;
                        cx.notify();
                    });
                    return;
                }
                Ok(Err(error)) => {
                    let _ = entity.update(&mut cx_async, |this, cx| {
                        this.finish_export(
                            Err(anyhow::anyhow!("打开保存对话框失败: {error:#}")),
                            cx,
                        );
                    });
                    return;
                }
                Err(error) => {
                    let _ = entity.update(&mut cx_async, |this, cx| {
                        this.finish_export(Err(anyhow::anyhow!("保存路径选择失败: {error:#}")), cx);
                    });
                    return;
                }
            };

            let output = path.with_extension(export_format_extension(format));
            log::info!(
                "[media-editor:export] item_id={} output={} trim={trim:?}",
                source.id,
                output.display()
            );
            let result = tokio::spawn(run_export(source, output.clone(), format, trim))
                .await
                .map_err(|error| anyhow::anyhow!("导出任务异常: {error}"))
                .and_then(|result| result.map(|_| output));

            let _ = entity.update(&mut cx_async, |this, cx| {
                this.finish_export(result, cx);
            });
        })
        .detach();
    }

    pub(in crate::gui::home::media_workspace) fn export_playlist(
        &mut self,
        format: PlayCoreTranscodeFormat,
        cx: &mut Context<Self>,
    ) {
        if self.export_in_progress {
            return;
        }

        if self.play_list.is_empty() {
            self.export_status = Some(ExportStatus::Error("播放列表为空".to_string()));
            cx.notify();
            return;
        }

        let tasks = self.play_list.clone();
        let total_items = self.play_list.len();
        let folder_dialog = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("选择批量导出目录".into()),
        });
        let entity = cx.entity().clone();
        let mut cx_async = cx.to_async().clone();

        self.export_in_progress = true;
        self.export_status = Some(ExportStatus::Working(format!(
            "正在准备按 {} 批量导出 {} 个媒体...",
            export_format_label(format),
            total_items
        )));
        cx.notify();

        cx.spawn(move |_, _: &mut AsyncApp| async move {
            let folder = match folder_dialog.await {
                Ok(Ok(Some(paths))) => paths.into_iter().find(|path| path.is_dir()),
                Ok(Ok(None)) => {
                    let _ = entity.update(&mut cx_async, |this, cx| {
                        this.export_in_progress = false;
                        this.export_status = None;
                        cx.notify();
                    });
                    return;
                }
                Ok(Err(error)) => {
                    let _ = entity.update(&mut cx_async, |this, cx| {
                        this.finish_export(
                            Err(anyhow::anyhow!("打开批量导出目录选择器失败: {error:#}")),
                            cx,
                        );
                    });
                    return;
                }
                Err(error) => {
                    let _ = entity.update(&mut cx_async, |this, cx| {
                        this.finish_export(
                            Err(anyhow::anyhow!("批量导出目录选择失败: {error:#}")),
                            cx,
                        );
                    });
                    return;
                }
            };
            let Some(folder) = folder else {
                let _ = entity.update(&mut cx_async, |this, cx| {
                    this.finish_export(
                        Err(anyhow::anyhow!("未选择有效的批量导出目录")),
                        cx,
                    );
                });
                return;
            };

            let mut succeeded = 0usize;
            let mut failed = 0usize;
            for (index, source) in tasks.into_iter().enumerate() {
                let output = unique_export_path(&folder, &source.title, format);
                let result = tokio::spawn(run_export(source.clone(), output.clone(), format, None))
                    .await
                    .map_err(|error| anyhow::anyhow!("导出任务异常: {error}"))
                    .and_then(|result| result);

                match result {
                    Ok(()) => {
                        succeeded += 1;
                        log::info!(
                            "[media-editor:batch-export] exported item_id={} output={}",
                            source.id,
                            output.display()
                        );
                    }
                    Err(error) => {
                        failed += 1;
                        log::error!(
                            "[media-editor:batch-export] failed item_id={} title={} output={}: {error:#}",
                            source.id,
                            source.title,
                            output.display()
                        );
                    }
                }

                let progress = format!(
                    "批量导出中：{}/{}（成功 {}，失败 {}）",
                    index + 1,
                    total_items,
                    succeeded,
                    failed
                );
                let _ = entity.update(&mut cx_async, |this, cx| {
                    this.export_status = Some(ExportStatus::Working(progress));
                    cx.notify();
                });
            }

            let message = format!(
                "批量导出完成：成功 {} 个，失败 {} 个（共 {} 个）",
                succeeded, failed, total_items
            );
            log::info!(
                "[media-editor:batch-export] finished succeeded={} failed={} total={} directory={}",
                succeeded,
                failed,
                total_items,
                folder.display()
            );
            let _ = entity.update(&mut cx_async, |this, cx| {
                this.export_in_progress = false;
                this.export_status = Some(if failed == 0 {
                    ExportStatus::Success(format!("{message}：{}", folder.display()))
                } else {
                    ExportStatus::Error(format!("{message}，失败详情见日志"))
                });
                cx.notify();
            });
        })
        .detach();
    }

    fn finish_export(&mut self, result: anyhow::Result<PathBuf>, cx: &mut Context<Self>) {
        self.export_in_progress = false;
        self.export_status = Some(match result {
            Ok(path) => {
                log::info!("[media-editor:export] completed: {}", path.display());
                ExportStatus::Success(format!("导出完成：{}", path.display()))
            }
            Err(error) => {
                log::error!("[media-editor:export] failed: {error:#}");
                ExportStatus::Error(format!("导出失败：{error:#}"))
            }
        });
        cx.notify();
    }
}

async fn run_export(
    source: PlayStatic,
    output: PathBuf,
    format: PlayCoreTranscodeFormat,
    trim: Option<Vec<PlayCoreExportTrim>>,
) -> anyhow::Result<()> {
    if source.url.starts_with("file://") {
        let input = super::file_path_from_url(&source.url)?;
        if let Some(segments) = trim {
            return PlayCoreExport::export_timeline(PlayCoreTimelineExportRequest {
                input,
                output,
                format,
                segments,
            })
            .await;
        }
        return PlayCoreTranscoder::transcode_offline(PlayCoreTranscodeRequest {
            input,
            output,
            format,
        })
        .await;
    }

    PlayCoreDownload::download(PlayCoreDownloadRequest {
        url: source.url,
        headers: source.headers,
        output,
        format,
    })
    .await
}

fn unique_export_path(folder: &Path, title: &str, format: PlayCoreTranscodeFormat) -> PathBuf {
    let file_name = suggested_export_name(title, format);
    let first_path = folder.join(&file_name);
    if !first_path.exists() {
        return first_path;
    }

    let file_name = Path::new(&file_name);
    let stem = file_name
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("导出媒体");
    let extension = export_format_extension(format);
    let mut suffix = 2usize;
    loop {
        let candidate = folder.join(format!("{stem}_{suffix}.{extension}"));
        if !candidate.exists() {
            return candidate;
        }
        suffix = suffix.saturating_add(1);
    }
}

pub(in crate::gui::home::media_workspace) fn export_format_label(
    format: PlayCoreTranscodeFormat,
) -> &'static str {
    match format {
        PlayCoreTranscodeFormat::Mp4 => "MP4 · H.264 + AAC",
        PlayCoreTranscodeFormat::Mkv => "MKV · H.264 + Opus",
        PlayCoreTranscodeFormat::MOV => "MOV · H.264 + AAC",
        PlayCoreTranscodeFormat::Mp3 => "MP3 音频",
        PlayCoreTranscodeFormat::FLAC => "FLAC 音频",
        PlayCoreTranscodeFormat::WAV => "WAV 音频",
        PlayCoreTranscodeFormat::AAC => "AAC 音频",
    }
}

fn export_format_extension(format: PlayCoreTranscodeFormat) -> &'static str {
    format.extension()
}

fn suggested_export_name(title: &str, format: PlayCoreTranscodeFormat) -> String {
    let title = title.rsplit(['/', '\\']).next().unwrap_or(title).trim();
    let stem = Path::new(title)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or(title);
    let safe_stem = stem
        .chars()
        .map(|character| {
            if matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            ) {
                '_'
            } else {
                character
            }
        })
        .collect::<String>();
    let stem = if safe_stem.trim().is_empty() {
        "导出媒体"
    } else {
        safe_stem.trim()
    };
    format!("{stem}.{}", export_format_extension(format))
}
