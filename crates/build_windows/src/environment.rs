use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub fn discover_gstreamer(explicit: Option<&Path>) -> Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = explicit {
        candidates.push(path.to_path_buf());
    }
    if let Some(path) = std::env::var_os("GSTREAMER_1_0_ROOT_MSVC_X86_64") {
        candidates.push(PathBuf::from(path));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        candidates.push(
            PathBuf::from(local)
                .join("Programs")
                .join("gstreamer")
                .join("1.0")
                .join("msvc_x86_64"),
        );
    }
    candidates
        .into_iter()
        .find(|root| root.join("bin").join("gst-inspect-1.0.exe").is_file())
        .map(|path| std::fs::canonicalize(&path).unwrap_or(path))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "GStreamer MSVC x86_64 SDK not found; set \
                 GSTREAMER_1_0_ROOT_MSVC_X86_64 or pass --gst-root"
            )
        })
}

pub fn source_environment(root: &Path) -> Result<BTreeMap<String, OsString>> {
    let registry = std::env::temp_dir().join("gpui-medio-gstreamer-source-registry.bin");
    Ok(common_environment(
        &root.join("bin"),
        &root.join("lib").join("gstreamer-1.0"),
        &root
            .join("libexec")
            .join("gstreamer-1.0")
            .join("gst-plugin-scanner.exe"),
        &root.join("lib").join("gio").join("modules"),
        &root
            .join("etc")
            .join("ssl")
            .join("certs")
            .join("ca-certificates.crt"),
        &root.join("share").join("glib-2.0").join("schemas"),
        &registry,
    )?)
}

pub fn runtime_environment(root: &Path) -> Result<BTreeMap<String, OsString>> {
    Ok(common_environment(
        root,
        &root.join("gst-plugins"),
        &root
            .join("libexec")
            .join("gstreamer-1.0")
            .join("gst-plugin-scanner.exe"),
        &root.join("gio-modules"),
        &root
            .join("etc")
            .join("ssl")
            .join("certs")
            .join("ca-certificates.crt"),
        &root.join("share").join("glib-2.0").join("schemas"),
        &root.join("registry.bin"),
    )?)
}

fn common_environment(
    bin: &Path,
    plugins: &Path,
    scanner: &Path,
    gio_modules: &Path,
    ca_file: &Path,
    schemas: &Path,
    registry: &Path,
) -> Result<BTreeMap<String, OsString>> {
    let current_path = std::env::var_os("PATH").unwrap_or_default();
    let path = std::env::join_paths(
        std::iter::once(bin.to_path_buf()).chain(std::env::split_paths(&current_path)),
    )
    .context("failed to construct isolated PATH")?;
    if !scanner.is_file() {
        bail!("missing GStreamer plugin scanner {}", scanner.display());
    }
    Ok(BTreeMap::from([
        ("PATH".into(), path),
        ("GST_PLUGIN_PATH_1_0".into(), plugins.as_os_str().into()),
        ("GST_PLUGIN_SYSTEM_PATH_1_0".into(), OsString::new()),
        ("GST_PLUGIN_SCANNER_1_0".into(), scanner.as_os_str().into()),
        ("GST_REGISTRY_1_0".into(), registry.as_os_str().into()),
        ("GIO_EXTRA_MODULES".into(), gio_modules.as_os_str().into()),
        ("SSL_CERT_FILE".into(), ca_file.as_os_str().into()),
        ("GSETTINGS_SCHEMA_DIR".into(), schemas.as_os_str().into()),
    ]))
}
