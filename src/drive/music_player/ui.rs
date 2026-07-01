use crate::component::home::rgb_to_u32;
use crate::drive::music_player::{MusicPlayer, ProgressDrag, VolumeDrag};
use gpui::*;
use gpui_component::button::Button;
use gpui_component::popover::Popover;
use gpui_component::scroll::{ScrollableElement, Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::text::markdown;
use gpui_component::*;
use std::rc::Rc;
use std::time::Duration;

impl MusicPlayer {
    pub(crate) fn player_progress_control_ui(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let total = self
            .total_duration
            .unwrap_or_else(|| Duration::from_secs(0));
        let display_position = self
            .scrub_position
            .filter(|_| self.is_scrubbing)
            .unwrap_or(self.current_position);
        let progress_bar_width = self
            .progress_bar_bounds
            .as_ref()
            .map(|bounds| bounds.size.width.as_f32())
            .unwrap_or(0.0);

        let progress_ratio = if total.as_secs_f32() > 0.0 {
            (display_position.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let progress_bar_entity = cx.entity();

        v_flex()
            .gap_2()
            .child(
                div()
                    .h(px(8.))
                    .w_full()
                    .rounded_full()
                    .bg(rgb_to_u32(226, 232, 240))
                    .cursor_pointer()
                    .on_prepaint({
                        let progress_bar_entity = progress_bar_entity.clone();
                        move |bounds: Bounds<Pixels>, _: &mut Window, cx: &mut App| {
                            let _ = progress_bar_entity.update(cx, |this, _cx| {
                                this.progress_bar_bounds = Some(bounds);
                            });
                        }
                    })
                    .id("music_progress_bar")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, _, cx| {
                            if let Some(bounds) = this.progress_bar_bounds {
                                if let Some(target) =
                                    this.get_progress_position(event.position, bounds)
                                {
                                    this.seek_audio_progress(target, cx);
                                    this.is_scrubbing = false;
                                    this.scrub_position = None;
                                }
                            }
                        }),
                    )
                    .on_drag(ProgressDrag, |_value, _offset, _, cx| cx.new(|_| Empty))
                    .on_drag_move::<ProgressDrag>(cx.listener(
                        |this, event: &DragMoveEvent<ProgressDrag>, _, _cx| {
                            if let Some(target) =
                                this.get_progress_position(event.event.position, event.bounds)
                            {
                                this.is_scrubbing = true;
                                this.scrub_position = Some(target);
                            }
                        },
                    ))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            if this.is_scrubbing {
                                if let Some(target) = this.scrub_position.take() {
                                    this.seek_audio_progress(target, cx);
                                }
                                this.is_scrubbing = false;
                            }
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            if this.is_scrubbing {
                                if let Some(target) = this.scrub_position.take() {
                                    this.seek_audio_progress(target, cx);
                                }
                                this.is_scrubbing = false;
                            }
                        }),
                    )
                    .child(
                        div()
                            .h(px(8.))
                            .w(px(progress_bar_width * progress_ratio))
                            .rounded_full()
                            .bg(rgb_to_u32(59, 130, 246)),
                    ),
            )
            .child(
                h_flex()
                    .text_size(px(12.))
                    .justify_between()
                    .w_full()
                    .child(
                        div()
                            .w(px(window.bounds().size.width.as_f32() * 0.7))
                            .text_color(rgb_to_u32(15, 23, 42))
                            .overflow_x_scrollbar()
                            .mb_3()
                            .child(
                                markdown(if let Some(r) = self.play_err.clone() {
                                    r
                                } else if self.current_player.name.is_empty() {
                                    "No music source loaded".to_string()
                                } else {
                                    format!(
                                        "{} / {}",
                                        self.current_player.name, self.current_player.source
                                    )
                                })
                                .selectable(true)
                                .whitespace_nowrap()
                                .text_color(rgb(0x94A3B8))
                                .cursor_text(),
                            ),
                    )
                    .child(
                        h_flex()
                            .flex_shrink_0()
                            .gap_2()
                            .child(self.format_time(display_position))
                            .child("/")
                            .child(self.format_time(total)),
                    ),
            )
    }

    fn player_list_vm(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_virtual_list(
            cx.entity().clone(),
            "music-player-vm-list",
            Rc::new(
                self.player_list
                    .iter()
                    .map(|_| size(px(100.), px(96.)))
                    .collect(),
            ),
            |view, visible_range, _, cx| {
                visible_range
                    .map(|index| {
                        let data = view.player_list[index].clone();
                        let is_current = view.current_player.id == data.id;
                        let row_bg = if is_current {
                            rgb_to_u32(239, 246, 255)
                        } else {
                            rgb_to_u32(255, 255, 255)
                        };
                        let row_hover_bg = if is_current {
                            rgb_to_u32(219, 234, 254)
                        } else {
                            rgb_to_u32(239, 246, 255)
                        };
                        let row_border = if is_current {
                            rgb_to_u32(96, 165, 250)
                        } else {
                            rgb_to_u32(226, 232, 240)
                        };
                        let row_hover_border = if is_current {
                            rgb_to_u32(59, 130, 246)
                        } else {
                            rgb_to_u32(203, 213, 225)
                        };
                        let track_name = if data.name.is_empty() {
                            "Untitled track".to_string()
                        } else {
                            data.name.clone()
                        };
                        let author_text = if data.author.is_empty() {
                            "Unknown artist".to_string()
                        } else {
                            data.author.clone()
                        };
                        let source_text = if data.source.is_empty() {
                            "Unknown source".to_string()
                        } else {
                            data.source.clone()
                        };

                        div().w_full().h_full().px_2().py_2().child(
                            h_flex()
                                .id(format!("{}-hover-id", data.id.clone()))
                                .w_full()
                                .h_full()
                                .items_center()
                                .justify_between()
                                .gap_3()
                                .p_3()
                                .rounded_lg()
                                .border_1()
                                .border_color(row_border)
                                .bg(row_bg)
                                .hover(move |style| {
                                    style.bg(row_hover_bg).border_color(row_hover_border)
                                })
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap_3()
                                        .flex_1()
                                        .min_w_0()
                                        .child(
                                            div()
                                                .size(px(26.))
                                                .rounded_full()
                                                .bg(if is_current {
                                                    rgb_to_u32(219, 234, 254)
                                                } else {
                                                    rgb_to_u32(241, 245, 249)
                                                })
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_size(px(11.))
                                                .text_color(if is_current {
                                                    rgb_to_u32(37, 99, 235)
                                                } else {
                                                    rgb_to_u32(100, 116, 139)
                                                })
                                                .child((index + 1).to_string()),
                                        )
                                        .child(if data.img.is_empty() {
                                            div()
                                                .size(px(52.))
                                                .rounded_lg()
                                                .bg(rgb_to_u32(226, 232, 240))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_color(rgb_to_u32(100, 116, 139))
                                                .text_size(px(18.))
                                                .child("♪")
                                                .into_any_element()
                                        } else {
                                            img(data.img.clone())
                                                .size(px(52.))
                                                .rounded_lg()
                                                .into_any_element()
                                        })
                                        .child(
                                            v_flex()
                                                .gap_1()
                                                .flex_1()
                                                .min_w_0()
                                                .child(
                                                    div()
                                                        .w_full()
                                                        .overflow_hidden()
                                                        .whitespace_nowrap()
                                                        .text_ellipsis()
                                                        .text_size(px(14.))
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(rgb_to_u32(15, 23, 42))
                                                        .child(track_name),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(12.))
                                                        .text_color(rgb_to_u32(71, 85, 105))
                                                        .child(author_text),
                                                )
                                                .child(
                                                    div()
                                                        .w_full()
                                                        .overflow_hidden()
                                                        .whitespace_nowrap()
                                                        .text_ellipsis()
                                                        .text_size(px(11.))
                                                        .text_color(rgb_to_u32(148, 163, 184))
                                                        .child(source_text),
                                                ),
                                        ),
                                )
                                .child(div().flex_shrink_0().child(if is_current {
                                    h_flex()
                                        .gap_1()
                                        .items_center()
                                        .px_3()
                                        .py_1()
                                        .rounded_full()
                                        .bg(rgb_to_u32(219, 234, 254))
                                        .text_size(px(11.))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb_to_u32(37, 99, 235))
                                        .child(
                                            h_flex()
                                                .gap_0p5()
                                                .items_end()
                                                .child(
                                                    div()
                                                        .w(px(3.))
                                                        .h(px(8.))
                                                        .rounded_sm()
                                                        .bg(rgb_to_u32(37, 99, 235)),
                                                )
                                                .child(
                                                    div()
                                                        .w(px(3.))
                                                        .h(px(12.))
                                                        .rounded_sm()
                                                        .bg(rgb_to_u32(59, 130, 246)),
                                                )
                                                .child(
                                                    div()
                                                        .w(px(3.))
                                                        .h(px(6.))
                                                        .rounded_sm()
                                                        .bg(rgb_to_u32(96, 165, 250)),
                                                ),
                                        )
                                        .child("Playing")
                                        .into_any_element()
                                } else {
                                    Button::new(("music-play-index-", index))
                                        .label("Play")
                                        .outline()
                                        .on_click({
                                            let c = data.clone();
                                            cx.listener(move |this, _, _, cx| {
                                                this.current_player = c.clone();
                                                this.play_current_music(cx)
                                            })
                                        })
                                        .into_any_element()
                                })),
                        )
                    })
                    .collect()
            },
        )
        .track_scroll(&self.vm_scroll_handle)
    }

    pub(crate) fn player_list_ui(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Popover::new("music-player-open-popover")
            .anchor(Anchor::BottomCenter)
            .trigger(Button::new("show-form").label("播放列表").outline())
            .child(
                div()
                    .w(px(760.))
                    .h(px(560.))
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .size_full()
                            .gap_2()
                            .p_2()
                            .child(
                                h_flex().items_center().justify_between().child(
                                    div()
                                        .px_3()
                                        .py_1()
                                        .rounded_full()
                                        .bg(rgb_to_u32(241, 245, 249))
                                        .text_size(px(11.))
                                        .text_color(rgb_to_u32(71, 85, 105))
                                        .child(format!("{} 首歌曲", self.player_list.len())),
                                ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .rounded_lg()
                                    .bg(rgb_to_u32(248, 250, 252))
                                    .border_1()
                                    .border_color(rgb_to_u32(226, 232, 240))
                                    .p_2()
                                    .child(
                                        h_flex()
                                            .size_full()
                                            .gap_2()
                                            .child(self.player_list_vm(cx))
                                            .child(
                                                div().w(px(10.)).h_full().child(
                                                    Scrollbar::vertical(&self.vm_scroll_handle)
                                                        .scrollbar_show(ScrollbarShow::Always)
                                                        .axis(ScrollbarAxis::Vertical),
                                                ),
                                            ),
                                    ),
                            ),
                    )
                    .with_animation(
                        "music-player-open-popover-animation",
                        Animation::new(Duration::from_millis(550)).with_easing(ease_in_out),
                        |el, delta| el.opacity(0.2 + 0.8 * delta).h(px(12. + 548. * delta)),
                    ),
            )
    }

    pub(crate) fn player_lyrics_ui(&self) -> impl IntoElement {
        Popover::new("music-player-lyrics-popover")
            .anchor(Anchor::BottomCenter)
            .trigger(
                Button::new("music-player-lyrics-trigger")
                    .label("歌词")
                    .outline(),
            )
            .child(
                div()
                    .w(px(420.))
                    .h(px(360.))
                    .overflow_hidden()
                    .child(
                        v_flex().size_full().gap_3().p_4().child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_size(px(16.))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb_to_u32(15, 23, 42))
                                                .child("Lyrics Panel"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(12.))
                                                .text_color(rgb_to_u32(100, 116, 139))
                                                .child("Slides up from the bottom center"),
                                        ),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded_full()
                                        .bg(rgb_to_u32(241, 245, 249))
                                        .text_size(px(11.))
                                        .text_color(rgb_to_u32(71, 85, 105))
                                        .child(self.format_time(self.current_position)),
                                ),
                        ),
                    )
                    .with_animation(
                        "music-player-lyrics-popover-animation",
                        Animation::new(Duration::from_millis(360)).with_easing(ease_in_out),
                        |el, delta| el.opacity(0.15 + 0.85 * delta).h(px(20. + 340. * delta)),
                    ),
            )
    }

    fn render_control_button(
        &self,
        id: impl Into<ElementId>,
        label: impl IntoElement,
        click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> impl IntoElement {
        div()
            .size(px(28.))
            .rounded_full()
            .bg(rgb_to_u32(241, 245, 249))
            .border_1()
            .border_color(rgb_to_u32(203, 213, 225))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(12.))
            .text_color(rgb_to_u32(15, 23, 42))
            .cursor_pointer()
            .id(id)
            .on_click(click)
            .child(label)
    }

    pub(crate) fn player_control_ui(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .child(self.render_control_button(
                "music_prev_button",
                "<",
                cx.listener(|this, _, _, cx| {
                    this.prev_music(cx);
                }),
            ))
            .child(self.render_control_button(
                "music_play_button",
                if self.is_player {
                    div().child("◼")
                } else {
                    div().child("▶")
                },
                cx.listener(|this, _, _, cx| {
                    this.toggle_play(cx);
                }),
            ))
            .child(self.render_control_button(
                "music_next_button",
                ">",
                cx.listener(|this, _, _, cx| {
                    this.next_music(cx);
                }),
            ))
    }

    pub(crate) fn player_volume_control_ui(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let volume_ratio = self.volume.clamp(0.0, 1.0);
        let volume_bar_width = 150.0;

        h_flex().child(
            h_flex()
                .w(px(220.))
                .gap_2()
                .items_center()
                .ml_auto()
                .child(img("icon/icons8-voice-100.png").size(px(24.)))
                .child(
                    div()
                        .w(px(35.))
                        .text_size(px(11.))
                        .text_color(rgb_to_u32(100, 116, 139))
                        .child(format!("{:.0}%", volume_ratio * 100.0)),
                )
                .child(
                    div()
                        .h(px(8.))
                        .w(px(volume_bar_width))
                        .rounded_full()
                        .bg(rgb_to_u32(226, 232, 240))
                        .cursor_pointer()
                        .on_prepaint({
                            let volume_bar_entity = cx.entity();
                            move |bounds: Bounds<Pixels>, _: &mut Window, cx: &mut App| {
                                let _ = volume_bar_entity.update(cx, |this, _cx| {
                                    this.volume_bar_bounds = Some(bounds);
                                });
                            }
                        })
                        .id("music_volume_bar")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, event: &MouseDownEvent, _, _cx| {
                                if let Some(bounds) = this.volume_bar_bounds {
                                    let ratio = this.get_volume_position(event.position, bounds);
                                    this.set_volume_size(ratio);
                                }
                            }),
                        )
                        .on_drag(VolumeDrag, |_value, _offset, _, cx| cx.new(|_| Empty))
                        .on_drag_move::<VolumeDrag>(cx.listener(
                            |this, event: &DragMoveEvent<VolumeDrag>, _, _cx| {
                                let left = event.bounds.origin.x.as_f32();
                                let width = event.bounds.size.width.as_f32().max(1.0);
                                let ratio = ((event.event.position.x.as_f32() - left) / width)
                                    .clamp(0.0, 1.0);
                                this.set_volume_size(ratio);
                            },
                        ))
                        .child(
                            div()
                                .h(px(8.))
                                .w(px(volume_bar_width * volume_ratio))
                                .rounded_full()
                                .bg(rgb_to_u32(148, 163, 184)),
                        ),
                ),
        )
    }
}
