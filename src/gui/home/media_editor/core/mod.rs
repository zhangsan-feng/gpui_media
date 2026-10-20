mod editing;
mod export;

pub(super) use export::export_format_label;

use super::MediaEditor;
use super::SidePanelState;
use anyhow::Context as AnyhowContext;
use gpui_kit::http_client::Url;
use gpui_kit::{AsyncApp, Context, PathPromptOptions, PromptButton, PromptLevel, Task, Window};
use player_core::PlayStatic;
use reqwest::header::HeaderMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

impl MediaEditor {
    pub(super) fn select_media_folder(&mut self, cx: &mut Context<Self>) {
        let dialog = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("选择媒体文件夹".into()),
        });
        let entity = cx.entity().clone();
        let mut cx_async = cx.to_async().clone();

        cx.spawn(move |_, _: &mut AsyncApp| async move {
            let folder = match dialog.await {
                Ok(Ok(Some(paths))) => paths.into_iter().find(|path| path.is_dir()),
                Ok(Ok(None)) => return,
                Ok(Err(error)) => {
                    log::error!("[media-editor:folder] folder picker failed: {error:#}");
                    return;
                }
                Err(error) => {
                    log::error!("[media-editor:folder] folder picker task failed: {error}");
                    return;
                }
            };
            let Some(folder) = folder else {
                log::warn!("[media-editor:folder] folder picker returned no directory");
                return;
            };

            let items = match read_media_folder(&folder) {
                Ok(items) => items,
                Err(error) => {
                    log::error!(
                        "[media-editor:folder] failed to read {}: {error:#}",
                        folder.display()
                    );
                    return;
                }
            };
            if items.is_empty() {
                log::warn!(
                    "[media-editor:folder] no supported media files in {}",
                    folder.display()
                );
                return;
            }

            let count = items.len();
            log::info!(
                "[media-editor:folder] loaded {count} media files from {}",
                folder.display()
            );
            let _ = entity.update(&mut cx_async, |this, cx| {
                let first_item = this.play_list.len();
                this.play_list.extend(items);
                this._play_item(first_item, cx);
            });
        })
        .detach();
    }

    pub(super) fn toggle_side_panel(&mut self, cx: &mut Context<Self>) {
        let next_state = match self.side_panel_state {
            SidePanelState::Open | SidePanelState::Opening => SidePanelState::Closing,
            SidePanelState::Closed | SidePanelState::Closing => SidePanelState::Opening,
        };
        self.side_panel_state = next_state;
        self.side_panel_animation_id = self.side_panel_animation_id.wrapping_add(1);
        let animation_id = self.side_panel_animation_id;

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;
            this.update(cx, |this, cx| {
                if this.side_panel_animation_id != animation_id {
                    return;
                }

                this.side_panel_state = match next_state {
                    SidePanelState::Opening => SidePanelState::Open,
                    SidePanelState::Closing => SidePanelState::Closed,
                    SidePanelState::Open | SidePanelState::Closed => return,
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    pub(super) fn _append_and_play(&mut self, item: PlayStatic, cx: &mut Context<Self>) {
        self._append_play_item(item, cx);
        self._play_item(self.play_list.len() - 1, cx);
    }

    pub(super) fn _play_url(&mut self, url: String, cx: &mut Context<Self>) {
        let title = url
            .rsplit('/')
            .next()
            .and_then(|part| part.split(['?', '#']).next())
            .filter(|part| !part.trim().is_empty())
            .unwrap_or("网络媒体")
            .to_string();
        self._append_and_play(
            PlayStatic {
                id: Uuid::new_v4().to_string(),
                title,
                url,
                headers: HeaderMap::new(),
            },
            cx,
        );
    }

    pub(super) fn _play_network_url(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let url = self.network_url_input.read(cx).value().trim().to_string();
        if url.is_empty() {
            return;
        }

        self._play_url(url, cx);
        self.network_url_input
            .update(cx, |input, cx| input.set_value("", window, cx));
    }

    pub fn _play_item(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(item) = self.play_list.get(index).cloned() else {
            return;
        };
        self.current_index = Some(index);
        self.clip = Default::default();
        let _ = self
            .play_core
            .update(cx, |player, cx| player._play_source(item, cx));
        cx.notify();
    }

    pub(super) fn confirm_delete_playlist_item(
        &mut self,
        item: PlayStatic,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        log::info!(
            "[media-editor:playlist-delete] requested: item_id={} title={} url={}",
            item.id,
            item.title,
            item.url
        );
        let path = match file_path_from_url(&item.url) {
            Ok(path) => path,
            Err(error) => {
                log::warn!(
                    "[media-editor:playlist-delete] refusing non-local item id={} url={}: {error:#}",
                    item.id,
                    item.url
                );
                self.playlist_delete_error = Some("只能删除本地媒体文件".to_string());
                cx.notify();
                return;
            }
        };
        let detail = format!("{}\n{}", item.title, path.display());
        let answers = [PromptButton::cancel("取消"), PromptButton::ok("删除")];
        let prompt = window.prompt(
            PromptLevel::Warning,
            "确定删除这个磁盘文件？",
            Some(&detail),
            &answers,
            cx,
        );
        let entity = cx.entity().clone();
        let mut cx_async = cx.to_async().clone();
        let background_executor = cx.background_executor().clone();

        cx.spawn(move |_, _: &mut AsyncApp| async move {
            let answer = match prompt.await {
                Ok(answer) => answer,
                Err(error) => {
                    log::warn!(
                        "[media-editor:playlist-delete] confirmation prompt failed: item_id={} path={} error={error}",
                        item.id,
                        path.display()
                    );
                    return;
                }
            };
            log::info!(
                "[media-editor:playlist-delete] confirmation returned: item_id={} path={} button_index={answer}",
                item.id,
                path.display()
            );
            if answer != 1 {
                log::info!(
                    "[media-editor:playlist-delete] canceled by user: item_id={} path={}",
                    item.id,
                    path.display()
                );
                return;
            }

            let prepared = entity.update(&mut cx_async, |this, cx| {
                this.prepare_playlist_file_deletion(&item, &path, cx)
            });
            let Some((index, stopped_current, release_task)) = prepared else {
                log::warn!(
                    "[media-editor:playlist-delete] could not prepare deletion: item_id={} path={}",
                    item.id,
                    path.display()
                );
                return;
            };
            if let Some(task) = release_task {
                task.await;
            }

            log::info!(
                "[media-editor:playlist-delete] removing file: item_id={} index={} stopped_current={stopped_current} path={}",
                item.id,
                index,
                path.display()
            );
            let delete_path = path.clone();
            let result = background_executor
                .spawn(async move { std::fs::remove_file(&delete_path) })
                .await;
            let _ = entity.update(&mut cx_async, |this, cx| {
                this.finish_playlist_file_deletion(&item, &path, stopped_current, result, cx);
            });
        })
        .detach();
    }

    fn prepare_playlist_file_deletion(
        &mut self,
        item: &PlayStatic,
        path: &Path,
        cx: &mut Context<Self>,
    ) -> Option<(usize, bool, Option<Task<()>>)> {
        if self.playlist_delete_in_progress {
            return None;
        }
        let index = self
            .play_list
            .iter()
            .position(|candidate| candidate.id == item.id)?;
        let active_url = self.play_core.read(cx)._view_state().player.url.clone();
        let stopped_current = file_path_from_url(&active_url)
            .is_ok_and(|active_path| paths_refer_to_same_file(&active_path, path));

        self.playlist_delete_error = None;
        self.playlist_delete_in_progress = true;
        let release_task = if stopped_current {
            log::info!(
                "[media-editor:playlist-delete] stopping GStreamer before deleting active file: {}",
                path.display()
            );
            Some(
                self.play_core
                    .update(cx, |player, cx| player._stop_and_release(cx)),
            )
        } else {
            None
        };
        cx.notify();
        Some((index, stopped_current, release_task))
    }

    fn finish_playlist_file_deletion(
        &mut self,
        item: &PlayStatic,
        path: &Path,
        stopped_current: bool,
        result: std::io::Result<()>,
        cx: &mut Context<Self>,
    ) {
        let Some(index) = self
            .play_list
            .iter()
            .position(|candidate| candidate.id == item.id)
        else {
            self.playlist_delete_in_progress = false;
            cx.notify();
            return;
        };

        self.playlist_delete_in_progress = false;
        match result {
            Ok(()) => {
                log::info!(
                    "[media-editor:playlist-delete] deleted file: {}",
                    path.display()
                );
                self.play_list.remove(index);
                self.playlist_delete_error = None;

                if stopped_current {
                    self.current_index = None;
                    if !self.play_list.is_empty() {
                        let next_index = index.min(self.play_list.len() - 1);
                        self._play_item(next_index, cx);
                    }
                } else if let Some(current_index) = self.current_index {
                    self.current_index = if current_index == index {
                        None
                    } else if current_index > index {
                        Some(current_index - 1)
                    } else {
                        Some(current_index)
                    };
                }
            }
            Err(error) => {
                log::error!(
                    "[media-editor:playlist-delete] failed to delete {}: kind={:?} error={error}",
                    path.display(),
                    error.kind()
                );
                self.playlist_delete_error =
                    Some(format!("删除失败：{error}。文件可能仍被其他程序占用。"));
                if stopped_current {
                    if let Some(index) = self
                        .play_list
                        .iter()
                        .position(|candidate| candidate.id == item.id)
                    {
                        self._play_item(index, cx);
                    }
                }
            }
        }
        cx.notify();
    }

    pub(crate) fn _play_previous(&mut self, cx: &mut Context<Self>) {
        let Some(index) = self.current_index else {
            return;
        };
        if let Some(previous) = index.checked_sub(1) {
            self._play_item(previous, cx);
        }
    }

    pub(crate) fn _play_next(&mut self, cx: &mut Context<Self>) {
        let Some(index) = self.current_index else {
            return;
        };
        if index + 1 < self.play_list.len() {
            self._play_item(index + 1, cx);
        }
    }
}

fn read_media_folder(folder: &Path) -> anyhow::Result<Vec<PlayStatic>> {
    let entries = std::fs::read_dir(folder)
        .with_context(|| format!("无法读取媒体目录: {}", folder.display()))?;
    let mut paths = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                log::warn!(
                    "[media-editor:folder] failed to read a directory entry in {}: {error}",
                    folder.display()
                );
                continue;
            }
        };
        let path = entry.path();
        if path.is_file() && is_supported_media_file(&path) {
            paths.push(path);
        }
    }

    paths.sort_by_cached_key(|path| {
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
    });

    Ok(paths
        .into_iter()
        .filter_map(|path| {
            let url = match Url::from_file_path(&path) {
                Ok(url) => url.to_string(),
                Err(()) => {
                    log::warn!(
                        "[media-editor:folder] could not convert media path to URL: {}",
                        path.display()
                    );
                    return None;
                }
            };
            let title = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "本地媒体".to_string());

            Some(PlayStatic {
                id: Uuid::new_v4().to_string(),
                title,
                url,
                headers: HeaderMap::new(),
            })
        })
        .collect())
}

fn is_supported_media_file(path: &Path) -> bool {
    const MEDIA_EXTENSIONS: &[&str] = &[
        "3g2", "3gp", "aac", "ac3", "aif", "aiff", "alac", "amr", "ape", "asf", "au", "avi", "dts",
        "flac", "flv", "m2ts", "m4a", "m4v", "mka", "mkv", "mov", "mp2", "mp3", "mp4", "mpc",
        "mpeg", "mpg", "oga", "ogg", "ogv", "opus", "ra", "rm", "rmvb", "ts", "tta", "vob", "wav",
        "webm", "wma", "wmv", "wv",
    ];

    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            MEDIA_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        })
}

fn file_path_from_url(url: &str) -> anyhow::Result<PathBuf> {
    let url = Url::parse(url).with_context(|| format!("无法解析本地媒体地址: {url}"))?;
    url.to_file_path()
        .map_err(|_| anyhow::anyhow!("无法转换本地媒体路径: {url}"))
}

fn paths_refer_to_same_file(left: &Path, right: &Path) -> bool {
    let left = std::fs::canonicalize(left).unwrap_or_else(|_| left.to_path_buf());
    let right = std::fs::canonicalize(right).unwrap_or_else(|_| right.to_path_buf());
    #[cfg(windows)]
    {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}
