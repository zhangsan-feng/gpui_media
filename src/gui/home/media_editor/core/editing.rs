use super::super::{ClipSelection, MediaEditor, TimelineTarget};
use gpui_kit::{AsyncApp, Context, ExternalPaths, PathPromptOptions, Pixels, Point};
use player_core::{PlayCoreExportTrim, PlayCoreTimeline};
use std::time::Duration;

impl MediaEditor {
    pub(in crate::gui::home::media_editor) fn sync_clip_selection(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        let state = self.play_core.read(cx)._view_state();
        let source_key = format!("{}\n{}", state.player.id, state.player.url);
        if self.clip.source_key != source_key {
            log::info!(
                "[media-editor:timeline] source changed: item_id={}",
                state.player.id
            );
            self.clip = ClipSelection {
                source_key,
                ..Default::default()
            };
        }
        if self.clip.timeline.is_none() {
            if let Some(duration) = state.duration.filter(|value| !value.is_zero()) {
                self.clip.timeline = Some(PlayCoreTimeline::new(duration));
            }
        }
        if state.is_paused
            && self
                .clip
                .active_preview
                .is_some_and(|range| state.position >= range.end)
        {
            self.clip.active_preview = None;
            self.play_next_preview(cx);
        }
    }

    pub(in crate::gui::home::media_editor) fn edited_duration(&self) -> Duration {
        self.clip
            .timeline
            .as_ref()
            .map(|timeline| timeline.duration())
            .unwrap_or_default()
    }

    pub(in crate::gui::home::media_editor) fn edited_position(
        &self,
        cx: &Context<Self>,
    ) -> Duration {
        let position = self.play_core.read(cx)._progress().position;
        self.clip
            .timeline
            .as_ref()
            .map(|timeline| timeline.edited_position(position))
            .unwrap_or_default()
    }

    pub(in crate::gui::home::media_editor) fn clip_range(
        &self,
        cx: &Context<Self>,
    ) -> Option<PlayCoreExportTrim> {
        let state = self.play_core.read(cx)._view_state();
        let duration = self.edited_duration();
        if duration.is_zero() || state.is_idle || state.is_loading || state.error_message.is_some()
        {
            return None;
        }
        Some(PlayCoreExportTrim {
            start: Duration::ZERO,
            end: duration,
        })
    }

    fn clear_preview(&mut self) {
        self.clip.active_preview = None;
        self.clip.preview_ranges.clear();
    }

    pub(in crate::gui::home::media_editor) fn pause_editor(&mut self, cx: &mut Context<Self>) {
        self.clear_preview();
        self.play_core.update(cx, |player, cx| player._pause(cx));
    }

    pub(in crate::gui::home::media_editor) fn import_media_files(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        let dialog = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("导入视频或音频素材".into()),
        });
        let entity = cx.entity();
        let mut cx_async = cx.to_async();
        cx.spawn(move |_, _: &mut AsyncApp| async move {
            match dialog.await {
                Ok(Ok(Some(paths))) => {
                    let _ = entity.update(&mut cx_async, |this, cx| {
                        this.import_dropped_media(&ExternalPaths(paths.into_iter().collect()), cx);
                    });
                }
                Ok(Ok(None)) => {}
                result => {
                    log::error!("[media-editor:import] file picker failed: {result:?}");
                    let _ = entity.update(&mut cx_async, |this, cx| {
                        this.clip.error = Some("无法打开素材选择器，详情见日志".into());
                        cx.notify();
                    });
                }
            }
        })
        .detach();
    }

    pub(in crate::gui::home::media_editor) fn import_dropped_media(
        &mut self,
        paths: &ExternalPaths,
        cx: &mut Context<Self>,
    ) {
        let first = self.play_list.len();
        for path in paths
            .paths()
            .iter()
            .filter(|path| path.is_file() && super::is_supported_media_file(path))
        {
            if let Some(item) = self
                .play_core
                .read(cx)
                ._file_drop_source(&ExternalPaths(std::iter::once(path.clone()).collect()))
            {
                self._append_play_item(item, cx);
            }
        }
        let count = self.play_list.len() - first;
        if count > 0 {
            log::info!("[media-editor:import] imported {count} media files");
            self._play_item(first, cx);
        } else {
            self.clip.error = Some("未找到支持的视频或音频文件".into());
            cx.notify();
        }
    }

    pub(in crate::gui::home::media_editor) fn seek_editor(
        &mut self,
        position: Duration,
        cx: &mut Context<Self>,
    ) {
        self.clear_preview();
        let Some(source_position) = self
            .clip
            .timeline
            .as_ref()
            .and_then(|timeline| timeline.source_position(position))
        else {
            return;
        };
        let seeked = self.play_core.update(cx, |player, cx| {
            player._drag_progress(source_position, cx) && player._commit_progress_drag(cx)
        });
        self.clip.error = if seeked {
            None
        } else {
            log::warn!(
                "[media-editor:seek] failed: edited={position:?} source={source_position:?}"
            );
            Some("无法定位，请等待素材加载完成；详情见日志".into())
        };
        cx.notify();
    }

    pub(in crate::gui::home::media_editor) fn step_editor(
        &mut self,
        direction: i32,
        cx: &mut Context<Self>,
    ) {
        let state = self.play_core.read(cx)._view_state();
        let duration = self.edited_duration();
        if duration.is_zero() {
            return;
        }
        let position = self.edited_position(cx);
        let step = Duration::from_secs_f64(
            1.0 / if state.frame_rate > 0.0 {
                state.frame_rate
            } else {
                30.0
            },
        );
        self.clear_preview();
        self.play_core.update(cx, |player, cx| player._pause(cx));
        self.seek_editor(
            if direction < 0 {
                position.saturating_sub(step)
            } else {
                position.saturating_add(step).min(duration)
            },
            cx,
        );
    }

    pub(in crate::gui::home::media_editor) fn move_timeline_pointer(
        &mut self,
        target: TimelineTarget,
        point: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let Some(position) = self.timeline_position(point) else {
            return;
        };
        match target {
            TimelineTarget::Playhead => self.seek_editor(position, cx),
            TimelineTarget::Selection => {
                let Some(anchor) = self.clip.selection_anchor else {
                    return;
                };
                let duration = self.edited_duration();
                let gap = Duration::from_millis(1).min(duration);
                let start = anchor.min(position).min(duration.saturating_sub(gap));
                let end = anchor.max(position).max(start + gap).min(duration);
                self.clip.selection = Some(PlayCoreExportTrim { start, end });
                self.clip.selection_dragging = true;
                self.clip.error = None;
                cx.notify();
            }
            TimelineTarget::SelectionStart | TimelineTarget::SelectionEnd => {
                let Some(mut range) = self.clip.selection else {
                    return;
                };
                if self.clip.selection_resize != Some(target) {
                    return;
                }
                let gap = Duration::from_millis(1).min(self.edited_duration());
                if target == TimelineTarget::SelectionStart {
                    range.start = position.min(range.end.saturating_sub(gap));
                } else {
                    range.end = position.max(range.start + gap).min(self.edited_duration());
                }
                self.clip.selection = Some(range);
                self.clip.selection_dragging = true;
                self.clip.error = None;
                cx.notify();
            }
        }
    }

    fn timeline_position(&self, point: Point<Pixels>) -> Option<Duration> {
        let bounds = self.clip.timeline_bounds?;
        let duration = self.edited_duration();
        if duration.is_zero() {
            return None;
        }
        let ratio = ((point.x.as_f32() - bounds.origin.x.as_f32())
            / bounds.size.width.as_f32().max(1.0))
        .clamp(0.0, 1.0);
        Some(duration.mul_f64(ratio as f64))
    }

    pub(in crate::gui::home::media_editor) fn cancel_timeline_selection(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        self.clip.selection = None;
        self.clip.selection_anchor = None;
        self.clip.selection_dragging = false;
        self.clip.selection_resize = None;
        cx.notify();
    }

    pub(in crate::gui::home::media_editor) fn begin_selection_resize(
        &mut self,
        target: TimelineTarget,
        cx: &mut Context<Self>,
    ) {
        if self.clip_range(cx).is_none() || self.clip.selection.is_none() {
            return;
        }
        self.clip.selection_anchor = None;
        self.clip.selection_dragging = false;
        self.clip.selection_resize = Some(target);
        self.pause_editor(cx);
        log::info!(
            "[media-editor:selection] resizing {} boundary",
            if target == TimelineTarget::SelectionStart {
                "start"
            } else {
                "end"
            }
        );
    }

    pub(in crate::gui::home::media_editor) fn begin_timeline_selection(
        &mut self,
        point: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        if self.clip_range(cx).is_none() {
            return;
        }
        let Some(position) = self.timeline_position(point) else {
            return;
        };
        self.cancel_timeline_selection(cx);
        self.clip.selection_anchor = Some(position);
        self.clear_preview();
        self.play_core.update(cx, |player, cx| player._pause(cx));
        self.seek_editor(position, cx);
    }

    pub(in crate::gui::home::media_editor) fn finish_timeline_selection(
        &mut self,
        point: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let was_dragging = self.clip.selection_dragging;
        if was_dragging {
            self.move_timeline_pointer(
                self.clip
                    .selection_resize
                    .unwrap_or(TimelineTarget::Selection),
                point,
                cx,
            );
        }
        self.clip.selection_anchor = None;
        self.clip.selection_dragging = false;
        self.clip.selection_resize = None;
        if was_dragging {
            if let Some(range) = self.clip.selection {
                self.seek_editor(range.start, cx);
            }
            self.log_clip_selection(cx);
        }
    }

    pub(in crate::gui::home::media_editor) fn reset_clip_selection(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        self.clear_preview();
        self.play_core.update(cx, |player, cx| player._pause(cx));
        self.clip.timeline = self
            .play_core
            .read(cx)
            ._progress()
            .duration
            .map(PlayCoreTimeline::new);
        self.clip.error = None;
        self.cancel_timeline_selection(cx);
        self.seek_editor(Duration::ZERO, cx);
        log::info!("[media-editor:timeline] reset edits to full source");
    }

    fn play_next_preview(&mut self, cx: &mut Context<Self>) {
        let Some(range) = self.clip.preview_ranges.pop_front() else {
            return;
        };
        self.clip.active_preview = Some(range);
        log::info!(
            "[media-editor:preview] source range {:?}..{:?}, {} queued",
            range.start,
            range.end,
            self.clip.preview_ranges.len()
        );
        let played = self.play_core.update(cx, |player, cx| {
            player._play_segment(range.start, range.end, cx)
        });
        if !played {
            self.clear_preview();
            self.clip.error = Some("预览失败，素材可能不支持定位；详情见日志".into());
            cx.notify();
        }
    }

    fn start_preview(&mut self, range: PlayCoreExportTrim, cx: &mut Context<Self>) {
        let Some(timeline) = self.clip.timeline.as_ref() else {
            return;
        };
        let ranges = timeline.ranges_for(range);
        self.clear_preview();
        self.clip.error = None;
        self.clip.preview_ranges = ranges.into();
        self.play_next_preview(cx);
    }

    pub(in crate::gui::home::media_editor) fn preview_clip(&mut self, cx: &mut Context<Self>) {
        let Some(full) = self.clip_range(cx) else {
            return;
        };
        self.start_preview(self.clip.selection.unwrap_or(full), cx);
    }

    pub(in crate::gui::home::media_editor) fn toggle_editor_playback(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        let state = self.play_core.read(cx)._view_state();
        if state.is_playing {
            self.clear_preview();
            self.play_core.update(cx, |player, cx| player._pause(cx));
            return;
        }
        let Some(full) = self.clip_range(cx) else {
            return;
        };
        let range = self.clip.selection.unwrap_or(full);
        let position = self.edited_position(cx);
        let start = if position >= range.start && position < range.end {
            position
        } else {
            range.start
        };
        self.start_preview(
            PlayCoreExportTrim {
                start,
                end: range.end,
            },
            cx,
        );
    }

    pub(in crate::gui::home::media_editor) fn delete_selected_clip(
        &mut self,
        cx: &mut Context<Self>,
    ) {
        let Some(selection) = self.clip.selection else {
            return;
        };
        self.clear_preview();
        self.play_core.update(cx, |player, cx| player._pause(cx));
        if let Some(timeline) = self.clip.timeline.as_mut() {
            timeline.delete(selection);
        }
        self.cancel_timeline_selection(cx);
        self.clip.error = None;
        let duration = self.edited_duration();
        self.seek_editor(selection.start.min(duration), cx);
        log::info!(
            "[media-editor:timeline] deleted {:?}..{:?}; remaining={:?} spans={}",
            selection.start,
            selection.end,
            duration,
            self.clip
                .timeline
                .as_ref()
                .map(|timeline| timeline.ranges().len())
                .unwrap_or(0)
        );
        cx.notify();
    }

    pub(in crate::gui::home::media_editor) fn log_clip_selection(&self, _: &Context<Self>) {
        if let Some(range) = self.clip.selection {
            log::info!(
                "[media-editor:selection] edited range {:?}..{:?} length={:?}",
                range.start,
                range.end,
                range.end - range.start
            );
        }
    }
}
