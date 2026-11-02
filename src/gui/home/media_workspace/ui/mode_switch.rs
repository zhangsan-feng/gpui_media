use super::super::{MODE_TRANSITION_DURATION, MediaWorkspace, WorkspaceMode};
use crate::component::color::rgb_to_u32;
use gpui_kit::component::h_flex;
use gpui_kit::*;

fn blend(from: [u8; 3], to: [u8; 3], progress: f32) -> Rgba {
    let channel = |index: usize| {
        (from[index] as f32 + (to[index] as f32 - from[index] as f32) * progress).round() as u8
    };
    rgb_to_u32(channel(0), channel(1), channel(2))
}

impl MediaWorkspace {
    pub(super) fn render_mode_switch(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_3()
            .child(self.render_mode_button(WorkspaceMode::Playback, "播放", cx))
            .child(self.render_mode_button(WorkspaceMode::Editing, "剪辑", cx))
    }

    fn render_mode_button(
        &self,
        mode: WorkspaceMode,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let from = self.mode_animation_from;
        let target = if self.mode == WorkspaceMode::Editing {
            1.0
        } else {
            0.0
        };
        let strength = move |editing: f32| {
            if mode == WorkspaceMode::Editing {
                editing
            } else {
                1.0 - editing
            }
        };
        let selected = strength(target);
        let button = div()
            .id(label)
            .h(px(32.))
            .px_3()
            .rounded_lg()
            .border_1()
            .bg(blend([255, 255, 255], [190, 48, 139], selected))
            .border_color(blend([226, 232, 240], [190, 48, 139], selected))
            .text_color(blend([71, 85, 105], [255, 255, 255], selected))
            .text_size(px(12.))
            .font_weight(FontWeight::MEDIUM)
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| this.set_mode(mode, cx)))
            .child(label);
        if self.mode_animation_started.is_some() {
            button
                .with_animation(
                    format!("workspace-mode-{label}-{}", self.mode_animation_id),
                    Animation::new(MODE_TRANSITION_DURATION).with_easing(ease_in_out),
                    move |button, delta| {
                        let selected = strength(from + (target - from) * delta);
                        button
                            .bg(blend([255, 255, 255], [190, 48, 139], selected))
                            .border_color(blend([226, 232, 240], [190, 48, 139], selected))
                            .text_color(blend([71, 85, 105], [255, 255, 255], selected))
                    },
                )
                .into_any_element()
        } else {
            button.into_any_element()
        }
    }

    pub(super) fn render_mode_timeline(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let from = self.mode_animation_from;
        let target = if self.mode == WorkspaceMode::Editing {
            1.0
        } else {
            0.0
        };
        let timeline = div()
            .id("workspace-mode-timeline")
            .w_full()
            .min_h_0()
            .flex_shrink_0()
            .overflow_hidden()
            .child(self.render_timeline(cx));
        if self.mode_animation_started.is_some() {
            timeline
                .with_animation(
                    format!("workspace-timeline-{}", self.mode_animation_id),
                    Animation::new(MODE_TRANSITION_DURATION).with_easing(ease_in_out),
                    move |timeline, delta| {
                        let progress = from + (target - from) * delta;
                        timeline.max_h(px(240.0 * progress)).opacity(progress)
                    },
                )
                .into_any_element()
        } else {
            timeline.into_any_element()
        }
    }
}
