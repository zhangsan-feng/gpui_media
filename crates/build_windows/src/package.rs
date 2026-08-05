use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

pub const EXPECTED_TARGET: &str = "x86_64-pc-windows-msvc";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginGroup {
    pub name: String,
    pub plugins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeModuleGroup {
    pub name: String,
    pub source_subdir: PathBuf,
    pub destination_subdir: PathBuf,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeDataGroup {
    pub name: String,
    pub source_subdir: PathBuf,
    pub destination_subdir: PathBuf,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeManifest {
    pub schema: u32,
    pub platform: String,
    pub target: String,
    pub max_size_mib: u64,
    pub core_dlls: Vec<String>,
    #[serde(default)]
    pub platform_dlls: Vec<String>,
    pub required_features: Vec<String>,
    pub plugin_groups: Vec<PluginGroup>,
    #[serde(default)]
    pub runtime_module_groups: Vec<RuntimeModuleGroup>,
    #[serde(default)]
    pub runtime_data_groups: Vec<RuntimeDataGroup>,
}

impl RuntimeManifest {
    pub fn from_toml(source: &str) -> Result<Self> {
        Self::validate(toml::from_str(source)?)
    }

    pub fn from_json(source: &[u8]) -> Result<Self> {
        Self::validate(serde_json::from_slice(source)?)
    }

    fn validate(manifest: Self) -> Result<Self> {
        if manifest.platform != "windows" {
            bail!(
                "unsupported runtime extractor {}; no dependency inspector is registered",
                manifest.platform
            );
        }
        if manifest.target != EXPECTED_TARGET {
            bail!("expected target {EXPECTED_TARGET}, got {}", manifest.target);
        }
        Ok(manifest)
    }
}

pub fn validate_source_runtime(gst_root: &Path) -> Result<()> {
    let inspect = gst_root.join("bin").join("gst-inspect-1.0.exe");
    let output = std::process::Command::new(&inspect)
        .arg("--version")
        .output()
        .with_context(|| format!("failed to run {}", inspect.display()))?;
    if !output.status.success() {
        bail!("{} --version failed", inspect.display());
    }
    Ok(())
}

pub fn is_windows_system_dll(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    if name.starts_with("api-ms-win-") || name.starts_with("ext-ms-win-") {
        return true;
    }
    matches!(
        name.as_str(),
        "advapi32.dll"
            | "avrt.dll"
            | "bcrypt.dll"
            | "bcryptprimitives.dll"
            | "cfgmgr32.dll"
            | "combase.dll"
            | "comctl32.dll"
            | "comdlg32.dll"
            | "cryptbase.dll"
            | "crypt32.dll"
            | "d2d1.dll"
            | "d3d11.dll"
            | "dcomp.dll"
            | "dbghelp.dll"
            | "dnsapi.dll"
            | "dwmapi.dll"
            | "dwrite.dll"
            | "dxgi.dll"
            | "dxcore.dll"
            | "gdi32.dll"
            | "gdiplus.dll"
            | "hid.dll"
            | "icuuc.dll"
            | "imm32.dll"
            | "iphlpapi.dll"
            | "kernel32.dll"
            | "mf.dll"
            | "mfplat.dll"
            | "mfreadwrite.dll"
            | "mfuuid.dll"
            | "msctf.dll"
            | "ncrypt.dll"
            | "normaliz.dll"
            | "ntdll.dll"
            | "netapi32.dll"
            | "ole32.dll"
            | "oleacc.dll"
            | "oleaut32.dll"
            | "opengl32.dll"
            | "powrprof.dll"
            | "profapi.dll"
            | "propsys.dll"
            | "psapi.dll"
            | "rpcrt4.dll"
            | "secur32.dll"
            | "setupapi.dll"
            | "shcore.dll"
            | "shell32.dll"
            | "shlwapi.dll"
            | "uiautomationcore.dll"
            | "ucrtbase.dll"
            | "user32.dll"
            | "userenv.dll"
            | "usp10.dll"
            | "uxtheme.dll"
            | "version.dll"
            | "wldap32.dll"
            | "windowscodecs.dll"
            | "winhttp.dll"
            | "wininet.dll"
            | "winmm.dll"
            | "winspool.drv"
            | "wintrust.dll"
            | "ws2_32.dll"
            | "wtsapi32.dll"
            | "msimg32.dll"
    )
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| anyhow::anyhow!("truncated PE at offset {offset:#x}"))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| anyhow::anyhow!("truncated PE at offset {offset:#x}"))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn rva_to_offset(bytes: &[u8], section_table: usize, sections: u16, rva: u32) -> Result<usize> {
    for index in 0..sections as usize {
        let section = section_table + index * 40;
        let virtual_size = read_u32(bytes, section + 8)?;
        let virtual_address = read_u32(bytes, section + 12)?;
        let raw_size = read_u32(bytes, section + 16)?;
        let raw_offset = read_u32(bytes, section + 20)?;
        let span = virtual_size.max(raw_size);
        if rva >= virtual_address && rva < virtual_address.saturating_add(span) {
            let offset = raw_offset.saturating_add(rva - virtual_address) as usize;
            if offset < bytes.len() {
                return Ok(offset);
            }
        }
    }
    bail!("PE RVA {rva:#x} is not mapped by a section")
}

pub fn pe_imports(bytes: &[u8]) -> Result<Vec<String>> {
    if bytes.get(0..2) != Some(b"MZ") {
        bail!("not a PE file: missing MZ header");
    }
    let pe_offset = read_u32(bytes, 0x3c)? as usize;
    if bytes.get(pe_offset..pe_offset + 4) != Some(b"PE\0\0") {
        bail!("not a PE file: missing PE signature");
    }
    let coff = pe_offset + 4;
    if read_u16(bytes, coff)? != 0x8664 {
        bail!("expected AMD64 PE");
    }
    let sections = read_u16(bytes, coff + 2)?;
    let optional_size = read_u16(bytes, coff + 16)? as usize;
    let optional = coff + 20;
    if read_u16(bytes, optional)? != 0x20b {
        bail!("expected PE32+ optional header");
    }
    let section_table = optional + optional_size;
    let import_rva = read_u32(bytes, optional + 120)?;
    let import_size = read_u32(bytes, optional + 124)? as usize;
    if import_rva == 0 || import_size == 0 {
        return Ok(Vec::new());
    }

    let import_offset = rva_to_offset(bytes, section_table, sections, import_rva)?;
    let mut imports = Vec::new();
    for descriptor_index in 0..=(import_size / 20) {
        let descriptor = import_offset + descriptor_index * 20;
        let fields = bytes
            .get(descriptor..descriptor + 20)
            .ok_or_else(|| anyhow::anyhow!("truncated PE import descriptor"))?;
        if fields.iter().all(|byte| *byte == 0) {
            break;
        }
        let name_rva = read_u32(bytes, descriptor + 12)?;
        let name_offset = rva_to_offset(bytes, section_table, sections, name_rva)?;
        let tail = bytes
            .get(name_offset..)
            .ok_or_else(|| anyhow::anyhow!("invalid PE import name offset"))?;
        let length = tail
            .iter()
            .position(|byte| *byte == 0)
            .ok_or_else(|| anyhow::anyhow!("unterminated PE import name"))?;
        let name = std::str::from_utf8(&tail[..length])?;
        imports.push(name.to_ascii_lowercase());
    }
    Ok(imports)
}

pub trait DependencyInspector {
    fn imports(&self, bytes: &[u8]) -> Result<Vec<String>>;
    fn is_system_library(&self, name: &str) -> bool;
}

pub struct WindowsPeInspector;

impl DependencyInspector for WindowsPeInspector {
    fn imports(&self, bytes: &[u8]) -> Result<Vec<String>> {
        pe_imports(bytes)
    }

    fn is_system_library(&self, name: &str) -> bool {
        is_windows_system_dll(name)
    }
}

pub fn dependency_closure(roots: &[PathBuf], search_dirs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    dependency_closure_with(&WindowsPeInspector, roots, search_dirs)
}

pub fn dependency_closure_with(
    inspector: &dyn DependencyInspector,
    roots: &[PathBuf],
    search_dirs: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let mut available = HashMap::<String, PathBuf>::new();
    for directory in search_dirs {
        for entry in std::fs::read_dir(directory)? {
            let path = entry?.path();
            if path.is_file()
                && let Some(name) = path.file_name().and_then(|name| name.to_str())
            {
                available.insert(name.to_ascii_lowercase(), path);
            }
        }
    }
    for root in roots {
        if let Some(name) = root.file_name().and_then(|name| name.to_str()) {
            available.insert(name.to_ascii_lowercase(), root.clone());
        }
    }

    let mut queue = VecDeque::from_iter(roots.iter().cloned());
    let mut visited = HashSet::<PathBuf>::new();
    let mut result = Vec::new();
    while let Some(path) = queue.pop_front() {
        let identity = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if !visited.insert(identity) {
            continue;
        }
        let bytes = std::fs::read(&path)?;
        for import in inspector.imports(&bytes)? {
            if inspector.is_system_library(&import) {
                continue;
            }
            let dependency = available.get(&import).ok_or_else(|| {
                anyhow::anyhow!(
                    "{} imports missing non-system dependency {import}",
                    path.display()
                )
            })?;
            queue.push_back(dependency.clone());
        }
        result.push(path);
    }
    Ok(result)
}

#[derive(Debug, Serialize)]
pub struct PackageFile {
    pub path: String,
    pub source: String,
    pub size: u64,
    pub sha256: String,
    pub imports: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PackageReport {
    pub target: String,
    pub total_size: u64,
    pub files: Vec<PackageFile>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PackageOptions {
    pub gst_root: PathBuf,
    pub app: PathBuf,
    pub output: PathBuf,
}

impl PackageOptions {
    pub fn parse(args: &[String]) -> Result<Self> {
        if args.first().map(String::as_str) != Some("package") {
            bail!("expected `package` command");
        }
        let mut gst_root = None;
        let mut app = None;
        let mut output = None;
        let mut index = 1;
        while index < args.len() {
            let flag = args[index].as_str();
            let value = args
                .get(index + 1)
                .ok_or_else(|| anyhow::anyhow!("{flag} requires a path"))?;
            match flag {
                "--gst-root" => gst_root = Some(PathBuf::from(value)),
                "--app" => app = Some(PathBuf::from(value)),
                "--output" => output = Some(PathBuf::from(value)),
                _ => bail!("unknown package option {flag}"),
            }
            index += 2;
        }
        Ok(Self {
            gst_root: gst_root.ok_or_else(|| anyhow::anyhow!("missing --gst-root"))?,
            app: app.ok_or_else(|| anyhow::anyhow!("missing --app"))?,
            output: output.ok_or_else(|| anyhow::anyhow!("missing --output"))?,
        })
    }
}

fn find_case_insensitive(directory: &Path, wanted: &str) -> Result<PathBuf> {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(wanted))
        {
            return Ok(path);
        }
    }
    bail!("{} not found in {}", wanted, directory.display())
}

pub fn stage_runtime(
    manifest: &RuntimeManifest,
    app: &Path,
    gst_root: &Path,
    platform_runtime_dirs: &[PathBuf],
    output: &Path,
) -> Result<PackageReport> {
    if output.exists() {
        bail!("output directory already exists: {}", output.display());
    }
    let bin = gst_root.join("bin");
    let plugin_dir = gst_root.join("lib").join("gstreamer-1.0");
    let mut roots = Vec::new();
    let mut runtime_module_dirs = Vec::new();
    let mut runtime_data_files = Vec::new();
    for dll in &manifest.core_dlls {
        roots.push(find_case_insensitive(&bin, dll)?);
    }
    for dll in &manifest.platform_dlls {
        let path = platform_runtime_dirs
            .iter()
            .find_map(|directory| find_case_insensitive(directory, dll).ok())
            .ok_or_else(|| anyhow::anyhow!("{dll} not found in VC Runtime directories"))?;
        roots.push(path);
    }
    for group in &manifest.plugin_groups {
        for plugin in &group.plugins {
            roots.push(find_case_insensitive(&plugin_dir, plugin)?);
        }
    }
    for group in &manifest.runtime_module_groups {
        let source_dir = gst_root.join(&group.source_subdir);
        for file in &group.files {
            roots.push(find_case_insensitive(&source_dir, file)?);
        }
        runtime_module_dirs.push((
            std::fs::canonicalize(&source_dir).unwrap_or(source_dir.clone()),
            group.destination_subdir.clone(),
        ));
    }
    for group in &manifest.runtime_data_groups {
        let source_dir = gst_root.join(&group.source_subdir);
        for file in &group.files {
            runtime_data_files.push((
                find_case_insensitive(&source_dir, file)?,
                group.destination_subdir.join(file),
            ));
        }
    }
    let mut search_dirs = vec![bin.clone(), plugin_dir.clone()];
    search_dirs.extend(
        runtime_module_dirs
            .iter()
            .map(|(source_dir, _)| source_dir.clone()),
    );
    search_dirs.extend_from_slice(platform_runtime_dirs);
    let closure = dependency_closure(&roots, &search_dirs)?;

    std::fs::create_dir_all(output.join("gst-plugins"))?;
    for (_, destination_subdir) in &runtime_module_dirs {
        std::fs::create_dir_all(output.join(destination_subdir))?;
    }
    for (_, destination) in &runtime_data_files {
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(output.join(parent))?;
        }
    }
    let canonical_plugins =
        std::fs::canonicalize(&plugin_dir).unwrap_or_else(|_| plugin_dir.clone());
    let app_name = app
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("application path has no file name"))?;
    let app_destination = output.join(app_name);
    std::fs::copy(app, &app_destination)?;
    let app_bytes = std::fs::read(&app_destination)?;
    let mut total_size = app_bytes.len() as u64;
    let mut files = vec![PackageFile {
        path: app_name.to_string_lossy().into_owned(),
        source: app.display().to_string(),
        size: app_bytes.len() as u64,
        sha256: format!("{:x}", Sha256::digest(&app_bytes)),
        imports: pe_imports(&app_bytes)?,
    }];
    for source in closure {
        let source_parent = source
            .parent()
            .and_then(|parent| std::fs::canonicalize(parent).ok());
        let is_plugin = source_parent.as_ref() == Some(&canonical_plugins);
        let file_name = source
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("runtime source has no file name"))?;
        let module_destination = source_parent.as_ref().and_then(|parent| {
            runtime_module_dirs
                .iter()
                .find(|(source_dir, _)| source_dir == parent)
                .map(|(_, destination)| destination)
        });
        let relative = if is_plugin {
            PathBuf::from("gst-plugins").join(file_name)
        } else if let Some(destination) = module_destination {
            destination.join(file_name)
        } else {
            PathBuf::from(file_name)
        };
        let destination = output.join(&relative);
        std::fs::copy(&source, &destination)?;
        let bytes = std::fs::read(&destination)?;
        let size = bytes.len() as u64;
        total_size = total_size.saturating_add(size);
        files.push(PackageFile {
            path: relative.to_string_lossy().replace('\\', "/"),
            source: source.display().to_string(),
            size,
            sha256: format!("{:x}", Sha256::digest(&bytes)),
            imports: pe_imports(&bytes)?,
        });
    }
    for (source, relative) in runtime_data_files {
        let destination = output.join(&relative);
        std::fs::copy(&source, &destination)?;
        let bytes = std::fs::read(&destination)?;
        let size = bytes.len() as u64;
        total_size = total_size.saturating_add(size);
        files.push(PackageFile {
            path: relative.to_string_lossy().replace('\\', "/"),
            source: source.display().to_string(),
            size,
            sha256: format!("{:x}", Sha256::digest(&bytes)),
            imports: Vec::new(),
        });
    }
    let max_size = manifest.max_size_mib.saturating_mul(1024 * 1024);
    if total_size > max_size {
        bail!(
            "runtime is {:.2} MiB, exceeding the {} MiB limit",
            total_size as f64 / 1024.0 / 1024.0,
            manifest.max_size_mib
        );
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let report = PackageReport {
        target: manifest.target.clone(),
        total_size,
        files,
    };
    std::fs::write(
        output.join("gst-runtime-manifest.json"),
        serde_json::to_vec_pretty(manifest)?,
    )?;
    std::fs::write(
        output.join("runtime-report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(report)
}

pub fn discover_vc_runtime() -> Result<PathBuf> {
    if let Some(root) = std::env::var_os("VCToolsRedistDir") {
        if let Some(candidate) = latest_vc_runtime(PathBuf::from(root).join("x64"))? {
            return Ok(candidate);
        }
    }
    let mut candidates = Vec::new();
    for program_files in ["ProgramFiles", "ProgramFiles(x86)"]
        .into_iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
    {
        let visual_studio = program_files.join("Microsoft Visual Studio");
        if !visual_studio.is_dir() {
            continue;
        }
        for release in std::fs::read_dir(visual_studio)? {
            let release = release?.path();
            if !release.is_dir() {
                continue;
            }
            for edition in std::fs::read_dir(release)? {
                let versions = edition?.path().join("VC").join("Redist").join("MSVC");
                if !versions.is_dir() {
                    continue;
                }
                for version in std::fs::read_dir(versions)? {
                    let x64 = version?.path().join("x64");
                    if let Some(candidate) = latest_vc_runtime(x64)? {
                        candidates.push(candidate);
                    }
                }
            }
        }
    }
    candidates.sort();
    candidates
        .pop()
        .ok_or_else(|| anyhow::anyhow!("Microsoft Visual C++ x64 runtime directory not found"))
}

fn latest_vc_runtime(x64: PathBuf) -> Result<Option<PathBuf>> {
    if !x64.is_dir() {
        return Ok(None);
    }
    let mut candidates = std::fs::read_dir(x64)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("Microsoft.VC") && name.ends_with(".CRT"))
        })
        .collect::<Vec<_>>();
    candidates.sort();
    Ok(candidates.pop())
}

#[cfg(test)]
mod tests {
    use super::{
        PackageOptions, RuntimeManifest, dependency_closure, is_windows_system_dll, pe_imports,
        stage_runtime,
    };
    use std::path::PathBuf;

    fn minimal_pe64_with_imports(names: &[&str]) -> Vec<u8> {
        let mut bytes = vec![0_u8; 0x600];
        bytes[0..2].copy_from_slice(b"MZ");
        bytes[0x3c..0x40].copy_from_slice(&(0x80_u32).to_le_bytes());
        bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
        bytes[0x84..0x86].copy_from_slice(&0x8664_u16.to_le_bytes());
        bytes[0x86..0x88].copy_from_slice(&1_u16.to_le_bytes());
        bytes[0x94..0x96].copy_from_slice(&0xf0_u16.to_le_bytes());

        let optional = 0x98;
        bytes[optional..optional + 2].copy_from_slice(&0x20b_u16.to_le_bytes());
        bytes[optional + 108..optional + 112].copy_from_slice(&16_u32.to_le_bytes());
        bytes[optional + 120..optional + 124].copy_from_slice(&0x1000_u32.to_le_bytes());
        bytes[optional + 124..optional + 128]
            .copy_from_slice(&((names.len() as u32 + 1) * 20).to_le_bytes());

        let section = optional + 0xf0;
        bytes[section..section + 8].copy_from_slice(b".rdata\0\0");
        bytes[section + 8..section + 12].copy_from_slice(&0x400_u32.to_le_bytes());
        bytes[section + 12..section + 16].copy_from_slice(&0x1000_u32.to_le_bytes());
        bytes[section + 16..section + 20].copy_from_slice(&0x400_u32.to_le_bytes());
        bytes[section + 20..section + 24].copy_from_slice(&0x200_u32.to_le_bytes());

        for (index, name) in names.iter().enumerate() {
            let descriptor = 0x200 + index * 20;
            let name_offset = 0x300 + index * 0x40;
            let name_rva = 0x1000 + (name_offset - 0x200) as u32;
            bytes[descriptor + 12..descriptor + 16].copy_from_slice(&name_rva.to_le_bytes());
            bytes[name_offset..name_offset + name.len()].copy_from_slice(name.as_bytes());
        }
        bytes
    }

    #[test]
    fn system_dll_filter_does_not_hide_third_party_dependencies() {
        assert!(is_windows_system_dll("KERNEL32.dll"));
        assert!(is_windows_system_dll("combase.dll"));
        assert!(is_windows_system_dll("icuuc.dll"));
        assert!(is_windows_system_dll("UIAutomationCore.dll"));
        assert!(is_windows_system_dll("bcryptprimitives.dll"));
        assert!(!is_windows_system_dll("vcruntime140.dll"));
        assert!(!is_windows_system_dll("vcruntime140_1.dll"));
        assert!(!is_windows_system_dll("msvcp140.dll"));
        assert!(is_windows_system_dll("ucrtbase.dll"));
        assert!(is_windows_system_dll("api-ms-win-core-file-l1-1-0.dll"));
        assert!(!is_windows_system_dll("gstreamer-1.0-0.dll"));
        assert!(!is_windows_system_dll("avcodec-61.dll"));
        for dll in [
            "dcomp.dll",
            "d2d1.dll",
            "dwrite.dll",
            "dxcore.dll",
            "shcore.dll",
            "windowscodecs.dll",
            "gdiplus.dll",
            "usp10.dll",
            "msctf.dll",
            "wintrust.dll",
            "cryptbase.dll",
            "powrprof.dll",
            "profapi.dll",
            "msimg32.dll",
            "uxtheme.dll",
            "hid.dll",
            "oleacc.dll",
            "avrt.dll",
            "winspool.drv",
            "netapi32.dll",
            "wtsapi32.dll",
        ] {
            assert!(is_windows_system_dll(dll), "{dll} is a Windows SDK library");
        }
    }

    #[test]
    fn pe_import_parser_reads_amd64_import_descriptors() {
        let bytes = minimal_pe64_with_imports(&["KERNEL32.dll", "gstreamer-1.0-0.dll"]);

        let imports = pe_imports(&bytes).expect("the synthetic AMD64 PE is valid");

        assert_eq!(imports, vec!["kernel32.dll", "gstreamer-1.0-0.dll"]);
    }

    #[test]
    fn dependency_closure_follows_transitive_non_system_dlls() {
        let temp = tempfile::tempdir().expect("temp directory");
        let app = temp.path().join("gui.exe");
        let plugin = temp.path().join("gstplayback.dll");
        let codec = temp.path().join("avcodec-61.dll");
        std::fs::write(
            &app,
            minimal_pe64_with_imports(&["KERNEL32.dll", "gstplayback.dll"]),
        )
        .unwrap();
        std::fs::write(
            &plugin,
            minimal_pe64_with_imports(&["avcodec-61.dll", "gui.exe"]),
        )
        .unwrap();
        std::fs::write(&codec, minimal_pe64_with_imports(&[])).unwrap();

        let closure = dependency_closure(std::slice::from_ref(&app), &[temp.path().into()])
            .expect("all third-party imports exist");
        let mut names = closure
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        names.sort();

        assert_eq!(names, vec!["avcodec-61.dll", "gstplayback.dll", "gui.exe"]);
    }

    #[test]
    fn staging_separates_plugins_from_runtime_dependencies() {
        let temp = tempfile::tempdir().unwrap();
        let gst_root = temp.path().join("gst");
        let bin = gst_root.join("bin");
        let plugins = gst_root.join("lib").join("gstreamer-1.0");
        let vc_runtime = temp.path().join("vc-runtime");
        let output = temp.path().join("dist");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::create_dir_all(&plugins).unwrap();
        std::fs::create_dir_all(&vc_runtime).unwrap();

        let app = temp.path().join("gui.exe");
        std::fs::write(&app, minimal_pe64_with_imports(&["application-only.dll"])).unwrap();
        std::fs::write(
            bin.join("gstreamer-1.0-0.dll"),
            minimal_pe64_with_imports(&["vcruntime140.dll"]),
        )
        .unwrap();
        std::fs::write(
            vc_runtime.join("vcruntime140.dll"),
            minimal_pe64_with_imports(&[]),
        )
        .unwrap();
        std::fs::write(
            plugins.join("gstplayback.dll"),
            minimal_pe64_with_imports(&["avcodec-61.dll"]),
        )
        .unwrap();
        std::fs::write(bin.join("avcodec-61.dll"), minimal_pe64_with_imports(&[])).unwrap();

        let manifest = RuntimeManifest::from_toml(
            r#"
schema = 1
extractor = "windows"
target = "x86_64-pc-windows-msvc"
max_size_mib = 250
core_dlls = ["gstreamer-1.0-0.dll"]
platform_dlls = ["vcruntime140.dll"]
required_features = ["playbin"]

[[plugin_groups]]
name = "core"
plugins = ["gstplayback.dll"]
"#,
        )
        .unwrap();

        let report = stage_runtime(&manifest, &app, &gst_root, &[vc_runtime], &output).unwrap();

        assert!(output.join("gui.exe").is_file());
        assert!(output.join("gstreamer-1.0-0.dll").is_file());
        assert!(output.join("avcodec-61.dll").is_file());
        assert!(output.join("vcruntime140.dll").is_file());
        assert!(output.join("gst-plugins").join("gstplayback.dll").is_file());
        assert_eq!(report.files.len(), 5);
    }

    #[test]
    fn staging_copies_dynamic_modules_and_their_transitive_dependencies() {
        let temp = tempfile::tempdir().unwrap();
        let gst_root = temp.path().join("gst");
        let bin = gst_root.join("bin");
        let plugins = gst_root.join("lib").join("gstreamer-1.0");
        let gio_modules = gst_root.join("lib").join("gio").join("modules");
        let output = temp.path().join("dist");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::create_dir_all(&plugins).unwrap();
        std::fs::create_dir_all(&gio_modules).unwrap();

        let app = temp.path().join("gui.exe");
        std::fs::write(&app, minimal_pe64_with_imports(&[])).unwrap();
        std::fs::write(
            bin.join("gstreamer-1.0-0.dll"),
            minimal_pe64_with_imports(&[]),
        )
        .unwrap();
        std::fs::write(
            gio_modules.join("gioopenssl.dll"),
            minimal_pe64_with_imports(&["libssl-3-x64.dll"]),
        )
        .unwrap();
        std::fs::write(bin.join("libssl-3-x64.dll"), minimal_pe64_with_imports(&[])).unwrap();

        let manifest = RuntimeManifest::from_toml(
            r#"
schema = 1
extractor = "windows"
target = "x86_64-pc-windows-msvc"
max_size_mib = 250
core_dlls = ["gstreamer-1.0-0.dll"]
required_features = []
plugin_groups = []

[[runtime_module_groups]]
name = "gio-tls"
source_subdir = "lib/gio/modules"
destination_subdir = "gio-modules"
files = ["gioopenssl.dll"]
"#,
        )
        .unwrap();

        let report = stage_runtime(&manifest, &app, &gst_root, &[], &output).unwrap();

        assert!(output.join("gio-modules").join("gioopenssl.dll").is_file());
        assert!(output.join("libssl-3-x64.dll").is_file());
        assert!(
            report
                .files
                .iter()
                .any(|file| file.path == "gio-modules/gioopenssl.dll")
        );
    }

    #[test]
    fn package_options_require_explicit_source_and_destination() {
        let args = [
            "package",
            "--gst-root",
            r"C:\gstreamer\1.0\msvc_x86_64",
            "--app",
            r"target\release\gui.exe",
            "--output",
            r"dist\windows-x86_64",
        ]
        .map(str::to_string);

        let options = PackageOptions::parse(&args).unwrap();

        assert_eq!(
            options.gst_root,
            PathBuf::from(r"C:\gstreamer\1.0\msvc_x86_64")
        );
        assert_eq!(options.app, PathBuf::from(r"target\release\gui.exe"));
        assert_eq!(options.output, PathBuf::from(r"dist\windows-x86_64"));
    }

    #[test]
    fn manifest_rejects_platform_without_a_dependency_inspector() {
        let source = r#"
schema = 1
extractor = "linux"
target = "x86_64-unknown-linux-gnu"
max_size_mib = 250
core_dlls = []
required_features = []
plugin_groups = []
"#;

        let error = RuntimeManifest::from_toml(source).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("unsupported runtime extractor linux")
        );
    }
}
