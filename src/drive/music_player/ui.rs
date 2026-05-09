use std::rc::Rc;
use std::time::Duration;
use gpui::*;
use gpui_component::*;
use gpui_component::button::Button;
use gpui_component::popover::Popover;
use gpui_component::scroll::{ScrollableElement, Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::text::markdown;
use crate::component::home::rgb_to_u32;
use crate::drive::music_player::{MusicPlayer, ProgressDrag, VolumeDrag};
use crate::state::{GlobalState, StateEvent};

impl MusicPlayer {

    pub(crate) fn player_progress_control_ui(&self, window: &mut Window, cx: &mut Context<Self> ) -> impl IntoElement {
        let total = self.total_duration.unwrap_or_else(|| Duration::from_secs(0));
        let display_position = self.scrub_position.filter(|_| self.is_scrubbing).unwrap_or(self.current_position);
        let progress_bar_width = self.progress_bar_bounds .as_ref().map(|bounds| bounds.size.width.as_f32()).unwrap_or(0.0);

        let progress_ratio = if total.as_secs_f32() > 0.0 {
            (display_position.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let progress_bar_entity = cx.entity();

        v_flex()
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
                                if let Some(target) = this.position_from_drag(event.position, bounds) {
                                    this.seek_audio_progress(target, cx);
                                    this.is_scrubbing = false;
                                    this.scrub_position = None;
                                }
                            }
                        }),
                    )
                    .on_drag(ProgressDrag, |_value, _offset, _, cx| {
                        cx.new(|_| Empty)
                    })
                    .on_drag_move::<ProgressDrag>(cx.listener(
                        |this, event: &DragMoveEvent<ProgressDrag>, _, _cx| {
                            if let Some(target) = this.position_from_drag(event.event.position, event.bounds) {
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
                    )

            )
            .child(
                h_flex()
                    .text_size(px(12.))
                    .justify_between()
                    .w_full()
                    .child(
                        div()
                            .w(px(window.bounds().size.width.as_f32().clone() * 0.7))
                            .text_color(rgb_to_u32(15, 23, 42))
                            .overflow_x_scrollbar()
                            .mb_3()
                            .child(
                                markdown(
                                    if let Some(r) = self.play_err.clone() {
                                        r
                                    } else {
                                        if !self.current_player.music_name.is_empty() {
                                            self.current_player.music_name.clone()
                                            // "aaaaaa".to_string()
                                        } else {
                                            "没有加载音乐来源".to_string()
                                        }
                                    }
                                )
                                    .selectable(true)
                                    .whitespace_nowrap()
                                    .text_color(rgb(0x94A3B8))
                                    .cursor_text(),
                            )
                    )
                    .child(
                        h_flex()
                            .flex_shrink_0()
                            .gap_2()
                            .child(self.format_time(display_position))
                            .child("/")
                            .child(self.format_time(total)),
                    )
            )
    }

    fn player_list_vm(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_virtual_list(
            cx.entity().clone(),
            "music-player-vm-list",
            Rc::new(
                self.player_list
                    .iter()
                    .map(|_| size(px(100.), px(40.)))
                    .collect(),
            ),
            |view, visible_range, _, cx| {
                visible_range
                    .map(|index| {
                        let data = view.player_list[index].clone();
                        div()
                            .flex()
                            .justify_between()
                            .w_full()
                            .pr_2()
                            .child(
                                div()
                                    .gap_2()
                                    .justify_between()
                                    .h_flex()
                                    .child(img(data.music_pic.clone()).size(px(24.)).rounded_full())
                                    .child(data.music_author.clone())
                                    .child(data.music_platform.clone())
                                    .child(data.music_name.clone()),
                            )
                            .child(if view.current_player.music_id == data.music_id {
                                div().child("正在播放").into_any_element()
                            } else {
                                Button::new(("music-play-index-", index))
                                    .label("播放")
                                    .on_click({
                                        let c = data.clone();
                                        cx.listener(move |_, _, _, cx| {
                                            let mut cx_async = cx.to_async().clone();
                                            let state_handler = cx.global::<GlobalState>().0.clone();
                                            let c = c.clone();
                                            cx.spawn(|_, _: &mut AsyncApp| async move {
                                                state_handler.update(&mut cx_async, |_, cx| {
                                                    cx.emit(StateEvent::TogglePlayMusic(c.clone()))
                                                });
                                            })
                                                .detach()
                                        })
                                    })
                                    .into_any_element()
                            })
                    })
                    .collect()
            },
        )
            .track_scroll(&self.vm_scroll_handle)
    }

    pub(crate) fn player_list_ui(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Popover::new("music-player-open-popover")
            .anchor(Anchor::BottomRight)
            .trigger(Button::new("show-form").label("播放列表").outline())
            .child(
                div()
                    .h(px(600.))
                    .w(px(800.))

                    .child(
                        v_flex()
                            .gap_2()
                            .p_4()
                            .size_full()
                            .child(self.player_list_vm(cx))
                            .child(
                                Scrollbar::vertical(&self.vm_scroll_handle)
                                    .scrollbar_show(ScrollbarShow::Always)
                                    .axis(ScrollbarAxis::Vertical),
                            )
                    )
                    .with_animation(
                        "music-player-open-popover-animation",
                        Animation::new(Duration::from_millis(550)).with_easing(ease_in_out),
                        |el, delta| el.opacity(0.2 + 0.8 * delta).h(px(8. + 592. * delta)),
                    ),
            )
    }

    fn render_control_button(
        &self,
        id:impl Into<ElementId>,
        label:impl IntoElement,
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

    pub(crate) fn player_control_ui(&self, cx: &mut Context<Self>) -> impl IntoElement{

        h_flex()
            .gap_2()
            .child(
                self.render_control_button(
                    "music_prev_button",
                    "<",
                    cx.listener(|this, _, _, cx| { this.prev_music(cx); }))
            )
            .child(
                self.render_control_button(
                    "music_play_button",
                    if self.is_player {
                        div().child("||")
                    } else {
                        div().child("▶")
                    },
                    cx.listener(|this, _, _, cx| { this.toggle_play(cx); }))
            )

            .child(
                self.render_control_button(
                    "music_next_button",
                    ">",
                    cx.listener(|this, _, _, cx| { this.next_music(cx); }))
            )
    }

    pub(crate) fn player_volume_control_ui(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let volume_ratio = self.volume.clamp(0.0, 1.0);
        let volume_bar_width = 150.0;

        h_flex()
            .child(
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
                                        let ratio =
                                            this.volume_from_position(event.position, bounds);
                                        this.set_volume(ratio);
                                    }
                                }),
                            )
                            .on_drag(VolumeDrag, |_value, _offset, _, cx| {
                                cx.new(|_| Empty)
                            })
                            .on_drag_move::<VolumeDrag>(cx.listener(
                                |this, event: &DragMoveEvent<VolumeDrag>, _, _cx| {
                                    let left = event.bounds.origin.x.as_f32();
                                    let width = event.bounds.size.width.as_f32().max(1.0);
                                    let ratio = ((event.event.position.x.as_f32() - left)
                                        / width)
                                        .clamp(0.0, 1.0);
                                    this.set_volume(ratio);
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