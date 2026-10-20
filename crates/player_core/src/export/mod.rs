pub mod export;
mod timeline;

#[derive(Clone, Debug)]
pub struct PlayCoreTimelineExportRequest {
    pub input: std::path::PathBuf,
    pub output: std::path::PathBuf,
    pub format: crate::PlayCoreTranscodeFormat,
    pub segments: Vec<crate::PlayCoreExportTrim>,
}
