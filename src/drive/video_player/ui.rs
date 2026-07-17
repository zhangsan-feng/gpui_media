use crate::component::color::rgb_to_u32;
use crate::drive::video_player::{PlatState,  VideoPlayer};
use crate::drive::{LocalStatic, NetworkStatic};
use gpui::*;
use gpui_component::ElementExt;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::Input;
use gpui_component::popover::Popover;
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::text::markdown;
use gpui_component::{h_flex, v_flex, v_virtual_list};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use crate::drive::video_player::core::{ProgressDrag, VolumeDrag};

impl VideoPlayer {
    fn player_list_vm(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_virtual_list(
            cx.entity().clone(),
            "video-player-vm-list",
            Rc::new(
                self.player_list
                    .iter()
                    .map(|_| size(px(100.), px(76.)))
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
                            rgb_to_u32(248, 250, 252)
                        };
                        let row_border = if is_current {
                            rgb_to_u32(96, 165, 250)
                        } else {
                            rgb_to_u32(226, 232, 240)
                        };

                        div().w_full().h_full().px_2().py_1().child(
                            h_flex()
                                .id(format!("video-player-vm-list-{}", data.id))
                                .w_full()
                                .h_full()
                                .items_center()
                                .justify_between()
                                .gap_3()
                                .p_2()
                                .rounded_lg()
                                .border_1()
                                .border_color(row_border)
                                .bg(row_bg)
                                .hover(move |style| {
                                    style
                                        .bg(rgb_to_u32(239, 246, 255))
                                        .border_color(rgb_to_u32(147, 197, 253))
                                })
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap_3()
                                        .flex_1()
                                        .min_w_0()
                                        .child(
                                            div()
                                                .size(px(32.))
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
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(if is_current {
                                                    rgb_to_u32(37, 99, 235)
                                                } else {
                                                    rgb_to_u32(100, 116, 139)
                                                })
                                                .child((index + 1).to_string()),
                                        )
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
                                                        .child(data.name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .w_full()
                                                        .overflow_hidden()
                                                        .whitespace_nowrap()
                                                        .text_ellipsis()
                                                        .text_size(px(11.))
                                                        .text_color(rgb_to_u32(148, 163, 184))
                                                        .child(data.source.clone()),
                                                ),
                                        ),
                                )
                                .child(if is_current {
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
                                        .child("播放中")
                                        .into_any_element()
                                } else {
                                    Button::new(("video-play-btn", index))
                                        .label("播放")
                                        .primary()
                                        .on_click({
                                            let c = data.clone();
                                            cx.listener(move |this, _, _, cx| {
                                                this.current_player = c.clone();
                                                this.play(cx);
                                            })
                                        })
                                        .into_any_element()
                                })
                                .child(
                                    Button::new(("video-refresh-btn", index))
                                        .label("刷新")
                                        .ghost()
                                        .on_click({
                                            let c = data.clone();
                                            cx.listener(move |this, _, _, cx| {
                                                this.current_player = c.clone();
                                                this.play(cx);
                                            })
                                        }),
                                ),
                        )
                    })
                    .collect()
            },
        )
        .track_scroll(&self.vm_scroll_handle)
    }

    pub(crate) fn player_menu_ui(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let menu_h = window.bounds().size.height * 0.7;
        let menu_w = window.bounds().size.width * 0.5;

        Popover::new("video-player-open-popover")
            .anchor(Anchor::BottomRight)
            .trigger(Button::new("show-form").label("播放列表").outline())
            .child(
                div()
                    .h(menu_h)
                    .w(menu_w)
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .gap_3()
                            .p_4()
                            .size_full()
                            .child(
                                h_flex()
                                    .items_center()
                                    .rounded_xl()
                                    .bg(rgb_to_u32(241, 245, 249))
                                    .border_1()
                                    .border_color(rgb_to_u32(226, 232, 240))
                                    .p_3()
                                    .gap_3()
                                    .child(Input::new(&self.input_text))
                                    .child(
                                        Button::new("load-video-url-btn")
                                            .label("加载视频")
                                            .primary()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                let source =
                                                    this.input_text.read(cx).text().to_string();
                                                let id = format!(
                                                    "{:x}",
                                                    md5::compute(source.as_bytes())
                                                );
                                                let player = NetworkStatic {
                                                    id: id.clone(),
                                                    name: "".to_string(),
                                                    img: "".to_string(),
                                                    author: "".to_string(),
                                                    category: "".to_string(),
                                                    headers: Default::default(),
                                                    source: source.clone(),
                                                    func: Arc::new(LocalStatic),
                                                };

                                                this.current_player = player.clone();

                                                if !this
                                                    .player_list
                                                    .iter()
                                                    .any(|data| data.id == id)
                                                {
                                                    this.player_list.push(player);
                                                }
                                                this.play(cx);
                                            })),
                                    ),
                            )
                            .child(
                                h_flex().items_center().justify_between().child(
                                    div()
                                        .rounded_full()
                                        .px_3()
                                        .py_1()
                                        .bg(rgb_to_u32(219, 234, 254))
                                        .text_size(px(11.))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb_to_u32(37, 99, 235))
                                        .child(format!("{} 个视频", self.player_list.len())),
                                ),
                            )
                            .child(
                                h_flex()
                                    .flex_1()
                                    .border_1()
                                    .rounded_lg()
                                    .border_color(rgb_to_u32(226, 232, 240))
                                    .bg(rgb_to_u32(241, 245, 249))
                                    .p_2()
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
                    )
                    .with_animation(
                        "video-player-open-popover-animation",
                        Animation::new(Duration::from_millis(550)).with_easing(ease_in_out),
                        move |el, delta| el.opacity(0.2 + 0.8 * delta).h(menu_h * delta.max(0.02)),
                    ),
            )
    }

    pub(crate) fn player_volume_control_ui(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let volume_ratio = self.video_player_volume.clamp(0.0, 1.0);
        let volume_bar_width = 150.0;

        h_flex().child(
            h_flex()
                .w(px(220.))
                .gap_2()
                .items_center()
                .ml_auto()
                .child(
                    div()
                        .size(px(32.))
                        .rounded_full()
                        .bg(rgb_to_u32(239, 246, 255))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(img("icon/icons8-voice-100.png").size(px(18.))),
                )
                .child(
                    div()
                        .w(px(35.))
                        .text_size(px(11.))
                        .text_color(rgb_to_u32(100, 116, 139))
                        .child(format!("{:.0}%", volume_ratio * 100.0)),
                )
                .child(
                    div()
                        .h(px(7.))
                        .w(px(volume_bar_width))
                        .rounded_full()
                        .bg(rgb_to_u32(226, 232, 240))
                        .cursor_pointer()
                        .on_prepaint({
                            let volume_bar_entity = cx.entity();
                            move |bounds: Bounds<Pixels>, _: &mut Window, cx: &mut App| {
                                let _ = volume_bar_entity.update(cx, |this, _| {
                                    this.volume_bar_bounds = Some(bounds);
                                });
                            }
                        })
                        .id("video_volume_bar")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, event: &MouseDownEvent, _, _| {
                                if let Some(bounds) = this.volume_bar_bounds {
                                    let ratio = this.get_volume_position(event.position, bounds);
                                    this.set_volume_size(ratio);
                                }
                            }),
                        )
                        .on_drag(VolumeDrag, |_value, _offset, _, cx| cx.new(|_| Empty))
                        .on_drag_move::<VolumeDrag>(cx.listener(
                            |this, event: &DragMoveEvent<VolumeDrag>, _, _| {
                                let left = event.bounds.origin.x.as_f32();
                                let width = event.bounds.size.width.as_f32().max(1.0);
                                let ratio = ((event.event.position.x.as_f32() - left) / width)
                                    .clamp(0.0, 1.0);
                                this.set_volume_size(ratio);
                            },
                        ))
                        .child(
                            div()
                                .h(px(7.))
                                .w(px(volume_bar_width * volume_ratio))
                                .rounded_full()
                                .bg(rgb_to_u32(59, 130, 246)),
                        ),
                ),
        )
    }

    pub(crate) fn player_progress_control_ui(
        &self,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let total = self
            .video_total_duration
            .unwrap_or_else(|| Duration::from_secs(0));
        let display_position = self
            .pending_seek_position
            .filter(|_| self.is_dragging_progress_bar)
            .unwrap_or(self.video_player_duration);
        let progress_ratio = if total.as_secs_f32() > 0.0 {
            (display_position.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let progress_bar_width = self
            .progress_bar_bounds
            .as_ref()
            .map(|bounds| bounds.size.width.as_f32())
            .unwrap_or(0.0);
        let progress_bar_entity = cx.entity();

        v_flex().gap_2().child(
            div()
                .h(px(7.))
                .w_full()
                .rounded_full()
                .bg(rgb_to_u32(226, 232, 240))
                .cursor_pointer()
                .on_prepaint({
                    let progress_bar_entity = progress_bar_entity.clone();
                    move |bounds: Bounds<Pixels>, _: &mut Window, cx: &mut App| {
                        let _ = progress_bar_entity.update(cx, |this, _| {
                            this.progress_bar_bounds = Some(bounds);
                        });
                    }
                })
                .id("video_progress_bar")
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, event: &MouseDownEvent, _, _| {
                        if let Some(bounds) = this.progress_bar_bounds {
                            if let Some(target) = this.get_progress_position(event.position, bounds)
                            {
                                this.seek_video_progress(target);
                                this.is_dragging_progress_bar = false;
                                this.pending_seek_position = None;
                            }
                        }
                    }),
                )
                .on_drag(ProgressDrag, |_, _, _, cx: &mut App| cx.new(|_| Empty))
                .on_drag_move::<ProgressDrag>(cx.listener(
                    |this, event: &DragMoveEvent<ProgressDrag>, _, _| {
                        if let Some(target) =
                            this.get_progress_position(event.event.position, event.bounds)
                        {
                            this.is_dragging_progress_bar = true;
                            this.pending_seek_position = Some(target);
                        }
                    },
                ))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _, _, _| {
                        if this.is_dragging_progress_bar {
                            if let Some(target) = this.pending_seek_position.take() {
                                this.seek_video_progress(target);
                            }
                            this.is_dragging_progress_bar = false;
                        }
                    }),
                )
                .on_mouse_up_out(
                    MouseButton::Left,
                    cx.listener(|this, _, _, _| {
                        if this.is_dragging_progress_bar {
                            if let Some(target) = this.pending_seek_position.take() {
                                this.seek_video_progress(target);
                            }
                            this.is_dragging_progress_bar = false;
                        }
                    }),
                )
                .child(
                    div()
                        .h(px(7.))
                        .w(px(progress_bar_width * progress_ratio))
                        .rounded_full()
                        .bg(rgb_to_u32(59, 130, 246)),
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
            .size(px(36.))
            .rounded_full()
            .bg(rgb_to_u32(248, 250, 252))
            .border_1()
            .border_color(rgb_to_u32(226, 232, 240))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(13.))
            .text_color(rgb_to_u32(15, 23, 42))
            .cursor_pointer()
            .hover(|style| {
                style
                    .bg(rgb_to_u32(239, 246, 255))
                    .border_color(rgb_to_u32(147, 197, 253))
                    .text_color(rgb_to_u32(37, 99, 235))
            })
            .id(id)
            .on_click(click)
            .child(label)
    }

    pub(crate) fn player_control_ui(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .p_1()
            .rounded_full()
            .bg(rgb_to_u32(241, 245, 249))
            .child(self.render_control_button(
                "video_prev_button",
                "<",
                cx.listener(|this, _, _, cx| {
                    this.prev_video(cx);
                }),
            ))
            .child(self.render_control_button(
                "video_play_button",
                if self.play_state == PlatState::Playing {
                    div().child("◼")
                } else {
                    div().child("▶")
                },
                cx.listener(|this, _, _, cx| {
                    this.toggle_play(cx);
                }),
            ))
            .child(self.render_control_button(
                "video_next_button",
                ">",
                cx.listener(|this, _, _, cx| {
                    this.next_video(cx);
                }),
            ))
    }

    pub(crate) fn video_frame_ui(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let frame_aspect = self.video_frame_size.max(0.01);
        let fitted_frame_size = self.video_frame_bounds.map(|bounds| {
            let container_width = bounds.size.width.as_f32().max(1.0);
            let container_height = bounds.size.height.as_f32().max(1.0);
            let container_aspect = container_width / container_height;

            if container_aspect > frame_aspect {
                (container_height * frame_aspect, container_height)
            } else {
                (container_width, container_width / frame_aspect)
            }
        });

        div()
            .flex_grow_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .relative()
            .justify_center()
            .items_center()
            .overflow_hidden()
            .rounded_xl()
            .bg(rgb_to_u32(15, 23, 42))
            .border_1()
            .border_color(rgb_to_u32(30, 41, 59))
            .on_prepaint({
                let video_frame_entity = cx.entity();
                move |bounds: Bounds<Pixels>, _: &mut Window, cx: &mut App| {
                    let _ = video_frame_entity.update(cx, |this, cx| {
                        let changed = this
                            .video_frame_bounds
                            .map(|current| {
                                current.size.width != bounds.size.width
                                    || current.size.height != bounds.size.height
                            })
                            .unwrap_or(true);

                        if changed {
                            this.video_frame_bounds = Some(bounds);
                            cx.notify();
                        }
                    });
                }
            })
            .child(if let Some(frame) = self.render_image.clone() {
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        if let Some((frame_width, frame_height)) = fitted_frame_size {
                            img(frame)
                                .w(px(frame_width))
                                .h(px(frame_height))
                                .object_fit(ObjectFit::Cover)
                                .into_any_element()
                        } else {
                            img(frame)
                                .size_full()
                                .object_fit(ObjectFit::Cover)
                                .into_any_element()
                        },
                    )
                    .into_any_element()
            } else {
                v_flex()
                    .absolute()
                    .inset_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        div().px_4().child(
                            markdown(match &self.play_state {
                                PlatState::Playing => "".to_string(),
                                PlatState::Paused => "".to_string(),
                                PlatState::Loading => "加载中".to_string(),
                                PlatState::UnLoading => "没有加载播放来源".to_string(),
                                PlatState::Error(err) => err.to_string(),
                                PlatState::Cache(val) => val.clone(),
                            })
                            .selectable(true)
                            .text_color(rgb(0xCBD5E1))
                            .cursor_text(),
                        ),
                    )
                    .into_any_element()
            })
    }
}
