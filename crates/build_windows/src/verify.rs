use crate::environment::{runtime_environment, source_environment};
use anyhow::{Context, Result, bail};
use build_windows::package::RuntimeManifest;
use serde_json::json;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Output};

pub fn source(manifest: &RuntimeManifest, gst_root: &Path) -> Result<()> {
    let inspect = gst_root.join("bin").join("gst-inspect-1.0.exe");
    inspect_features(
        manifest,
        &inspect,
        gst_root,
        &source_environment(gst_root)?,
        None,
    )
}

pub fn package(manifest: &RuntimeManifest, root: &Path) -> Result<()> {
    let registry = root.join("registry.bin");
    if registry.exists() {
        std::fs::remove_file(&registry)?;
    }
    let inspect = root.join("gst-inspect-1.0.exe");
    inspect_features(
        manifest,
        &inspect,
        root,
        &runtime_environment(root)?,
        Some(root),
    )
}

fn inspect_features(
    manifest: &RuntimeManifest,
    inspect: &Path,
    cwd: &Path,
    environment: &BTreeMap<String, OsString>,
    report_root: Option<&Path>,
) -> Result<()> {
    execute(inspect, &["--version"], cwd, environment)?;
    let mut resolved = Vec::new();
    for feature in &manifest.required_features {
        execute(inspect, &[feature], cwd, environment)
            .with_context(|| format!("required GStreamer feature is missing: {feature}"))?;
        resolved.push(feature.clone());
    }
    if let Some(root) = report_root {
        std::fs::write(
            root.join("verification.json"),
            serde_json::to_vec_pretty(&json!({
                "resolved_features": resolved,
            }))?,
        )?;
    }
    Ok(())
}

fn execute(
    program: &Path,
    arguments: &[&str],
    cwd: &Path,
    environment: &BTreeMap<String, OsString>,
) -> Result<Output> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(cwd)
        .envs(environment)
        .output()
        .with_context(|| format!("failed to run {}", program.display()))?;
    if !output.status.success() {
        bail!(
            "{} {} failed:\n{}",
            program.display(),
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output)
}
