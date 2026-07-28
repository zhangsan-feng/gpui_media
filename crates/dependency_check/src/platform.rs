use anyhow::{Context, Result};
use gst_runtime::{RuntimeLayout, RuntimeStrategy, strategy_for, validate_runtime};
use std::path::PathBuf;

pub struct Platform {
    application: PathBuf,
}

impl Platform {
    pub fn new() -> Self {
        let root = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            application: root.join(format!("gui{}", std::env::consts::EXE_SUFFIX)),
        }
    }

    pub fn dependency_error(&self) -> Option<String> {
        self.validate().err().map(|error| format!("{error:#}"))
    }

    pub fn check_dependencies(&self) -> bool {
        self.validate().is_ok()
    }

    fn validate(&self) -> Result<()> {
        if !self.application.is_file() {
            anyhow::bail!("missing application {}", self.application.display());
        }
        if strategy_for(std::env::consts::OS) == RuntimeStrategy::PrivateBundle {
            let layout = RuntimeLayout::from_executable(&self.application)?;
            validate_runtime(&layout).context("private GStreamer runtime is incomplete")?;
        }
        Ok(())
    }

    pub fn start_app(&self) -> Result<()> {
        std::process::Command::new(&self.application)
            .spawn()
            .with_context(|| format!("failed to start {}", self.application.display()))?;
        Ok(())
    }
}
