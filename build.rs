fn main() {
    println!("cargo:rerun-if-env-changed=GSTREAMER_1_0_ROOT_MSVC_X86_64");
    println!("cargo:rerun-if-changed=packaging/gstreamer-runtime.toml");
    println!("cargo:rustc-env=GPUI_MEDIO_GSTREAMER_VERSION=1.28.1");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        if let Some(root) = std::env::var_os("GSTREAMER_1_0_ROOT_MSVC_X86_64") {
            let lib = std::path::PathBuf::from(root).join("lib");
            println!("cargo:rustc-link-search=native={}", lib.display());
        }
    }
}
