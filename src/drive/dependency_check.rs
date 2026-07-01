use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use futures_util::StreamExt;
use gpui::{
    AppContext, Context, IntoElement, ParentElement, Render, Styled, Task, Window, div, px, rgb,
};
use gpui_component::button::Button;
use gpui_component::{Disableable, Root, h_flex, v_flex};
use tokio::io::AsyncWriteExt;

use crate::com::window_center_options;
use crate::component::home::rgb_to_u32;
use crate::drive::video_player::VideoPlayer;
const WINDOWS_INSTALL_PATH:&str = "C:/Users/10463/AppData/Local/Programs/gstreamer/1.0/msvc_x86_64/bin";
const WINDOWS_GSTREAMER_URL: &str = "https://gstreamer.freedesktop.org/data/pkg/windows/1.28.4/msvc/gstreamer-1.0-msvc-x86_64-1.28.4.exe";
const WINDOWS_INSTALLER_FILE: &str = "gstreamer-1.0-msvc-x86_64-1.28.4.exe";

#[derive(Clone, Debug, PartialEq, Eq)]
enum DependencyStatus {
    Checking,
    Installed,
    Missing,
    Downloading,
    Downloaded(PathBuf),
    Unsupported(&'static str),
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DependencyInstallPlan {
    os: &'static str,
    url: Option<&'static str>,
    file_name: Option<&'static str>,
    unsupported_message: Option<&'static str>,
}

pub struct DownloadDependency {
    status: DependencyStatus,
    task: Option<Task<()>>,
}

impl DownloadDependency {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut dependency = Self {
            status: DependencyStatus::Checking,
            task: None,
        };
        dependency.check_gstreamer(window, cx);
        dependency
    }

    fn open_window(&self, window: &mut Window, cx: &mut Context<Self>) {
        cx.open_window(
            window_center_options(window, 1300., 700.),
            move |window, app| {
                let view = app.new(|cx| VideoPlayer::new(window, cx));
                app.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("open window failed");
    }

    fn check_gstreamer(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.status = DependencyStatus::Checking;
        self.task = Some(cx.spawn(async move |this, cx| {
            let installed = cx
                .background_executor()
                .spawn(async move { is_gstreamer_installed() })
                .await;

            let _ = this.update(cx, |this, cx| {
                this.status = if installed {
                    DependencyStatus::Installed
                } else {
                    let plan = dependency_plan_for_os(env::consts::OS);
                    if let Some(message) = plan.unsupported_message {
                        DependencyStatus::Unsupported(message)
                    } else {
                        DependencyStatus::Missing
                    }
                };
                this.task = None;
                cx.notify();
            });
        }));
    }

    fn download_gstreamer(&mut self, cx: &mut Context<Self>) {
        let plan = dependency_plan_for_os(env::consts::OS);
        let (Some(url), Some(file_name)) = (plan.url, plan.file_name) else {
            self.status = DependencyStatus::Unsupported(
                plan.unsupported_message
                    .unwrap_or("当前平台暂未配置 GStreamer 自动下载"),
            );
            cx.notify();
            return;
        };

        self.status = DependencyStatus::Downloading;
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = download_installer(url, file_name).await;
            let _ = this.update(cx, |this, cx| {
                this.status = match result {
                    Ok(path) => DependencyStatus::Downloaded(path),
                    Err(err) => DependencyStatus::Error(format!("下载 GStreamer 失败: {err}")),
                };
                this.task = None;
                cx.notify();
            });
        }));
    }

    fn run_installer(&mut self, installer: &Path, cx: &mut Context<Self>) {
        match Command::new(installer).spawn() {
            Ok(_) => {
                self.status = DependencyStatus::Missing;
            }
            Err(err) => {
                self.status = DependencyStatus::Error(format!("启动安装程序失败: {err}"));
            }
        }
        cx.notify();
    }
}

impl Render for DownloadDependency {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.status.clone();
        let can_download = matches!(
            status,
            DependencyStatus::Missing | DependencyStatus::Error(_)
        );
        let can_open_video = matches!(status, DependencyStatus::Installed);

        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_4()
            .p_8()
            .bg(rgb(0xF8FAFC))
            .child(
                v_flex()
                    .w(px(520.))
                    .gap_4()
                    .p_6()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(0xCBD5E1))
                    .bg(rgb(0xFFFFFF))
                    .child(
                        div()
                            .text_size(px(22.))
                            .text_color(rgb_to_u32(15, 23, 42))
                            .child("GStreamer 环境检测"),
                    )
                    .child(
                        div()
                            .text_size(px(14.))
                            .text_color(rgb_to_u32(71, 85, 105))
                            .child(status_message(&status)),
                    )
                    .child(
                        h_flex()
                            .gap_3()
                            .child(
                                Button::new("gst-check-again")
                                    .label("重新检测")
                                    .outline()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.check_gstreamer(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("gst-download")
                                    .label("下载 Windows 安装包")
                                    .disabled(!can_download)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.download_gstreamer(cx);
                                    })),
                            )
                            .child(match status {
                                DependencyStatus::Downloaded(path) => {
                                    Button::new("gst-run-installer")
                                        .label("打开安装包")
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.run_installer(&path, cx);
                                        }))
                                        .into_any_element()
                                }
                                _ => Button::new("gst-run-installer-disabled")
                                    .label("打开安装包")
                                    .disabled(true)
                                    .into_any_element(),
                            })
                            .child(
                                Button::new("open-video-player")
                                    .label("打开播放器")
                                    .disabled(!can_open_video)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_window(window, cx);
                                    })),
                            ),
                    ),
            )
    }
}

fn status_message(status: &DependencyStatus) -> String {
    match status {
        DependencyStatus::Checking => "正在检测本机是否已安装 GStreamer...".to_string(),
        DependencyStatus::Installed => "已检测到 GStreamer，可以打开播放器。".to_string(),
        DependencyStatus::Missing => {
            "未检测到 GStreamer。Windows 将下载官方 1.28.4 MSVC x86_64 安装包。".to_string()
        }
        DependencyStatus::Downloading => "正在下载 GStreamer Windows 安装包...".to_string(),
        DependencyStatus::Downloaded(path) => format!("下载完成: {}", path.display()),
        DependencyStatus::Unsupported(message) => message.to_string(),
        DependencyStatus::Error(message) => message.clone(),
    }
}

fn dependency_plan_for_os(os: &'static str) -> DependencyInstallPlan {
    match os {
        "windows" => DependencyInstallPlan {
            os,
            url: Some(WINDOWS_GSTREAMER_URL),
            file_name: Some(WINDOWS_INSTALLER_FILE),
            unsupported_message: None,
        },
        "linux" => DependencyInstallPlan {
            os,
            url: None,
            file_name: None,
            unsupported_message: Some("Linux 暂未配置 GStreamer 自动下载，请先手动安装。"),
        },
        "macos" => DependencyInstallPlan {
            os,
            url: None,
            file_name: None,
            unsupported_message: Some("macOS 暂未配置 GStreamer 自动下载，请先手动安装。"),
        },
        _ => DependencyInstallPlan {
            os,
            url: None,
            file_name: None,
            unsupported_message: Some("当前平台暂未配置 GStreamer 自动下载。"),
        },
    }
}

fn is_gstreamer_installed() -> bool {
    has_gstreamer_env() || command_exists("gst-launch-1.0")
}

fn has_gstreamer_env() -> bool {
    let vars = [
        "GSTREAMER_1_0_ROOT_MSVC_X86_64",
        "GSTREAMER_1_0_ROOT_MINGW_X86_64",
        "GSTREAMER_ROOT",
    ];

    vars.iter().any(|name| {
        env::var_os(name)
            .map(PathBuf::from)
            .is_some_and(|path| is_gstreamer_root(&path))
    })
}

fn is_gstreamer_root(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    if cfg!(windows) {
        return path.join("bin").join("gst-launch-1.0.exe").is_file()
            || path.join("bin").join("gstreamer-1.0-0.dll").is_file();
    }

    path.join("bin").join("gst-launch-1.0").is_file()
}

fn command_exists(command: &str) -> bool {
    Command::new(command).arg("--version").output().is_ok()
}

async fn download_installer(url: &str, file_name: &str) -> anyhow::Result<PathBuf> {
    let path = env::temp_dir().join(file_name);
    if path.exists() {
        return Ok(path);
    }

    let response = reqwest::get(url).await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("HTTP {}", response.status()));
    }

    let mut file = tokio::fs::File::create(&path).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk?).await?;
    }
    file.flush().await?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_plan_uses_requested_gstreamer_installer() {
        let plan = dependency_plan_for_os("windows");

        assert_eq!(plan.url, Some(WINDOWS_GSTREAMER_URL));
        assert_eq!(plan.file_name, Some(WINDOWS_INSTALLER_FILE));
        assert_eq!(plan.unsupported_message, None);
    }

    #[test]
    fn linux_and_macos_are_placeholder_plans() {
        for os in ["linux", "macos"] {
            let plan = dependency_plan_for_os(os);

            assert_eq!(plan.url, None);
            assert_eq!(plan.file_name, None);
            assert!(plan.unsupported_message.is_some());
        }
    }

    #[test]
    fn status_text_mentions_downloaded_installer_path() {
        let path = PathBuf::from("C:\\Temp\\gstreamer.exe");
        let message = status_message(&DependencyStatus::Downloaded(path.clone()));

        assert!(message.contains(&path.display().to_string()));
    }

    #[test]
    fn gstreamer_root_requires_platform_binary() {
        let root = env::temp_dir().join(format!("gpui-medio-gst-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("bin")).unwrap();

        assert!(!is_gstreamer_root(&root));

        let marker = if cfg!(windows) {
            root.join("bin").join("gst-launch-1.0.exe")
        } else {
            root.join("bin").join("gst-launch-1.0")
        };
        std::fs::write(&marker, "").unwrap();

        assert!(is_gstreamer_root(&root));

        std::fs::remove_dir_all(root).unwrap();
    }
}
