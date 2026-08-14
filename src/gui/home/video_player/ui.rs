use super::{SidePanelState, VideoPlayer};
use crate::component::color::rgb_to_u32;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::input::Input;
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarMode};
use gpui_component::{IconName, h_flex, v_flex};
use std::rc::Rc;
use std::time::Duration;

impl VideoPlayer {
    fn _render_control_button(
        &self,
        id: impl Into<ElementId>,
        label: impl IntoElement,
        is_primary: bool,
        click: impl Fn(&mut VideoPlayer, &ClickEvent, &mut Window, &mut Context<VideoPlayer>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .h(px(36.))
            .px_3()
            .rounded_lg()
            .border_1()
            .border_color(if is_primary {
                rgb_to_u32(190, 48, 139)
            } else {
                rgb_to_u32(218, 208, 225)
            })
            .bg(if is_primary {
                rgb_to_u32(190, 48, 139)
            } else {
                rgb_to_u32(255, 255, 255)
            })
            .text_color(if is_primary {
                rgb_to_u32(255, 255, 255)
            } else {
                rgb_to_u32(73, 66, 92)
            })
            .text_size(px(13.))
            .font_weight(FontWeight::MEDIUM)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(move |style| {
                if is_primary {
                    style.bg(rgb_to_u32(165, 37, 120))
                } else {
                    style.bg(rgb_to_u32(248, 243, 249))
                }
            })
            .on_click(cx.listener(click))
            .child(label)
    }

    fn _render_media_queue(&self, cx: &mut Context<Self>) -> AnyElement {
        let playlist_content = self._render_playlist_content(cx);
        let project_info = self._render_project_info(cx);

        let media_queue = self._render_media_queue_content(playlist_content, project_info, cx);

        self._render_side_panel(media_queue)
    }

    fn _render_side_panel(&self, media_queue: AnyElement) -> AnyElement {
        let side_panel_state = self.side_panel_state;
        let animation_id = self.side_panel_animation_id;
        let panel_fraction = 1.0 / 3.0;
        let panel = div()
            .h_full()
            .w(relative(panel_fraction))
            .flex_shrink_0()
            .overflow_hidden()
            .child(
                v_flex()
                    .size_full()
                    .min_h_0()
                    .gap_3()
                    .p_3()
                    .rounded_xl()
                    .overflow_hidden()
                    .border_1()
                    .border_color(rgb_to_u32(231, 220, 235))
                    .bg(rgb_to_u32(252, 249, 254))
                    .child(media_queue),
            );

        match side_panel_state {
            SidePanelState::Open => panel.into_any_element(),
            SidePanelState::Closed => panel.w(relative(0.)).opacity(0.).into_any_element(),
            SidePanelState::Opening | SidePanelState::Closing => {
                let opening = side_panel_state == SidePanelState::Opening;
                panel
                    .with_animation(
                        format!("video-player-media-queue-{animation_id}"),
                        Animation::new(Duration::from_millis(500)).with_easing(ease_in_out),
                        move |el, delta| {
                            let progress = if opening { delta } else { 1.0 - delta };
                            el.w(relative(panel_fraction * progress)).opacity(progress)
                        },
                    )
                    .into_any_element()
            }
        }
    }

    fn _render_media_queue_content(
        &self,
        playlist_content: AnyElement,
        project_info: AnyElement,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .size_full()
            .min_h_0()
            .gap_3()
            .child(
                h_flex()
                    .h(px(40.))
                    .w_full()
                    .flex_shrink_0()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(Input::new(&self.network_url_input).cleanable(true)),
                    )
                    .child(self._render_control_button(
                        "custmer-player-play-url",
                        "播放",
                        false,
                        |this, _, window, cx| this._play_network_url(window, cx),
                        cx,
                    )),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .gap_3()
                    .child(
                        v_flex()
                            .flex_1()
                            .min_h_0()
                            .gap_2()
                            .p_3()
                            .rounded_lg()
                            .overflow_hidden()
                            .border_1()
                            .border_color(rgb_to_u32(231, 220, 235))
                            .bg(rgb_to_u32(255, 255, 255))
                            .child(playlist_content),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_h_0()
                            .gap_3()
                            .p_3()
                            .rounded_lg()
                            .overflow_hidden()
                            .border_1()
                            .border_color(rgb_to_u32(231, 220, 235))
                            .bg(rgb_to_u32(255, 255, 255))
                            .child(project_info),
                    ),
            )
            .into_any_element()
    }

    fn _render_project_info(&self, cx: &mut Context<Self>) -> AnyElement {
        let view_state = self.play_core.read(cx)._view_state();
        let resource = if !view_state.player.title.trim().is_empty() {
            view_state.player.title.clone()
        } else if !view_state.player.id.trim().is_empty() {
            view_state.player.id.clone()
        } else if !view_state.player.url.trim().is_empty() {
            view_state.player.url.clone()
        } else {
            "暂无资源".to_string()
        };
        let resolution = if view_state.frame_width > 0.0 && view_state.frame_height > 0.0 {
            format!(
                "{} × {}",
                view_state.frame_width as u32, view_state.frame_height as u32
            )
        } else {
            "未知".to_string()
        };
        let frame_rate = if view_state.frame_rate > 0.0 {
            format!("{:.2} FPS", view_state.frame_rate)
        } else {
            "未知".to_string()
        };
        let codec = view_state.codec.unwrap_or_else(|| "未知".to_string());

        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .w_full()
                    .items_start()
                    .gap_3()
                    .child(
                        div()
                            .w(px(64.))
                            .flex_shrink_0()
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(148, 140, 163))
                            .child("资源"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(73, 66, 92))
                            .text_ellipsis()
                            .child(resource),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .w(px(64.))
                            .flex_shrink_0()
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(148, 140, 163))
                            .child("分辨率"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(73, 66, 92))
                            .child(resolution),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .w(px(64.))
                            .flex_shrink_0()
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(148, 140, 163))
                            .child("帧率"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(73, 66, 92))
                            .child(frame_rate),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .w(px(64.))
                            .flex_shrink_0()
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(148, 140, 163))
                            .child("编码"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(73, 66, 92))
                            .child(codec),
                    ),
            )
            .into_any_element()
    }

    fn _render_playlist_content(&self, cx: &mut Context<Self>) -> AnyElement {
        let play_list = Rc::new(self.play_list.clone());
        let current_index = self.current_index;
        let player = cx.entity().clone();
        let play_list_state = self.play_list_state.clone();

        if play_list.is_empty() {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(13.))
                .text_color(rgb_to_u32(148, 140, 163))
                .child("暂无播放内容")
                .into_any_element()
        } else {
            let playlist_list = list(play_list_state.clone(), move |index, _, _| {
                let Some(item) = play_list.get(index).cloned() else {
                    return div().into_any_element();
                };
                let is_current = current_index == Some(index);
                let title = if !item.title.trim().is_empty() {
                    item.title
                } else if !item.id.trim().is_empty() {
                    item.id
                } else {
                    format!("播放项 {}", index + 1)
                };
                let player = player.clone();

                div()
                    .id(("custmer-player-play-list-item", index))
                    .w_full()
                    .min_h(px(44.))
                    .px_3()
                    .rounded_lg()
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .bg(if is_current {
                        rgb_to_u32(252, 226, 244)
                    } else {
                        rgb_to_u32(255, 255, 255)
                    })
                    .text_color(if is_current {
                        rgb_to_u32(168, 38, 122)
                    } else {
                        rgb_to_u32(73, 66, 92)
                    })
                    .hover(|style| style.bg(rgb_to_u32(247, 240, 249)))
                    .on_click(move |_, _, app| {
                        let _ = player.update(app, |this, cx| this._play_item(index, cx));
                    })
                    .child(
                        div()
                            .w_full()
                            .text_size(px(13.))
                            .text_ellipsis()
                            .child(title),
                    )
                    .into_any_element()
            })
            .size_full();

            h_flex()
                .size_full()
                .min_h_0()
                .items_stretch()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .min_h_0()
                        .child(playlist_list),
                )
                .child(
                    div().w(px(8.)).h_full().child(
                        Scrollbar::vertical(&play_list_state)
                            .mode(ScrollbarMode::Always)
                            .axis(ScrollbarAxis::Vertical),
                    ),
                )
                .into_any_element()
        }
    }

    fn _render_player_content(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let frame = self.play_core.update(cx, |player, cx| {
            player._frame_ui(window, cx).into_any_element()
        });
        let progress = self.play_core.update(cx, |player, cx| {
            player._progress_ui(window, cx).into_any_element()
        });
        let volume = self
            .play_core
            .update(cx, |player, cx| player._volume_ui(cx).into_any_element());
        let duration = self
            .play_core
            .update(cx, |player, _| player._duration_ui().into_any_element());
        let is_playing = self.play_core.read(cx)._view_state().is_playing;

        v_flex()
            .flex_1()
            .h_full()
            .min_w_0()
            .min_h_0()
            .gap_4()
            .p_4()
            .rounded_xl()
            .border_1()
            .border_color(rgb_to_u32(231, 220, 235))
            .bg(rgb_to_u32(255, 255, 255))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .rounded_xl()
                    .overflow_hidden()
                    .border_1()
                    .border_color(rgb_to_u32(218, 208, 225))
                    .shadow_sm()
                    .child(frame),
            )
            .child(
                v_flex()
                    .mt_auto()
                    .w_full()
                    .gap_3()
                    .p_3()
                    .rounded_xl()
                    .border_1()
                    .border_color(rgb_to_u32(218, 208, 225))
                    .bg(rgb_to_u32(252, 249, 254))
                    .shadow_sm()
                    .child(progress)
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .flex_shrink_0()
                                    .gap_2()
                                    .items_center()
                                    .child(self._render_control_button(
                                        "custmer-player-settings",
                                        IconName::Menu,
                                        false,
                                        |this, _, _, cx| this.toggle_side_panel(cx),
                                        cx,
                                    ))
                                    .child(self._render_control_button(
                                        "custmer-player-previous",
                                        IconName::ChevronLeft,
                                        false,
                                        |this, _, _, cx| this._play_previous(cx),
                                        cx,
                                    ))
                                    .child(self._render_control_button(
                                        "custmer-player-toggle",
                                        if is_playing {
                                            IconName::Pause
                                        } else {
                                            IconName::Play
                                        },
                                        true,
                                        |this, _, _, cx| {
                                            let _ = this
                                                .play_core
                                                .update(cx, |player, cx| player._toggle_play(cx));
                                        },
                                        cx,
                                    ))
                                    .child(self._render_control_button(
                                        "custmer-player-next",
                                        IconName::ChevronRight,
                                        false,
                                        |this, _, _, cx| this._play_next(cx),
                                        cx,
                                    )),
                            )
                            .child(
                                h_flex()
                                    .flex_shrink_0()
                                    .gap_3()
                                    .items_center()
                                    .child(volume)
                                    .child(duration),
                            ),
                    ),
            )
    }
}

impl Render for VideoPlayer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .min_w_0()
            .min_h_0()
            .p_3()
            .overflow_hidden()
            .rounded_xl()
            .bg(rgb_to_u32(242, 237, 246))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                let item = this.play_core.read(cx)._file_drop_source(paths);
                if let Some(item) = item {
                    this._append_and_play(item, cx);
                }
            }))
            .child(
                h_flex()
                    .size_full()
                    .min_w_0()
                    .min_h_0()
                    .items_stretch()
                    .gap_4()
                    .overflow_hidden()
                    .child(self._render_player_content(window, cx))
                    .child(self._render_media_queue(cx)),
            )
    }
}
