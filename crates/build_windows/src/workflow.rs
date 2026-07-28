use crate::command::{Options, project_root};
use crate::environment::discover_gstreamer;
use crate::support;
use crate::verify;
use anyhow::{Context, Result, bail};
use build_windows::package::{
    RuntimeManifest, discover_vc_runtime, stage_runtime, validate_source_runtime,
};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn doctor(options: &Options) -> Result<()> {
    require_windows()?;
    let gst_root = discover_gstreamer(options.gst_root.as_deref())?;
    let vc_runtime = discover_vc_runtime()?;
    println!("GStreamer SDK: {}", gst_root.display());
    println!("MSVC Runtime: {}", vc_runtime.display());
    println!("Output: {}", absolute(&options.output)?.display());
    Ok(())
}

pub fn package(options: &Options) -> Result<()> {
    require_windows()?;
    let root = project_root();
    let manifest = load_manifest()?;
    let gst_root = discover_gstreamer(options.gst_root.as_deref())?;
    validate_source_runtime(&gst_root)?;
    verify::source(&manifest, &gst_root)?;

    let app = match &options.app {
        Some(path) => absolute(path)?,
        None => build_application(&root, &gst_root)?,
    };
    let output = absolute(&options.output)?;
    prepare_output(&output, options.force)?;
    let vc_runtime = discover_vc_runtime()?;
    let report = stage_runtime(&manifest, &app, &gst_root, &[vc_runtime], &output)?;
    verify::package(&manifest, &output)?;
    println!(
        "packaged {} files ({:.2} MiB) into {}",
        report.files.len(),
        report.total_size as f64 / 1024.0 / 1024.0,
        output.display()
    );
    Ok(())
}

pub fn verify(options: &Options) -> Result<()> {
    require_windows()?;
    let output = absolute(&options.output)?;
    let manifest = load_packaged_manifest(&output)?;
    verify::package(&manifest, &output)?;
    println!("verified {}", output.display());
    Ok(())
}

pub fn support(options: &Options) -> Result<()> {
    require_windows()?;
    let output = absolute(&options.output)?;
    support::print(&output, options.all)
}

fn load_manifest() -> Result<RuntimeManifest> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("gstreamer-runtime.toml");
    let source = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    RuntimeManifest::from_toml(&source)
}

fn load_packaged_manifest(output: &Path) -> Result<RuntimeManifest> {
    let path = output.join("gst-runtime-manifest.json");
    let source =
        std::fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
    RuntimeManifest::from_json(&source)
}

fn build_application(root: &Path, gst_root: &Path) -> Result<PathBuf> {
    let status = Command::new("cargo")
        .args(["build", "--release", "--bin", "gpui-medio"])
        .current_dir(root)
        .env("GSTREAMER_1_0_ROOT_MSVC_X86_64", gst_root)
        .status()
        .context("failed to run cargo build")?;
    if !status.success() {
        bail!("cargo build failed with {status}");
    }
    let app = root.join("target").join("release").join("gpui-medio.exe");
    if !app.is_file() {
        bail!("cargo did not produce {}", app.display());
    }
    Ok(app)
}

fn prepare_output(output: &Path, force: bool) -> Result<()> {
    if !output.exists() {
        return Ok(());
    }
    if !output.join("runtime-report.json").is_file() {
        bail!(
            "refusing to replace unrecognized output directory {}",
            output.display()
        );
    }
    if !force {
        bail!("output already exists: {} (use --force)", output.display());
    }
    std::fs::remove_dir_all(output)
        .with_context(|| format!("failed to remove generated output {}", output.display()))
}

fn absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn require_windows() -> Result<()> {
    if cfg!(windows) {
        Ok(())
    } else {
        bail!("build_windows only supports Windows MSVC x86_64")
    }
}
