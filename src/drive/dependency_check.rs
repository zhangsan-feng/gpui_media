#![windows_subsystem = "windows"]

use futures_util::StreamExt;
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Task, Window, div, px,
    relative, rgb, size,
};
use gpui_component::input::{Input, InputState};
use gpui_component::{Root, v_flex};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

const GUI_EXE: &str = "gui.exe";
// https://gstreamer.freedesktop.org/download
const WINDOWS_GSTREAMER_URL: &str = "https://gstreamer.freedesktop.org/data/pkg/windows/1.28.4/msvc/gstreamer-1.0-msvc-x86_64-1.28.4.exe";
const WINDOWS_INSTALLER_FILE: &str = "gstreamer-1.0-msvc-x86_64-1.28.4.exe";
const WINDOWS_GSTREAMER_LOCAL_BIN: &[&str] =
    &["Programs", "gstreamer", "1.0", "msvc_x86_64", "bin"];

pub struct DownloadDependency {
    progress: f32,
    message: String,
    download_path: String,
    download_path_input: Entity<InputState>,
    task: Option<Task<()>>,
}

impl DownloadDependency {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let download_path_input = cx.new(|cx| InputState::new(window, cx));
        let mut dependency = Self {
            progress: 0.0,
            message: "Checking GStreamer...".to_string(),
            download_path: "".to_string(),
            download_path_input,
            task: None,
        };
        dependency.install_dependency(window, cx);
        dependency
    }

    pub fn install_dependency(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.task.is_some() {
            return;
        }

        #[cfg(windows)]
        {
            let path = env::temp_dir().join(WINDOWS_INSTALLER_FILE);
            // self.message = format!("Downloading GStreamer to {}", path.display());
            self.download_path = path.display().to_string();
            self.download_path_input.update(cx, |input, cx| {
                input.set_value(self.download_path.clone(), window, cx);
            });
            self.task = Some(cx.spawn(async move |this, cx| {
                let (tx, mut rx) = mpsc::unbounded_channel();
                let download_path = path.clone();
                let mut download = tokio::spawn(async move {
                    Self::download_file(WINDOWS_GSTREAMER_URL, &download_path, |progress, message| {
                        let _ = tx.send((progress, message.to_string()));
                    })
                    .await
                });

                let message = loop {
                    tokio::select! {
                        update = rx.recv() => {
                            if let Some((progress, message)) = update {
                                let _ = this.update(cx, |this, cx| {
                                    this.progress = progress;
                                    this.message = message;
                                    cx.notify();
                                });
                            }
                        }
                        result = &mut download => {
                            break match result {
                                Ok(Ok(())) => {
                                    let _ = this.update(cx, |this, cx| {
                                        this.message = "Installing GStreamer...".to_string();
                                        cx.notify();
                                    });
                                    let status = cx
                                        .background_executor()
                                        .spawn(async move { Command::new(&path).arg("/S").status() })
                                        .await;
                                    match status {
                                        Ok(status) if status.success() => {
                                            if check_windows() && launch_core_gui() {
                                                std::process::exit(0);
                                            }
                                            "Install finished, but GUI did not start.".to_string()
                                        }
                                        Ok(status) => {
                                            format!("Install failed: installer exit code {status}")
                                        }
                                        Err(err) => format!("Install failed: {err}"),
                                    }
                                }
                                Ok(Err(err)) => format!("Download failed: {err}"),
                                Err(err) => format!("Download task failed: {err}"),
                            };
                        }
                    }
                };

                let _ = this.update(cx, |this, cx| {
                    this.message = message;
                    this.task = None;
                    cx.notify();
                });
            }));
        }

        #[cfg(not(windows))]
        {
            self.message = "Auto install is only configured for Windows.".to_string();
            cx.notify();
        }
    }

    async fn download_file(
        url: &str,
        path: &Path,
        mut on_update: impl FnMut(f32, &str),
    ) -> anyhow::Result<()> {
        let _ = tokio::fs::remove_file(path).await;

        let mut last_err = None;
        for retry in 1..=3 {
            let message = format!("");
            on_update(0.0, &message);
            match Self::download_once(url, path, &mut on_update).await {
                Ok(()) => return Ok(()),
                Err(err) => {
                    last_err = Some(err);
                    let _ = tokio::fs::remove_file(path).await;
                    if retry == 3 {
                        if let Some(err) = &last_err {
                            log::error!("download dependency failed: {err}");
                        }
                        break;
                    }
                    on_update(0.0, "Download failed, retrying...");
                }
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("download failed")))
    }

    async fn download_once(
        url: &str,
        path: &Path,
        on_update: &mut impl FnMut(f32, &str),
    ) -> anyhow::Result<()> {
        let response = reqwest::get(url).await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP {}", response.status()));
        }

        let total = response.content_length().unwrap_or(0);
        let mut downloaded = 0;
        let started = Instant::now();
        let mut last_update = Instant::now();
        let partial = path.with_extension("exe.part");
        let _ = tokio::fs::remove_file(&partial).await;
        let mut file = tokio::fs::File::create(&partial).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            if total > 0 && last_update.elapsed() >= Duration::from_millis(200) {
                let progress = downloaded as f32 / total as f32;
                let speed = downloaded as f64 / started.elapsed().as_secs_f64().max(0.1);
                let message = format!(
                    "Downloading GStreamer: {:.0}% ({}/s)",
                    progress * 100.0,
                    format_bytes(speed)
                );
                on_update(progress, &message);
                last_update = Instant::now();
            }
        }

        file.flush().await?;
        drop(file);
        tokio::fs::rename(&partial, path).await?;
        let message = format!("Download finished: {}", path.display());
        on_update(1.0, &message);
        Ok(())
    }
}

fn format_bytes(bytes_per_second: f64) -> String {
    if bytes_per_second >= 1024.0 * 1024.0 {
        format!("{:.1} MB", bytes_per_second / 1024.0 / 1024.0)
    } else {
        format!("{:.0} KB", bytes_per_second / 1024.0)
    }
}

impl Render for DownloadDependency {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().size_full().p_8().bg(rgb(0xF8FAFC)).child(
            v_flex()
                .w_full()
                .gap_4()
                .p_4()
                .rounded_md()
                .border_1()
                .border_color(rgb(0xCBD5E1))
                .bg(rgb(0xFFFFFF))
                .child(
                    div()
                        .text_size(px(22.))
                        .text_color(rgb(0x0F172A))
                        .child("GStreamer check"),
                )
                .child(
                    div()
                        .text_size(px(14.))
                        .text_color(rgb(0x475569))
                        .child(self.message.clone()),
                )
                .child(
                    div().w_full().mb_3().child(
                        Input::new(&self.download_path_input)
                            .appearance(false)
                            .disabled(true)
                            .cursor_text()
                            .text_color(rgb(0x94A3B8)),
                    ),
                )
                .child(
                    div()
                        .w_full()
                        .h(px(10.))
                        .rounded_full()
                        .bg(rgb(0xE2E8F0))
                        .child(
                            div()
                                .h(px(10.))
                                .w(relative(self.progress.clamp(0.0, 1.0)))
                                .rounded_full()
                                .bg(rgb(0x2563EB)),
                        ),
                ),
        )
    }
}

#[tokio::main]
async fn main() {
    if check_windows() {
        let _ = launch_core_gui();
        return;
    }

    let mut app = gpui_platform::application();
    app.run(move |cx| {
        let mut window_options = gpui::WindowOptions::default();
        let window_size = size(px(600.), px(280.));
        window_options.window_bounds = Some(gpui::WindowBounds::centered(window_size, cx));
        window_options.window_min_size = Some(window_size);
        window_options.is_resizable = false;
        window_options.titlebar = Some(gpui::TitlebarOptions {
            title: None,
            appears_transparent: false,
            traffic_light_position: None,
        });

        cx.open_window(window_options, |window, app| {
            gpui_component::init(app);
            let view = app.new(|cx| DownloadDependency::new(window, cx));
            app.new(|cx| Root::new(view, window, cx))
        })
        .expect("Failed to create dependency window");
    });
}

fn launch_core_gui() -> bool {
    let gui = exe_dir().join(GUI_EXE);
    let mut command = Command::new(gui);
    if let Some(bin) = windows_gstreamer_bin() {
        let path = env::var_os("PATH").unwrap_or_default();
        let mut paths = env::split_paths(&path).collect::<Vec<_>>();
        if !paths.iter().any(|path| path == &bin) {
            paths.insert(0, bin);
        }
        if let Ok(path) = env::join_paths(paths) {
            command.env("PATH", path);
        }
    }

    command.spawn().is_ok()
}

fn exe_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|path| path.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn windows_gstreamer_bin() -> Option<PathBuf> {
    let mut path = PathBuf::from(env::var_os("LOCALAPPDATA")?);
    path.extend(WINDOWS_GSTREAMER_LOCAL_BIN);
    Some(path)
}

fn gstreamer_windows_to_path() -> bool {
    let Some(bin) = windows_gstreamer_bin() else {
        return false;
    };
    if !bin.join("gst-launch-1.0.exe").is_file() && !bin.join("gstreamer-1.0-0.dll").is_file() {
        return false;
    }

    let path = env::var_os("PATH").unwrap_or_default();
    let mut paths = env::split_paths(&path).collect::<Vec<_>>();
    if !paths.iter().any(|path| path == &bin) {
        paths.insert(0, bin);
        if let Ok(path) = env::join_paths(paths) {
            // ponytail: process-local PATH patch; use a real installer env refresh if this ever races.
            unsafe {
                env::set_var("PATH", path);
            }
        }
    }

    true
}

pub fn check_windows() -> bool {
    let is_install = [
        "GSTREAMER_1_0_ROOT_X86_64",
        "GSTREAMER_1_0_ROOT_MSVC_X86_64",
        "GSTREAMER_1_0_ROOT_MINGW_X86_64",
        "GSTREAMER_ROOT",
    ]
    .iter()
    .filter_map(env::var_os)
    .map(PathBuf::from)
    .any(|root| {
        root.join("bin").join("gst-launch-1.0.exe").is_file()
            || root.join("bin").join("gstreamer-1.0-0.dll").is_file()
    });

    is_install
        || gstreamer_windows_to_path()
        || Command::new("gst-launch-1.0")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
}
