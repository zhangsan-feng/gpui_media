mod export;
mod side_panel;
mod timeline;

use super::{DeleteSelection, MediaEditor, TogglePlayback};
use crate::component::color::rgb_to_u32;
use gpui_kit::component::{IconName, h_flex, v_flex};
use gpui_kit::*;

impl MediaEditor {
    pub(in crate::gui::home::media_editor) fn _render_control_button(
        &self,
        id: impl Into<ElementId>,
        label: impl IntoElement,
        is_primary: bool,
        click: impl Fn(&mut MediaEditor, &ClickEvent, &mut Window, &mut Context<MediaEditor>) + 'static,
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
            timeline::timecode(self.edited_position(cx)),
            timeline::timecode(self.edited_duration())
        );
        let title = if state.player.title.is_empty() {
            "导入素材开始剪辑".to_owned()
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
                            .child("素材预览"),
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
                            this.toggle_editor_playback(cx);
                        }),
                    )
                    .child(frame),
            )
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
                            .child(self._render_control_button(
                                "media-editor-previous-frame",
                                IconName::ChevronLeft,
                                false,
                                |this, _, _, cx| this.step_editor(-1, cx),
                                cx,
                            ))
                            .child(self._render_control_button(
                                "media-editor-toggle-play",
                                if state.is_playing {
                                    IconName::Pause
                                } else {
                                    IconName::Play
                                },
                                true,
                                |this, _, _, cx| this.toggle_editor_playback(cx),
                                cx,
                            ))
                            .child(self._render_control_button(
                                "media-editor-next-frame",
                                IconName::ChevronRight,
                                false,
                                |this, _, _, cx| this.step_editor(1, cx),
                                cx,
                            ))
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

    pub(in crate::gui::home::media_editor) fn render_workspace(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .id("media-editor-workspace")
            .track_focus(&self.focus_handle)
            .key_context("MediaEditor")
            .on_action(
                cx.listener(|this, _: &TogglePlayback, _, cx| this.toggle_editor_playback(cx)),
            )
            .on_action(
                cx.listener(|this, _: &DeleteSelection, _, cx| this.delete_selected_clip(cx)),
            )
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
                            .child(div().font_weight(FontWeight::SEMIBOLD).child("媒体编辑器")),
                    )
                    .child(div().flex_1()),
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
            .child(self.render_timeline(cx))
            .child(self._render_export_status())
            .into_any_element()
    }
}
