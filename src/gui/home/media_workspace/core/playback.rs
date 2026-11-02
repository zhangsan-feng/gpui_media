use super::super::{MODE_TRANSITION_DURATION, MediaWorkspace, WorkspaceMode};
use super::file_path_from_url;
use gpui_kit::Context;
use player_core::PlayStatic;
use std::time::Instant;

impl MediaWorkspace {
    pub(in crate::gui::home::media_workspace) fn set_mode(
        &mut self,
        mode: WorkspaceMode,
        cx: &mut Context<Self>,
    ) {
        if self.mode == mode || self.playlist_delete_in_progress || self.save_in_progress {
            return;
        }
        let position = self.play_core.read(cx)._progress().position;
        self.pause_editor(cx);
        self.play_core.update(cx, |player, cx| {
            player._set_retain_source_at_end(mode == WorkspaceMode::Editing);
            // A regular seek clears the bounded segment used by editing previews.
            if player._drag_progress(position, cx) {
                player._commit_progress_drag(cx);
            }
        });
        self.mode_animation_from = self.mode_transition_progress();
        self.mode_animation_started = Some(Instant::now());
        self.mode_animation_id = self.mode_animation_id.wrapping_add(1);
        let animation_id = self.mode_animation_id;
        self.mode = mode;
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(MODE_TRANSITION_DURATION)
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.mode_animation_id == animation_id {
                    this.mode_animation_started = None;
                    cx.notify();
                }
            });
        })
        .detach();
        log::info!(
            "[media-workspace:mode] mode={mode:?} source_position={position:?} index={:?} transition_ms=500",
            self.current_index
        );
        cx.notify();
    }

    pub(in crate::gui::home::media_workspace) fn mode_transition_progress(&self) -> f32 {
        let target = if self.mode == WorkspaceMode::Editing {
            1.0
        } else {
            0.0
        };
        let Some(started) = self.mode_animation_started else {
            return target;
        };
        let progress = gpui_kit::ease_in_out(
            (started.elapsed().as_secs_f32() / MODE_TRANSITION_DURATION.as_secs_f32())
                .clamp(0.0, 1.0),
        );
        self.mode_animation_from + (target - self.mode_animation_from) * progress
    }

    pub(in crate::gui::home::media_workspace) fn toggle_playback(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        if self.playlist_delete_in_progress || self.save_in_progress {
            return;
        }
        if self.mode == WorkspaceMode::Editing {
            self.toggle_editor_playback(cx);
        } else {
            self.play_core
                .update(cx, |player, cx| player._toggle_play(cx));
        }
    }
    pub(in crate::gui::home::media_workspace) fn open_playlist_item_folder(
        &self,
        item: &PlayStatic,
        cx: &mut Context<Self>,
    ) {
        let path = match file_path_from_url(&item.url) {
            Ok(path) => path,
            Err(error) => {
                log::warn!(
                    "[media-workspace:folder] refusing to open folder for non-local item id={} url={}: {error:#}",
                    item.id,
                    item.url
                );
                return;
            }
        };
        log::info!(
            "[media-workspace:folder] revealing file in folder: item_id={} path={} exists={}",
            item.id,
            path.display(),
            path.exists()
        );
        cx.reveal_path(&path);
    }

    pub(in crate::gui::home::media_workspace) fn set_auto_play_next(
        &mut self,
        enabled: bool,
        cx: &mut Context<Self>,
    ) {
        if self.auto_play_next == enabled {
            return;
        }
        self.auto_play_next = enabled;
        log::info!(
            "[media-workspace:auto-next] enabled={enabled} mode={:?}",
            self.mode
        );
        cx.notify();
    }

    pub(in crate::gui::home::media_workspace) fn play_next_after_finished(
        &mut self,
        finished: &PlayStatic,
        cx: &mut Context<Self>,
    ) {
        if self.playlist_delete_in_progress || self.save_in_progress {
            return;
        }
        let Some(index) = self.current_index else {
            return;
        };
        let Some(current) = self.play_list.get(index) else {
            return;
        };
        if current.id != finished.id || current.url != finished.url {
            log::debug!(
                "[media-workspace:auto-next] ignored stale completion: finished_id={} current_id={}",
                finished.id,
                current.id
            );
            return;
        }
        if !self.auto_play_next {
            log::info!(
                "[media-workspace:auto-next] stopped at current item: enabled=false item_id={}",
                finished.id
            );
            return;
        }
        let Some(next) = self.play_list.get(index + 1) else {
            log::info!(
                "[media-workspace:auto-next] playlist finished: item_id={} count={}",
                finished.id,
                self.play_list.len()
            );
            return;
        };
        log::info!(
            "[media-workspace:auto-next] finished_id={} next_id={} next_index={}",
            finished.id,
            next.id,
            index + 1
        );
        self._play_next(cx);
    }
}
