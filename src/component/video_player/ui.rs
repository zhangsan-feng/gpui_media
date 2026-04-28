use std::rc::Rc;
use std::time::Duration;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::popover::Popover;
use gpui_component::{h_flex, v_flex, v_virtual_list, Anchor, ElementExt, StyledExt};
use gpui_component::input::Input;
use gpui_component::scroll::{ScrollableElement, Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::text::markdown;
use crate::component::home::rgb_to_u32;
use crate::component::video_player::{ProgressDrag, VideoPlayer, VolumeDrag};


impl VideoPlayer {
    fn player_list_vm(&self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_virtual_list(
            cx.entity().clone(),
            "video-player-vm-list",
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
                            .child(data.clone())
                            .child(if view.current_player_video == data {
                                div().child("正在播放").into_any_element()
                            } else {
                                Button::new(("video-play-btn", index))
                                    .label("播放")
                                    .on_click({
                                        let c = data.clone();
                                        cx.listener(move |this, _, _, cx| {
                                            let c = c.clone();
                                            this.current_player_video = c;
                                            this.reset_pipeline();
                                            this.play(cx);
                                        })
                                    })
                                    .into_any_element()
                            })
                            .child(
                                Button::new(("video-refresh-btn", index)).label("刷新")
                            )
                    })
                    .collect()
            },
        )
            .track_scroll(&self.vm_vm_scroll_handle)
    }

    pub(crate) fn player_list_ui(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Popover::new("video-player-open-popover")
            .anchor(Anchor::BottomRight)
            .trigger(Button::new("show-form").label("菜单").outline())
            .child(
                div()
                    .h(px(600.))
                    .w(px(800.))
                    .child(
                        v_flex()
                            .gap_2()
                            .p_4()
                            .size_full()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Input::new(&self.input_text)
                                    )
                                    .child(
                                        Button::new("load-video-url-btn")
                                            .label("加载")
                                            .on_click(cx.listener(|this, _, _, cx|{
                                                this.player_list.push(this.input_text.read(cx).text().to_string()) ;
                                                this.refresh(cx);
                                            }))
                                    )
                            )
                            .child(
                                h_flex()
                                    .border_1()
                                    .rounded_2xl()
                                    .border_color(rgb_to_u32(203, 213, 225))
                                    .p_2()
                                    .gap_2()
                                    .flex_grow()
                                    .child(self.player_list_vm(window, cx))
                                    .child(
                                        div()
                                            .w(px(10.))
                                            .h_full()
                                            .child(
                                                Scrollbar::vertical(&self.vm_vm_scroll_handle)
                                                    .scrollbar_show(ScrollbarShow::Always)
                                                    .axis(ScrollbarAxis::Vertical)
                                            )
                                    )
                            )
                    )
                    .with_animation(
                        "video-player-open-popover-animation",
                        Animation::new(Duration::from_millis(550)).with_easing(ease_in_out),
                        |el, delta| el.opacity(0.2 + 0.8 * delta).h(px(8. + 592. * delta)),
                    ),
            )
    }

    pub(crate) fn player_volume_control_ui(&self, _: &mut Window, cx: &mut Context<Self>,) -> impl IntoElement {
        let volume_ratio = self.video_player_volume.clamp(0.0, 1.0);
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
                            move |bounds: Bounds<Pixels>, _window: &mut Window, cx: &mut App| {
                                let _ = volume_bar_entity.update(cx, |this, _cx| {
                                    this.volume_bar_bounds = Some(bounds);
                                });
                            }
                        })
                        .id("music_volume_bar")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                                if let Some(bounds) = this.volume_bar_bounds {
                                    let ratio = this.volume_from_position(event.position, bounds);
                                    this.set_volume(ratio);
                                }
                            }),
                        )
                        .on_drag(VolumeDrag, |_value, _offset, _window, cx| cx.new(|_| Empty))
                        .on_drag_move::<VolumeDrag>(cx.listener(
                            |this, event: &DragMoveEvent<VolumeDrag>, _window, _cx| {
                                let left = event.bounds.origin.x.as_f32();
                                let width = event.bounds.size.width.as_f32().max(1.0);
                                let ratio = ((event.event.position.x.as_f32() - left) / width).clamp(0.0, 1.0);
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

    pub(crate) fn player_progress_control_ui(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let total = self.video_total_duration.unwrap_or_else(|| Duration::from_secs(0));
        let display_position = self.scrub_position.filter(|_| self.is_scrubbing).unwrap_or(self.video_player_duration);
        let progress_ratio = if total.as_secs_f32() > 0.0 {
            (display_position.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let progress_bar_width = self.progress_bar_bounds.as_ref().map(|bounds| bounds.size.width.as_f32()).unwrap_or(0.0);
        let progress_bar_entity = cx.entity();

        v_flex()
            .child(
                div()
                    .h(px(8.))
                    .w_full()
                    .rounded_full()
                    .bg(rgb(0xE2E8F0))
                    .cursor_pointer()
                    .on_prepaint({
                        let progress_bar_entity = progress_bar_entity.clone();
                        move |bounds: Bounds<Pixels>, _window: &mut Window, cx: &mut App| {
                            let _ = progress_bar_entity.update(cx, |this, _cx| {
                                this.progress_bar_bounds = Some(bounds);
                            });
                        }
                    })
                    .id("video_progress_bar")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                            if let Some(bounds) = this.progress_bar_bounds {
                                if let Some(target) = this.position_from_drag(event.position, bounds){
                                    this.seek_video(target);
                                    this.is_scrubbing = false;
                                    this.scrub_position = None;
                                }
                            }
                        }),
                    )
                    .on_drag(ProgressDrag, |_value, _offset, _window, cx: &mut App| {
                        cx.new(|_| Empty)
                    })
                    .on_drag_move::<ProgressDrag>(cx.listener(
                        |this, event: &DragMoveEvent<ProgressDrag>, _window, _cx| {
                            if let Some(target) = this.position_from_drag(event.event.position, event.bounds){
                                this.is_scrubbing = true;
                                this.scrub_position = Some(target);
                            }
                        },
                    ))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, _cx| {
                            if this.is_scrubbing {
                                if let Some(target) = this.scrub_position.take() {
                                    this.seek_video(target);
                                }
                                this.is_scrubbing = false;
                            }
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, _cx| {
                            if this.is_scrubbing {
                                if let Some(target) = this.scrub_position.take() {
                                    this.seek_video(target);
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
                            .bg(rgb(0x3B82F6)),
                    ),
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
                                    if self.current_player_video.is_empty() {
                                    "没有加载视频来源".to_string()
                                    } else {
                                        self.current_player_video.to_string()
                                    })
                                    .selectable(true)
                                    .text_color(rgb(0x94A3B8))
                                    .cursor_text()
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(self.format_time(display_position))
                            .child("/")
                            .child(self.format_time(total)),
                    ),
            )
    }
    pub(crate) fn player_control_ui(&self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .child(
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
                    .id("music_prev_button")
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.prev_video(cx);
                    }))
                    .child("<"),
            )
            .child(
                div()
                    .size(px(36.))
                    .rounded_full()
                    .bg(rgb_to_u32(59, 130, 246))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(14.))
                    .text_color(white())
                    .cursor_pointer()
                    .id("music_play_button")
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.toggle_play(cx);
                    }))
                    .child(
                        if self.is_player {
                            div().child("||")
                        } else {
                            div().child("▶")
                        }
                    ),
            )
            .child(
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
                    .id("music_nest_button")
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.next_video(cx);
                    }))
                    .child(">"),
            )
    }

    pub(crate) fn video_frame_ui(&self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {

        div()
            .flex_grow()
            .flex()
            .justify_center()
            .items_center()
            .overflow_hidden()
            .rounded_md()
            .border_1()
            .border_color(rgb(0xE2E8F0))
            // .bg(rgb(0x0F172A))
            .child(if let Some(frame) = self.render_image.clone() {
                div()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        img(frame)
                            .w_full()
                            .h_full()
                            .object_fit(ObjectFit::Contain),
                    )
                    .into_any_element()
            } else {
                v_flex()
                    .flex_grow()
                    .justify_center()
                    .items_center()
                    .w(window.bounds().size.width * 0.5)
                    .h(window.bounds().size.height * 0.5)
                    .overflow_hidden()
                    .overflow_scrollbar()
                    .child(
                        markdown(if let Some(player_err) = self.last_error.clone() {
                            player_err.to_string()
                        } else {
                            "没有加载视频来源".to_string()
                        })
                            .selectable(true)
                            .text_color(rgb(0x94A3B8))
                            .cursor_text()
                    )
                    .into_any_element()
            })
    }
}
