mod delete_dialog;
mod export;
mod mode_switch;
mod side_panel;
mod timeline;

use super::{DeleteSelection, MediaWorkspace, TogglePlayback, WorkspaceMode};
use crate::component::color::rgb_to_u32;
use gpui_kit::component::{IconName, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl MediaWorkspace {
    pub(in crate::gui::home::media_workspace) fn _render_control_button(
        &self,
        id: impl Into<ElementId>,
        label: impl IntoElement,
        is_primary: bool,
        click: impl Fn(&mut MediaWorkspace, &ClickEvent, &mut Window, &mut Context<MediaWorkspace>)
        + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .h(px(32.))
            .px_3()
            .rounded_lg()
            .border_1()
            .border_color(if is_primary {
                rgb_to_u32(190, 48, 139)
            } else {
                rgb_to_u32(226, 232, 240)
            })
            .bg(if is_primary {
                rgb_to_u32(190, 48, 139)
            } else {
                rgb_to_u32(255, 255, 255)
            })
            .text_color(if is_primary {
                rgb_to_u32(255, 255, 255)
            } else {
                rgb_to_u32(71, 85, 105)
            })
            .text_size(px(12.))
            .font_weight(FontWeight::MEDIUM)
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(move |style| {
                style.bg(if is_primary {
                    rgb_to_u32(165, 37, 120)
                } else {
                    rgb_to_u32(248, 250, 252)
                })
            })
            .on_click(cx.listener(click))
            .child(label)
    }

    fn render_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let state = self.play_core.read(cx)._view_state();
        let frame = self.play_core.update(cx, |player, cx| {
            player._frame_preview_ui(window, cx).into_any_element()
        });
        let volume = self
            .play_core
            .update(cx, |player, cx| player._volume_ui(cx).into_any_element());
        let duration = format!(
            "{} / {}",
            timeline::timecode(if self.mode == WorkspaceMode::Editing {
                self.edited_position(cx)
            } else {
                state.position
            }),
            timeline::timecode(if self.mode == WorkspaceMode::Editing {
                self.edited_duration()
            } else {
                state.duration.unwrap_or_default()
            })
        );
        let title = if state.player.title.is_empty() {
            "导入视频或音频开始播放".to_owned()
        } else {
            state.player.title
        };
        v_flex()
            .flex_1()
            .h_full()
            .min_w_0()
            .min_h_0()
            .gap_3()
            .p_3()
            .rounded_xl()
            .border_1()
            .border_color(rgb_to_u32(226, 232, 240))
            .bg(rgb_to_u32(255, 255, 255))
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(if self.mode == WorkspaceMode::Editing {
                                "素材预览"
                            } else {
                                "正在播放"
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(100, 116, 139))
                            .text_ellipsis()
                            .child(title),
                    ),
            )
            .child(
                div()
                    .id("media-editor-preview-frame")
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .rounded_lg()
                    .overflow_hidden()
                    .bg(rgb_to_u32(15, 23, 42))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            window.focus(&this.focus_handle, cx);
                            this.toggle_playback(cx);
                        }),
                    )
                    .child(frame),
            )
            .when(self.mode == WorkspaceMode::Playback, |preview| {
                preview.child(self.play_core.update(cx, |player, cx| {
                    player._progress_ui(window, cx).into_any_element()
                }))
            })
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                self._render_control_button(
                                    "media-workspace-previous",
                                    h_flex()
                                        .gap_1()
                                        .child(IconName::ChevronLeft)
                                        .child("上一项"),
                                    false,
                                    |this, _, _, cx| this._play_previous(cx),
                                    cx,
                                ),
                            )
                            .child(self._render_control_button(
                                "media-editor-toggle-play",
                                if state.is_playing {
                                    IconName::Pause
                                } else {
                                    IconName::Play
                                },
                                true,
                                |this, _, _, cx| this.toggle_playback(cx),
                                cx,
                            ))
                            .child(
                                self._render_control_button(
                                    "media-workspace-next",
                                    h_flex()
                                        .gap_1()
                                        .child("下一项")
                                        .child(IconName::ChevronRight),
                                    false,
                                    |this, _, _, cx| this._play_next(cx),
                                    cx,
                                ),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(rgb_to_u32(100, 116, 139))
                                    .child(duration),
                            ),
                    )
                    .child(volume),
            )
            .into_any_element()
    }

    pub(in crate::gui::home::media_workspace) fn render_workspace(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .id("media-editor-workspace")
            .track_focus(&self.focus_handle)
            .key_context("MediaWorkspace")
            .on_action(cx.listener(|this, _: &TogglePlayback, _, cx| this.toggle_playback(cx)))
            .on_action(cx.listener(|this, _: &DeleteSelection, _, cx| {
                if this.mode == WorkspaceMode::Editing {
                    this.delete_selected_clip(cx);
                }
            }))
            .size_full()
            .min_w_0()
            .min_h_0()
            .p_3()
            .gap_3()
            .overflow_hidden()
            .rounded_xl()
            .bg(rgb_to_u32(248, 250, 252))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                this.import_dropped_media(paths, cx)
            }))
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .gap_3()
                    .child(self._render_control_button(
                        "media-editor-toggle-library",
                        IconName::PanelLeft,
                        false,
                        |this, _, _, cx| this.toggle_side_panel(cx),
                        cx,
                    ))
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(gpui_kit::component::Icon::default().path("icon/scissors.svg"))
                            .child(div().font_weight(FontWeight::SEMIBOLD).child("播放器")),
                    )
                    .child(div().flex_1())
                    .child(self._render_control_button(
                        "workspace-auto-play-next",
                        "自动播放",
                        self.auto_play_next,
                        |this, _, _, cx| this.set_auto_play_next(!this.auto_play_next, cx),
                        cx,
                    ))
                    .child(self.render_mode_switch(cx)),
            )
            .child(
                h_flex()
                    .flex_1()
                    .w_full()
                    .min_w_0()
                    .min_h_0()
                    .items_stretch()
                    .gap_3()
                    .child(self._render_media_queue(cx))
                    .child(self.render_preview(window, cx)),
            )
            .when(
                self.mode == WorkspaceMode::Editing || self.mode_animation_started.is_some(),
                |workspace| workspace.child(self.render_mode_timeline(cx)),
            )
            .child(self._render_export_status())
            .into_any_element()
    }
}
