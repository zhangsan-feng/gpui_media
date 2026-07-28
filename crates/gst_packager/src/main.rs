use anyhow::{Context, Result};
use gst_packager::{
    PackageOptions, RuntimeManifest, discover_vc_runtime, stage_runtime, validate_source_runtime,
};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let options = PackageOptions::parse(&args)?;
    let manifest_path = std::path::Path::new("packaging").join("gstreamer-runtime.toml");
    let source = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest = RuntimeManifest::from_toml(&source)?;
    validate_source_runtime(&manifest, &options.gst_root)?;
    let vc_runtime = discover_vc_runtime()?;
    let report = stage_runtime(
        &manifest,
        &options.app,
        &options.gst_root,
        &[vc_runtime],
        &options.output,
    )?;
    println!(
        "packaged {} files ({:.2} MiB) into {}",
        report.files.len(),
        report.total_size as f64 / 1024.0 / 1024.0,
        options.output.display()
    );
    Ok(())
}
