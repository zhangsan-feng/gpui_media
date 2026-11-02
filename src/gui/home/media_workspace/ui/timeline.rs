use super::super::{MediaWorkspace, TimelineDrag, TimelineTarget};
use crate::component::color::rgb_to_u32;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{ContextMenuExt, PopupMenuItem};
use gpui_kit::component::{Disableable, ElementExt, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::time::Duration;

pub(super) fn timecode(time: Duration) -> String {
    let millis = time.as_millis();
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        millis / 3_600_000,
        millis / 60_000 % 60,
        millis / 1000 % 60,
        millis % 1000
    )
}

impl MediaWorkspace {
    pub(in crate::gui::home::media_workspace) fn render_timeline(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = self.play_core.read(cx)._view_state();
        let duration = self.edited_duration();
        let position = self.edited_position(cx);
        let enabled = self.clip_range(cx).is_some();
        let selection = self.clip.selection;
        let parts = self
            .clip
            .timeline
            .as_ref()
            .map(|timeline| timeline.ranges().len())
            .unwrap_or(0);
        v_flex()
            .w_full()
            .flex_shrink_0()
            .gap_3()
            .p_3()
            .rounded_xl()
            .border_1()
            .border_color(rgb_to_u32(226, 232, 240))
            .bg(rgb_to_u32(255, 255, 255))
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_size(px(13.))
                            .child("剪辑时间线"),
                    )
                    .child(div().flex_1())
                    .child(self.timeline_button(
                        "media-editor-reset-clip",
                        "重置",
                        state.duration.is_some() && !state.is_loading && !self.save_in_progress,
                        |this, _, _, cx| this.reset_clip_selection(cx),
                        cx,
                    ))
                    .child(self.timeline_button(
                        "media-editor-preview-clip",
                        if selection.is_some() {
                            "播放选区"
                        } else {
                            "预览结果"
                        },
                        enabled,
                        |this, _, _, cx| this.preview_clip(cx),
                        cx,
                    ))
                    .child(
                        Button::new("media-workspace-save")
                            .label(if self.save_in_progress { "保存中…" } else { "保存" })
                            .disabled(!self.can_save_edits(cx))
                            .tooltip("将整条剪辑时间线写回原文件；支持 MP3、FLAC、WAV、AAC、MP4、MKV、MOV，有修改后可保存")
                            .on_click(cx.listener(|this, _, _, cx| this.save_edits(cx))),
                    )
                    .child(self.render_export_menu(cx)),
            )
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .text_size(px(11.))
                    .text_color(rgb_to_u32(100, 116, 139))
                    .children(
                        (0..=4).map(|index| {
                            div().child(timecode(duration.mul_f64(index as f64 / 4.0)))
                        }),
                    ),
            )
            .child(self.render_timeline_track(duration, position, enabled, cx))
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .gap_3()
                    .flex_wrap()
                    .text_size(px(12.))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb_to_u32(51, 65, 85))
                            .child(format!("剪辑后 {} · {} 个片段", timecode(duration), parts)),
                    )
                    .when_some(selection, |el, range| {
                        el.child(div().text_color(rgb_to_u32(168, 38, 122)).child(format!(
                            "选区 {} – {} · {}",
                            timecode(range.start),
                            timecode(range.end),
                            timecode(range.end - range.start)
                        )))
                    })
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_color(rgb_to_u32(100, 116, 139))
                            .child(format!("当前位置 {}", timecode(position))),
                    ),
            )
            .when_some(self.clip.error.clone(), |el, error| {
                el.child(
                    div()
                        .text_size(px(11.))
                        .text_color(rgb_to_u32(190, 48, 90))
                        .child(error),
                )
            })
            .into_any_element()
    }

    fn timeline_button(
        &self,
        id: &'static str,
        label: &'static str,
        enabled: bool,
        click: impl Fn(&mut MediaWorkspace, &ClickEvent, &mut Window, &mut Context<MediaWorkspace>)
        + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        Button::new(id)
            .label(label)
            .secondary()
            .disabled(!enabled)
            .on_click(cx.listener(click))
            .into_any_element()
    }

    fn render_timeline_track(
        &self,
        duration: Duration,
        position: Duration,
        enabled: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let total = duration.as_secs_f32().max(0.001);
        let position_ratio = (position.as_secs_f32() / total).clamp(0.0, 1.0);
        let editor = cx.entity();
        let selection_editor = editor.clone();
        let drag = TimelineDrag {
            owner: editor.entity_id(),
            target: TimelineTarget::Selection,
        };
        let mut offset = Duration::ZERO;
        let parts: Vec<_> = self
            .clip
            .timeline
            .as_ref()
            .map(|timeline| {
                timeline
                    .ranges()
                    .iter()
                    .enumerate()
                    .map(|(index, span)| {
                        let start = offset;
                        let length = span.end - span.start;
                        offset += length;
                        div()
                            .id(("media-editor-kept-segment", index))
                            .absolute()
                            .left(relative(start.as_secs_f32() / total))
                            .top(px(17.))
                            .h(px(33.))
                            .w(relative(length.as_secs_f32() / total))
                            .border_1()
                            .border_color(rgb_to_u32(218, 180, 218))
                            .bg(rgb_to_u32(250, 232, 247))
                            .rounded_sm()
                            .child(
                                div()
                                    .px_2()
                                    .pt_1()
                                    .text_size(px(11.))
                                    .text_color(rgb_to_u32(168, 38, 122))
                                    .text_ellipsis()
                                    .child(format!("片段 {}", index + 1)),
                            )
                            .into_any_element()
                    })
                    .collect()
            })
            .unwrap_or_default();
        div()
            .id("media-editor-timeline-track")
            .relative()
            .w_full()
            .h(px(56.))
            .rounded_lg()
            .overflow_hidden()
            .bg(rgb_to_u32(241, 245, 249))
            .border_1()
            .border_color(rgb_to_u32(226, 232, 240))
            .on_prepaint(move |bounds, _, app| {
                let _ = editor.update(app, |this, _| this.clip.timeline_bounds = Some(bounds));
            })
            .when(enabled, |el| {
                el.cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, window, cx| {
                            window.focus(&this.focus_handle, cx);
                            this.begin_timeline_selection(event.position, cx);
                        }),
                    )
                    .on_drag(drag, move |_, _, window, cx| {
                        let position = window.mouse_position();
                        let _ = selection_editor.update(cx, |this, cx| {
                            this.move_timeline_pointer(TimelineTarget::Selection, position, cx);
                        });
                        cx.new(|_| Empty)
                    })
                    .on_drag_move::<TimelineDrag>(cx.listener(
                        |this, event: &DragMoveEvent<TimelineDrag>, _, cx| {
                            let drag = event.drag(cx);
                            if drag.owner == cx.entity_id() {
                                this.move_timeline_pointer(drag.target, event.event.position, cx);
                            }
                        },
                    ))
                    .on_drop(cx.listener(|this, drag: &TimelineDrag, window, cx| {
                        if drag.owner == cx.entity_id() {
                            if drag.target != TimelineTarget::Playhead {
                                // GPUI consumes an accepted drop before mouse-up bubbles.
                                this.finish_timeline_selection(window.mouse_position(), cx);
                            }
                        }
                    }))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseUpEvent, _, cx| {
                            this.finish_timeline_selection(event.position, cx);
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseUpEvent, _, cx| {
                            this.finish_timeline_selection(event.position, cx);
                        }),
                    )
            })
            .children((0..=20).map(|index| {
                div()
                    .absolute()
                    .left(relative(index as f32 / 20.0))
                    .top_0()
                    .w(px(1.))
                    .h(px(if index % 5 == 0 { 14. } else { 7. }))
                    .bg(rgb_to_u32(203, 213, 225))
            }))
            .children(parts)
            .when_some(self.clip.selection, |el, range| {
                let menu_editor = cx.entity();
                el.child(
                    div()
                        .id("media-editor-selection")
                        .absolute()
                        .left(relative(range.start.as_secs_f32() / total))
                        .top(px(14.))
                        .h(px(38.))
                        .w(relative((range.end - range.start).as_secs_f32() / total))
                        .border_2()
                        .border_color(rgb_to_u32(190, 48, 139))
                        .bg(rgba(0xec489940))
                        .rounded_sm()
                        .when(enabled, |el| {
                            el.child(
                                self.render_selection_handle(TimelineTarget::SelectionStart, cx),
                            )
                            .child(self.render_selection_handle(TimelineTarget::SelectionEnd, cx))
                        })
                        .context_menu(move |menu, _, _| {
                            let editor = menu_editor.clone();
                            menu.item(PopupMenuItem::new("删除选区").disabled(!enabled).on_click(
                                move |_, _, app| {
                                    let _ = editor.update(app, |this, cx| {
                                        this.delete_selected_clip(cx);
                                    });
                                },
                            ))
                        }),
                )
            })
            .when(enabled, |el| {
                el.child(
                    div()
                        .absolute()
                        .left(relative(position_ratio.min(0.998)))
                        .top_0()
                        .bottom_0()
                        .w(px(2.))
                        .bg(rgb_to_u32(15, 23, 42)),
                )
                .child(
                    div()
                        .id("media-editor-playhead-handle")
                        .absolute()
                        .left(relative(position_ratio.min(0.99)))
                        .top_0()
                        .w(px(10.))
                        .h(px(14.))
                        .rounded_sm()
                        .bg(rgb_to_u32(15, 23, 42))
                        .cursor_col_resize()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, window, cx| {
                                cx.stop_propagation();
                                window.focus(&this.focus_handle, cx);
                                this.clip.selection_anchor = None;
                                this.clip.selection_dragging = false;
                                this.clip.selection_resize = None;
                                this.pause_editor(cx);
                            }),
                        )
                        .on_drag(
                            TimelineDrag {
                                owner: cx.entity_id(),
                                target: TimelineTarget::Playhead,
                            },
                            |_, _, _, cx| cx.new(|_| Empty),
                        ),
                )
            })
            .into_any_element()
    }

    fn render_selection_handle(
        &self,
        target: TimelineTarget,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let is_start = target == TimelineTarget::SelectionStart;
        let editor = cx.entity();
        div()
            .id(if is_start {
                "media-editor-selection-start"
            } else {
                "media-editor-selection-end"
            })
            .absolute()
            .top(px(-2.))
            .bottom(px(-2.))
            .w(px(12.))
            .when(is_start, |el| el.left(px(-2.)))
            .when(!is_start, |el| el.right(px(-2.)))
            .rounded_sm()
            .bg(rgb_to_u32(190, 48, 139))
            .hover(|style| style.bg(rgb_to_u32(157, 23, 106)))
            .cursor_col_resize()
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    cx.stop_propagation();
                    window.focus(&this.focus_handle, cx);
                    this.begin_selection_resize(target, cx);
                }),
            )
            .on_drag(
                TimelineDrag {
                    owner: cx.entity_id(),
                    target,
                },
                move |_, _, window, cx| {
                    let _ = editor.update(cx, |this, cx| {
                        this.move_timeline_pointer(target, window.mouse_position(), cx);
                    });
                    cx.new(|_| Empty)
                },
            )
            .child(div().w(px(2.)).h(px(16.)).rounded_sm().bg(white()))
            .into_any_element()
    }
}
