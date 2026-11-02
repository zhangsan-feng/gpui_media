use super::super::{ExportStatus, MediaWorkspace, WorkspaceMode};
use gpui_kit::{AsyncApp, Context};
use player_core::PlayCoreExport;

impl MediaWorkspace {
    pub(in crate::gui::home::media_workspace) fn can_save_edits(&self, cx: &Context<Self>) -> bool {
        if self.mode != WorkspaceMode::Editing
            || self.export_in_progress
            || self.playlist_delete_in_progress
            || self.clip_range(cx).is_none()
        {
            return false;
        }
        let state = self.play_core.read(cx)._view_state();
        let Ok(path) = super::file_path_from_url(&state.player.url) else {
            return false;
        };
        if PlayCoreExport::save_format(&path).is_none() {
            return false;
        }
        let Some(timeline) = self.clip.timeline.as_ref() else {
            return false;
        };
        let ranges = timeline.ranges();
        !ranges.is_empty()
            && (ranges.len() != 1
                || !ranges[0].start.is_zero()
                || Some(ranges[0].end) != state.duration)
    }

    pub(in crate::gui::home::media_workspace) fn save_edits(&mut self, cx: &mut Context<Self>) {
        if !self.can_save_edits(cx) {
            return;
        }
        let source = self.play_core.read(cx)._view_state().player;
        let Ok(input) = super::file_path_from_url(&source.url) else {
            return;
        };
        let segments = self.clip.timeline.as_ref().unwrap().ranges().to_vec();
        self.pause_editor(cx);
        self.save_in_progress = true;
        self.export_in_progress = true;
        self.export_status = Some(ExportStatus::Working("正在保存剪辑到原文件…".into()));
        log::info!(
            "[media-workspace:save] requested: item_id={} path={} segments={segments:?}",
            source.id,
            input.display()
        );
        // Stop the pipeline before writing, and keep the edit model while the source is released.
        let release = self
            .play_core
            .update(cx, |player, cx| player._stop_and_release(cx));
        let entity = cx.entity();
        let mut cx_async = cx.to_async();
        cx.notify();
        cx.spawn(move |_, _: &mut AsyncApp| async move {
            release.await;
            let result =
                tokio::spawn(async move { PlayCoreExport::save_timeline(input, segments).await })
                    .await
                    .map_err(|error| anyhow::anyhow!("保存任务异常: {error}"))
                    .and_then(|result| result);
            let _ = entity.update(&mut cx_async, |this, cx| {
                let status = match result {
                    Ok(path) => {
                        log::info!("[media-workspace:save] saved: {}", path.display());
                        this.clip = Default::default();
                        ExportStatus::Success(format!("已保存到原文件：{}", path.display()))
                    }
                    Err(error) => {
                        log::error!("[media-workspace:save] failed: {error:#}");
                        ExportStatus::Error(format!("保存失败，未替换原文件：{error:#}"))
                    }
                };
                this.play_core
                    .update(cx, |player, cx| player._play_source(source, cx));
                this.save_in_progress = false;
                this.export_in_progress = false;
                this.export_status = Some(status);
                cx.notify();
            });
        })
        .detach();
    }
}
