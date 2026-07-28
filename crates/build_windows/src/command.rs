use anyhow::{Result, bail};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Doctor,
    Package,
    Support,
    Verify,
}

#[derive(Debug)]
pub struct Options {
    pub action: Action,
    pub gst_root: Option<PathBuf>,
    pub app: Option<PathBuf>,
    pub output: PathBuf,
    pub all: bool,
    pub force: bool,
}

impl Options {
    pub fn parse() -> Result<Self> {
        let project_root = project_root();
        let mut args = std::env::args().skip(1);
        let action = match args.next().as_deref() {
            Some("doctor") => Action::Doctor,
            Some("package" | "all") => Action::Package,
            Some("support") => Action::Support,
            Some("verify") => Action::Verify,
            Some("-h" | "--help") => {
                print_usage();
                std::process::exit(0);
            }
            Some(value) => bail!("unknown command {value}"),
            None => Action::Package,
        };
        let mut options = Self {
            action,
            gst_root: None,
            app: None,
            output: project_root.join("target").join("build_windows"),
            all: false,
            force: false,
        };
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--gst-root" => options.gst_root = Some(next_path(&mut args, &flag)?),
                "--app" => options.app = Some(next_path(&mut args, &flag)?),
                "--output" => options.output = next_path(&mut args, &flag)?,
                "--all" => options.all = true,
                "--force" => options.force = true,
                _ => bail!("unknown option {flag}"),
            }
        }
        Ok(options)
    }
}

pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("build_windows must be located under crates/")
        .to_path_buf()
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("{flag} requires a path"))
}

fn print_usage() {
    println!(
        "cargo run --release -p build_windows -- package [--force] \
         [--gst-root PATH] [--app PATH] [--output PATH]\n\
         cargo run --release -p build_windows -- support [--all] [--output PATH]"
    );
}
