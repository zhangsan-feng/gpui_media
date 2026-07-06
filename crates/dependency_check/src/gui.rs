use std::path::PathBuf;
use crate::platform::Platform;
use futures_util::StreamExt;
use gpui::*;
use gpui_component::*;
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::v_flex;
use tokio::io::AsyncWriteExt;
use tokio::time::Instant;

pub struct Gui {
    progress: f32,
    message: String,
    save_path: Entity<InputState>,
    platform: Platform,
    task: Option<Task<()>>,
}

impl Gui {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut ctx = Self {
            progress: 0.0,
            message: "Checking GStreamer...".to_string(),
            save_path: cx.new(|cx| InputState::new(window, cx)),
            platform: Platform::new(),
            task: None,
        };

        ctx.check_dependencies(window, cx);
        ctx.check_update(window, cx);
        ctx
    }

    fn format_bytes(&self, bytes_per_second: f64) -> String {
        if bytes_per_second >= 1024.0 * 1024.0 {
            format!("{:.1} MB", bytes_per_second / 1024.0 / 1024.0)
        } else {
            format!("{:.0} KB", bytes_per_second / 1024.0)
        }
    }

    pub fn check_dependencies(&mut self, window: &mut Window, cx: &mut Context<Self>) {

        self.save_path.update(cx, |this, cx| {
            this.set_value(format!("download path :{}", &self.platform.save_path), window, cx);
        });

        let gstreamer_url = self.platform.gstreamer_url.clone();
        let save_path = self.platform.save_path.clone();
        let gstreamer_file = self.platform.gstreamer_file.clone();
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel::<(f32, f64)>();

        self.task = Some(cx.spawn(async move |this, cx| {
            let mut download_task = tokio::spawn(async move {
                Gui::download_file(gstreamer_url, save_path, gstreamer_file, progress_tx).await
            });

            loop {
                tokio::select! {
                    progress = progress_rx.recv() => {
                        if let Some((progress, bytes_per_second)) = progress {
                            let _ = this.update(cx, |this, cx| {
                                this.progress = progress;
                                this.message = format!(
                                    "Downloading GStreamer dependency ... {} /s",
                                    this.format_bytes(bytes_per_second),
                                );
                                cx.notify();
                            });
                        }
                    }
                    result = &mut download_task => {
                        let result = match result {
                            Ok(result) => result,
                            Err(error) => Err(error.into()),
                        };

                        let _ = this.update(cx, |this, cx| {
                            match result {
                                Ok(()) => {
                                    this.progress = 1.0;
                                    this.platform.install_dependencies();
                                    if !this.platform.check_dependencies() {
                                        this.message = "Dependency installation failed".to_string();
                                    } else {
                                        this.platform.start_app();
                                        std::process::exit(0);
                                    }
                                }
                                Err(error) => {
                                    this.message = format!("Download failed: {error}");
                                }
                            }
                            cx.notify();
                        });
                        break;
                    }
                }
            }
        }));
    }

    pub fn check_update(&self, _window: &mut Window, _cx: &mut Context<Self>) {}

    async fn download_file(
        gstreamer_url: String,
        save_path: String,
        gstreamer_file: String,
        progress_tx: tokio::sync::mpsc::UnboundedSender<(f32, f64)>,
    ) -> anyhow::Result<()> {

        let save_path = PathBuf::from(save_path).join(gstreamer_file);

        let response = reqwest::get(gstreamer_url).await?.error_for_status()?;
        let total_size = response.content_length();
        let mut stream = response.bytes_stream();
        let file = tokio::fs::File::create(&save_path).await?;
        let mut writer = tokio::io::BufWriter::new(file);
        let mut download_len = 0u64;
        let mut last_update_time = Instant::now();
        let mut last_downloaded_bytes = 0u64;

        tokio::pin!(stream);
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            let chunk_len = chunk.len() as u64;
            writer.write_all(&chunk).await?;


            download_len += chunk_len;
            let elapsed = last_update_time.elapsed();

            if elapsed.as_secs_f64() >= 0.2 {
                let bytes_since_update = download_len.saturating_sub(last_downloaded_bytes);
                let bytes_per_second = bytes_since_update as f64 / elapsed.as_secs_f64();

                let progress = match total_size {
                    Some(total) if total > 0 => (download_len as f32 / total as f32).clamp(0.0, 1.0),
                    _ => 0.0,
                };

                let _ = progress_tx.send((progress, bytes_per_second));

                last_update_time = Instant::now();
                last_downloaded_bytes = download_len;
            }
        }
        let _ = progress_tx.send((1.0, 0.0));

        Ok(())
    }
}

impl Render for Gui {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().size_full().p_8().bg(rgb(0xF8FAFC)).child(
            v_flex()
                .w_full()
                .gap_4()
                .p_4()
                .bg(rgb(0xFFFFFF))
                .rounded_md()
                .border_1()
                .border_color(rgb(0xCBD5E1))
                .text_center()
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
                    div()
                        .w_full()
                        .child(
                            Input::new(&self.save_path)
                                .appearance(false)
                                .disabled(true)
                                .cursor_text()
                                .text_align(TextAlign::Center)
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
