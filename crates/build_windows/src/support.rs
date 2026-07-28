use crate::environment::runtime_environment;
use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

const FACTORY_TYPES: &[(&str, &str)] = &[
    ("视频解码器", "Decoder/Video"),
    ("音频解码器", "Decoder/Audio"),
    ("容器解析器", "Demuxer"),
    ("网络源", "Source/Network"),
];

pub fn print(root: &Path, all: bool) -> Result<()> {
    let inspect = root.join("gst-inspect-1.0.exe");
    if !inspect.is_file() {
        bail!(
            "Windows package not found at {}; run `cargo run --release -p \
             build_windows -- package` first",
            root.display()
        );
    }
    let registry = TemporaryRegistry::new();
    let mut environment = runtime_environment(root)?;
    environment.insert(
        "GST_REGISTRY_1_0".into(),
        registry.path.as_os_str().to_os_string(),
    );

    for (label, factory_type) in FACTORY_TYPES {
        let output = inspect_types(&inspect, root, &environment, factory_type)?;
        if all {
            println!("{label}\n{}\n", output.trim());
        } else {
            print_factories(label, &factories(&output));
        }
    }
    Ok(())
}

fn print_factories(label: &str, factories: &[Factory]) {
    println!("{label}（{}）", factories.len());
    for factory in factories {
        println!("  {} [{}]", factory.description, factory.name);
    }
    println!();
}

fn factories(output: &str) -> Vec<Factory> {
    let mut factories = output
        .lines()
        .filter_map(|line| line.split_once(":  "))
        .filter_map(|(_, detail)| detail.split_once(':'))
        .map(|(name, description)| Factory {
            name: name.trim().to_string(),
            description: description.trim().to_string(),
        })
        .collect::<Vec<_>>();
    factories.sort_by(|left, right| {
        left.description
            .to_lowercase()
            .cmp(&right.description.to_lowercase())
            .then_with(|| left.name.cmp(&right.name))
    });
    factories
}

fn inspect_types(
    inspect: &Path,
    root: &Path,
    environment: &BTreeMap<String, OsString>,
    kind: &str,
) -> Result<String> {
    let output = Command::new(inspect)
        .arg(format!("--types={kind}"))
        .current_dir(root)
        .envs(environment)
        .output()
        .with_context(|| format!("failed to run {}", inspect.display()))?;
    if !output.status.success() {
        bail!(
            "gst-inspect {kind} failed:\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

struct Factory {
    name: String,
    description: String,
}

struct TemporaryRegistry {
    path: PathBuf,
}

impl TemporaryRegistry {
    fn new() -> Self {
        Self {
            path: std::env::temp_dir()
                .join(format!("gpui-medio-support-{}.bin", std::process::id())),
        }
    }
}

impl Drop for TemporaryRegistry {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
