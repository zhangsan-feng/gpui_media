use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const EXPECTED_GSTREAMER_VERSION: &str = "1.28.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStrategy {
    PrivateBundle,
    System,
}

pub fn strategy_for(os: &str) -> RuntimeStrategy {
    if os == "windows" {
        RuntimeStrategy::PrivateBundle
    } else {
        RuntimeStrategy::System
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeLayout {
    pub root: PathBuf,
    pub plugin_dir: PathBuf,
    pub gio_module_dir: PathBuf,
    pub manifest: PathBuf,
}

impl RuntimeLayout {
    pub fn from_executable(executable: &Path) -> Result<Self> {
        let root = executable
            .parent()
            .ok_or_else(|| anyhow::anyhow!("executable path has no parent"))?
            .to_path_buf();
        Ok(Self {
            plugin_dir: root.join("gst-plugins"),
            gio_module_dir: root.join("gio-modules"),
            manifest: root.join("gst-runtime-manifest.json"),
            root,
        })
    }
}

#[derive(Debug, Deserialize)]
struct PluginGroup {
    plugins: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RuntimeModuleGroup {
    destination_subdir: PathBuf,
    files: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RuntimeContract {
    platform: String,
    gstreamer_version: String,
    target: String,
    core_dlls: Vec<String>,
    plugin_groups: Vec<PluginGroup>,
    #[serde(default)]
    runtime_module_groups: Vec<RuntimeModuleGroup>,
}

pub fn validate_runtime(layout: &RuntimeLayout) -> Result<()> {
    let bytes = std::fs::read(&layout.manifest)
        .with_context(|| format!("missing {}", layout.manifest.display()))?;
    let contract: RuntimeContract = serde_json::from_slice(&bytes)?;
    if contract.platform != "windows" {
        bail!("unsupported private runtime platform {}", contract.platform);
    }
    if contract.gstreamer_version != EXPECTED_GSTREAMER_VERSION {
        bail!(
            "expected GStreamer {EXPECTED_GSTREAMER_VERSION}, got {}",
            contract.gstreamer_version
        );
    }
    if contract.target != "x86_64-pc-windows-msvc" {
        bail!("unsupported GStreamer runtime target {}", contract.target);
    }
    for dll in contract.core_dlls {
        let path = layout.root.join(&dll);
        if !path.is_file() {
            bail!("missing runtime file {dll}");
        }
    }
    for group in contract.plugin_groups {
        for plugin in group.plugins {
            let path = layout.plugin_dir.join(&plugin);
            if !path.is_file() {
                bail!("missing runtime file gst-plugins/{plugin}");
            }
        }
    }
    for group in contract.runtime_module_groups {
        for file in group.files {
            let relative = group.destination_subdir.join(&file);
            let path = layout.root.join(&relative);
            if !path.is_file() {
                bail!(
                    "missing runtime file {}",
                    relative.to_string_lossy().replace('\\', "/")
                );
            }
        }
    }
    Ok(())
}

pub fn runtime_environment(
    layout: &RuntimeLayout,
    local_app_data: &Path,
) -> BTreeMap<String, OsString> {
    BTreeMap::from([
        (
            "GST_PLUGIN_PATH_1_0".to_string(),
            layout.plugin_dir.as_os_str().to_os_string(),
        ),
        ("GST_PLUGIN_SYSTEM_PATH_1_0".to_string(), OsString::new()),
        (
            "GIO_MODULE_DIR".to_string(),
            layout.gio_module_dir.as_os_str().to_os_string(),
        ),
        (
            "GST_REGISTRY_1_0".to_string(),
            local_app_data
                .join("gpui-medio")
                .join("gstreamer")
                .join(format!("registry-{EXPECTED_GSTREAMER_VERSION}.bin"))
                .into_os_string(),
        ),
    ])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMode {
    Private,
    SystemDevelopment,
}

pub fn prepare_current_process() -> Result<RuntimeMode> {
    if strategy_for(std::env::consts::OS) == RuntimeStrategy::System {
        return Ok(RuntimeMode::SystemDevelopment);
    }
    let executable = std::env::current_exe()?;
    let layout = RuntimeLayout::from_executable(&executable)?;
    if !layout.manifest.is_file() {
        return Ok(RuntimeMode::SystemDevelopment);
    }
    validate_runtime(&layout)?;
    let local_app_data = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| layout.root.clone());
    let environment = runtime_environment(&layout, &local_app_data);
    if let Some(registry) = environment.get("GST_REGISTRY_1_0")
        && let Some(parent) = Path::new(registry).parent()
    {
        std::fs::create_dir_all(parent)?;
    }
    for (name, value) in environment {
        unsafe {
            std::env::set_var(name, value);
        }
    }
    Ok(RuntimeMode::Private)
}

#[cfg(test)]
mod tests {
    use super::{
        RuntimeLayout, RuntimeStrategy, runtime_environment, strategy_for, validate_runtime,
    };

    #[test]
    fn layout_is_anchored_to_the_executable_directory() {
        let layout =
            RuntimeLayout::from_executable(r"C:\apps\gpui-medio\gui.exe".as_ref()).unwrap();

        assert_eq!(
            layout.plugin_dir,
            std::path::PathBuf::from(r"C:\apps\gpui-medio\gst-plugins")
        );
        assert_eq!(
            layout.gio_module_dir,
            std::path::PathBuf::from(r"C:\apps\gpui-medio\gio-modules")
        );
        assert_eq!(
            layout.manifest,
            std::path::PathBuf::from(r"C:\apps\gpui-medio\gst-runtime-manifest.json")
        );
    }

    #[test]
    fn validation_reports_the_exact_missing_runtime_file() {
        let temp = tempfile::tempdir().unwrap();
        let exe = temp.path().join("gui.exe");
        std::fs::write(&exe, b"app").unwrap();
        std::fs::write(
            temp.path().join("gst-runtime-manifest.json"),
            r#"{
  "schema": 1,
  "platform": "windows",
  "gstreamer_version": "1.28.1",
  "target": "x86_64-pc-windows-msvc",
  "max_size_mib": 250,
  "core_dlls": ["gstreamer-1.0-0.dll"],
  "required_features": ["playbin"],
  "plugin_groups": [{"name":"core","plugins":["gstplayback.dll"]}]
}"#,
        )
        .unwrap();
        std::fs::write(temp.path().join("gstreamer-1.0-0.dll"), b"gst").unwrap();
        std::fs::create_dir(temp.path().join("gst-plugins")).unwrap();

        let layout = RuntimeLayout::from_executable(&exe).unwrap();
        let error = validate_runtime(&layout).unwrap_err();

        assert!(error.to_string().contains("gst-plugins/gstplayback.dll"));
    }

    #[test]
    fn private_environment_disables_system_plugins_and_uses_user_registry() {
        let layout =
            RuntimeLayout::from_executable(r"C:\apps\gpui-medio\gui.exe".as_ref()).unwrap();

        let values = runtime_environment(&layout, r"C:\Users\tester\AppData\Local".as_ref());

        assert_eq!(
            values.get("GST_PLUGIN_PATH_1_0").unwrap(),
            &std::ffi::OsString::from(r"C:\apps\gpui-medio\gst-plugins")
        );
        assert_eq!(
            values.get("GST_PLUGIN_SYSTEM_PATH_1_0").unwrap(),
            &std::ffi::OsString::new()
        );
        assert_eq!(
            values.get("GIO_MODULE_DIR").unwrap(),
            &std::ffi::OsString::from(r"C:\apps\gpui-medio\gio-modules")
        );
        assert_eq!(
            values.get("GST_REGISTRY_1_0").unwrap(),
            &std::ffi::OsString::from(
                r"C:\Users\tester\AppData\Local\gpui-medio\gstreamer\registry-1.28.1.bin"
            )
        );
    }

    #[test]
    fn windows_uses_private_bundle_while_other_platforms_keep_system_mode() {
        assert_eq!(strategy_for("windows"), RuntimeStrategy::PrivateBundle);
        assert_eq!(strategy_for("linux"), RuntimeStrategy::System);
        assert_eq!(strategy_for("macos"), RuntimeStrategy::System);
    }

    #[test]
    fn validation_reports_a_missing_dynamic_runtime_module() {
        let temp = tempfile::tempdir().unwrap();
        let exe = temp.path().join("gui.exe");
        std::fs::write(&exe, b"app").unwrap();
        std::fs::write(
            temp.path().join("gst-runtime-manifest.json"),
            r#"{
  "schema": 1,
  "platform": "windows",
  "gstreamer_version": "1.28.1",
  "target": "x86_64-pc-windows-msvc",
  "max_size_mib": 250,
  "core_dlls": ["gstreamer-1.0-0.dll"],
  "required_features": [],
  "plugin_groups": [],
  "runtime_module_groups": [{
    "name": "gio-tls",
    "source_subdir": "lib/gio/modules",
    "destination_subdir": "gio-modules",
    "files": ["gioopenssl.dll"]
  }]
}"#,
        )
        .unwrap();
        std::fs::write(temp.path().join("gstreamer-1.0-0.dll"), b"gst").unwrap();

        let layout = RuntimeLayout::from_executable(&exe).unwrap();
        let error = validate_runtime(&layout).unwrap_err();

        assert!(error.to_string().contains("gio-modules/gioopenssl.dll"));
    }
}
