use crate::{
    PlayCoreExport, PlayCoreExportTrim, PlayCoreTimelineExportRequest, PlayCoreTranscodeFormat,
};
use anyhow::{Context, bail};
use std::path::{Path, PathBuf};

impl PlayCoreExport {
    pub fn save_format(path: &Path) -> Option<PlayCoreTranscodeFormat> {
        Some(
            match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
                "mp3" => PlayCoreTranscodeFormat::Mp3,
                "flac" => PlayCoreTranscodeFormat::FLAC,
                "wav" => PlayCoreTranscodeFormat::WAV,
                "aac" => PlayCoreTranscodeFormat::AAC,
                "mp4" => PlayCoreTranscodeFormat::Mp4,
                "mkv" => PlayCoreTranscodeFormat::Mkv,
                "mov" => PlayCoreTranscodeFormat::MOV,
                _ => return None,
            },
        )
    }

    /// Render the full edited timeline, then replace the source on the same filesystem.
    /// The caller must release any playback pipeline that holds the source open.
    pub async fn save_timeline(
        input: PathBuf,
        segments: Vec<PlayCoreExportTrim>,
    ) -> anyhow::Result<PathBuf> {
        tokio::task::spawn_blocking(move || {
            let format =
                Self::save_format(&input).context("原文件格式不支持直接保存，请使用导出")?;
            let input = std::fs::canonicalize(input).context("无法定位原文件")?;
            replace_with(&input, |output| {
                Self::export_timeline_blocking(PlayCoreTimelineExportRequest {
                    input: input.clone(),
                    output: output.to_path_buf(),
                    format,
                    segments,
                })
            })?;
            Ok(input)
        })
        .await
        .context("保存剪辑任务异常")?
    }
}

struct StagingDirectory(PathBuf);
impl Drop for StagingDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            log::warn!(
                "[media-save] staging cleanup failed: {}: {error}",
                self.0.display()
            );
        }
    }
}

fn replace_with(
    input: &Path,
    render: impl FnOnce(&Path) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let original = std::fs::metadata(input).context("无法读取原文件信息")?;
    if !original.is_file() || original.permissions().readonly() {
        bail!("原文件不是可写的普通文件");
    }
    let modified = original.modified().context("无法读取原文件修改时间")?;
    let parent = input.parent().context("无法确定原文件目录")?;
    let directory = parent.join(format!(".gpui-medio-save-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).context("无法在原文件目录创建保存临时目录")?;
    let staging = StagingDirectory(directory);
    let output = staging
        .0
        .join("edited")
        .with_extension(input.extension().unwrap_or_default());
    log::info!(
        "[media-save] staging: input={} output={}",
        input.display(),
        output.display()
    );
    render(&output).context("生成剪辑结果失败")?;
    if std::fs::metadata(&output).context("未生成剪辑成品")?.len() == 0 {
        bail!("剪辑成品为空，保留原文件");
    }
    std::fs::OpenOptions::new()
        .write(true)
        .open(&output)?
        .sync_all()
        .context("写入剪辑成品失败")?;
    std::fs::set_permissions(&output, original.permissions()).context("保留文件权限失败")?;
    let current = std::fs::metadata(input).context("原文件已被移走，取消替换")?;
    if current.len() != original.len() || current.modified()? != modified {
        bail!("保存期间原文件被其他程序修改，已取消替换");
    }
    // rename replaces an existing file without deleting it first. On Windows this
    // uses MoveFileExW with MOVEFILE_REPLACE_EXISTING. A failure leaves the source intact.
    std::fs::rename(&output, input).context("替换原文件失败，文件可能被其他程序占用")?;
    log::info!("[media-save] replaced original: {}", input.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target")
                .join(format!("medio-save-test-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn source(&self) -> PathBuf {
            let path = self.0.join("original.mp3");
            std::fs::write(&path, b"original media").unwrap();
            path
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn save_replaces_only_after_success_and_cleans_staging() {
        let fixture = Fixture::new();
        let input = fixture.source();
        replace_with(&input, |output| {
            assert_eq!(std::fs::read(&input)?, b"original media");
            std::fs::write(output, b"edited media")?;
            Ok(())
        })
        .unwrap();
        assert_eq!(std::fs::read(&input).unwrap(), b"edited media");
        assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 1);
    }

    #[test]
    fn failed_render_preserves_original_and_cleans_partial_output() {
        let fixture = Fixture::new();
        let input = fixture.source();
        assert!(
            replace_with(&input, |output| {
                std::fs::write(output, b"partial")?;
                anyhow::bail!("encoder failed")
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&input).unwrap(), b"original media");
        assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 1);
    }

    #[test]
    fn empty_render_cannot_replace_original() {
        let fixture = Fixture::new();
        let input = fixture.source();
        assert!(
            replace_with(&input, |output| {
                std::fs::write(output, b"")?;
                Ok(())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&input).unwrap(), b"original media");
    }

    #[test]
    fn concurrent_file_change_is_preserved() {
        let fixture = Fixture::new();
        let input = fixture.source();
        assert!(
            replace_with(&input, |output| {
                std::fs::write(output, b"edited media")?;
                std::fs::write(&input, b"external application changed the original")?;
                Ok(())
            })
            .is_err()
        );
        assert_eq!(
            std::fs::read(&input).unwrap(),
            b"external application changed the original"
        );
    }

    #[test]
    fn save_format_preserves_container_and_rejects_unsupported_extensions() {
        assert_eq!(
            PlayCoreExport::save_format(Path::new("clip.MP3")),
            Some(PlayCoreTranscodeFormat::Mp3)
        );
        assert_eq!(PlayCoreExport::save_format(Path::new("clip.m4a")), None);
    }
}
