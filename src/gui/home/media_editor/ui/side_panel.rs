use super::super::{MediaEditor, SidePanelState};
use crate::component::color::rgb_to_u32;
use gpui_kit::component::input::Input;
use gpui_kit::component::menu::{ContextMenuExt, PopupMenuItem};
use gpui_kit::component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarMode};
use gpui_kit::component::{IconName, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use player_core::{PlayCoreMediaType, PlayCoreViewState};
use std::rc::Rc;
use std::time::Duration;

impl MediaEditor {
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
                    .items_center()
                    .justify_between()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("素材库"))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb_to_u32(148, 140, 163))
                            .child(format!("{} 个素材", self.play_list.len())),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(self._render_control_button(
                        "media-editor-import-files",
                        h_flex().gap_1().child(IconName::Plus).child("导入素材"),
                        true,
                        |this, _, _, cx| this.import_media_files(cx),
                        cx,
                    ))
                    .child(self._render_control_button(
                        "media-editor-import-folder",
                        IconName::FolderOpen,
                        false,
                        |this, _, _, cx| this.select_media_folder(cx),
                        cx,
                    )),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(Input::new(&self.network_url_input).cleanable(true)),
                    )
                    .child(self._render_control_button(
                        "media-editor-import-url",
                        "导入",
                        false,
                        |this, _, window, cx| this._play_network_url(window, cx),
                        cx,
                    )),
            )
            .child(div().flex_1().min_h(px(80.)).child(playlist_content))
            .child(
                div()
                    .id("media-editor-source-info")
                    .max_h(px(180.))
                    .overflow_y_scroll()
                    .border_t_1()
                    .border_color(rgb_to_u32(231, 220, 235))
                    .pt_3()
                    .child(project_info),
            )
            .into_any_element()
    }
    pub(super) fn _render_media_queue(&self, cx: &mut Context<Self>) -> AnyElement {
        let playlist_content = self._render_playlist_content(cx);
        let project_info = self._render_project_info(cx);
        let media_queue = self._render_media_queue_content(playlist_content, project_info, cx);

        self._render_side_panel(media_queue)
    }

    fn _render_side_panel(&self, media_queue: AnyElement) -> AnyElement {
        let side_panel_state = self.side_panel_state;
        let animation_id = self.side_panel_animation_id;
        let panel_width = 300.0;
        let panel = div()
            .h_full()
            .w(px(panel_width))
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
            SidePanelState::Closed => panel.w(px(0.)).opacity(0.).into_any_element(),
            SidePanelState::Opening | SidePanelState::Closing => {
                let opening = side_panel_state == SidePanelState::Opening;
                panel
                    .with_animation(
                        format!("media-editor-library-{animation_id}"),
                        Animation::new(Duration::from_millis(500)).with_easing(ease_in_out),
                        move |el, delta| {
                            let progress = if opening { delta } else { 1.0 - delta };
                            el.w(px(panel_width * progress)).opacity(progress)
                        },
                    )
                    .into_any_element()
            }
        }
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
        v_flex()
            .size_full()
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
            .child(self._render_media_info(&view_state))
            .into_any_element()
    }

    fn _render_media_info(&self, view_state: &PlayCoreViewState) -> AnyElement {
        let media_type = match view_state.media_type {
            PlayCoreMediaType::Audio => "音乐",
            PlayCoreMediaType::Video => "视频",
            PlayCoreMediaType::Unknown => "未知",
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
        let codec = view_state.codec.as_deref().unwrap_or("未知").to_string();

        v_flex()
            .w_full()
            .gap_2()
            .child(self._render_project_info_row("媒体类型", media_type.to_string()))
            .child(self._render_project_info_row("分辨率", resolution))
            .child(self._render_project_info_row("帧率", frame_rate))
            .child(self._render_project_info_row("编码", codec))
            .into_any_element()
    }

    fn _render_project_info_row(&self, label: &str, value: String) -> AnyElement {
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
                    .child(label.to_string()),
            )
            .child(
                div()
                    .flex_1()
                    .text_size(px(12.))
                    .text_color(rgb_to_u32(73, 66, 92))
                    .text_ellipsis()
                    .child(value),
            )
            .into_any_element()
    }

    pub(super) fn _render_export_status(&self) -> AnyElement {
        let Some(status) = &self.export_status else {
            return div().into_any_element();
        };
        let (message, color) = match status {
            super::super::ExportStatus::Working(message) => {
                (message.as_str(), rgb_to_u32(148, 140, 163))
            }
            super::super::ExportStatus::Success(message) => {
                (message.as_str(), rgb_to_u32(41, 139, 78))
            }
            super::super::ExportStatus::Error(message) => {
                (message.as_str(), rgb_to_u32(190, 48, 90))
            }
        };

        div()
            .w_full()
            .text_size(px(12.))
            .text_color(color)
            .text_ellipsis()
            .child(message.to_owned())
            .into_any_element()
    }

    fn _render_playlist_content(&self, cx: &mut Context<Self>) -> AnyElement {
        let play_list = Rc::new(self.play_list.clone());
        let current_index = self.current_index;
        let player = cx.entity().clone();
        let play_list_state = self.play_list_state.clone();
        let delete_error = self.playlist_delete_error.clone();

        if play_list.is_empty() {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(13.))
                .text_color(rgb_to_u32(148, 140, 163))
                .child("导入或拖入视频、音频开始剪辑")
                .into_any_element()
        } else {
            // Fixed-height virtualization keeps total extent stable across redraws and resizes.
            let playlist_list = uniform_list(
                "media_editor-playlist",
                play_list.len(),
                move |range, _, _| {
                    range
                        .map(|index| {
                            let Some(item) = play_list.get(index).cloned() else {
                                return div().into_any_element();
                            };
                            let is_current = current_index == Some(index);
                            let delete_item = item.clone();
                            let is_local_file = item.url.starts_with("file://");
                            let title = if !item.title.trim().is_empty() {
                                item.title.clone()
                            } else if !item.id.trim().is_empty() {
                                item.id.clone()
                            } else {
                                format!("播放项 {}", index + 1)
                            };
                            let play_player = player.clone();
                            let menu_player = player.clone();

                            div()
                                .id(("media-editor-play-list-item", index))
                                .w_full()
                                .h(px(44.))
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
                                    let _ = play_player.update(app, |this, cx| {
                                        if !this.playlist_delete_in_progress {
                                            this._play_item(index, cx);
                                        }
                                    });
                                })
                                .context_menu(move |menu, _, _| {
                                    let menu_player = menu_player.clone();
                                    let delete_item = delete_item.clone();
                                    menu.item(
                                        PopupMenuItem::new("删除磁盘文件")
                                            .disabled(!is_local_file)
                                            .on_click(move |_, window, app| {
                                                let _ = menu_player.update(app, |this, cx| {
                                                    this.confirm_delete_playlist_item(
                                                        delete_item.clone(),
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            }),
                                    )
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
                        .collect()
                },
            )
            .track_scroll(&play_list_state)
            .size_full();

            v_flex()
                .size_full()
                .min_h_0()
                .gap_2()
                .when_some(delete_error, |this, error| {
                    this.child(
                        div()
                            .flex_shrink_0()
                            .text_size(px(11.))
                            .text_color(rgb_to_u32(190, 48, 90))
                            .child(error),
                    )
                })
                .child(
                    h_flex()
                        .flex_1()
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
                            div().w(px(16.)).h_full().child(
                                Scrollbar::vertical(&play_list_state)
                                    .mode(ScrollbarMode::Always)
                                    .axis(ScrollbarAxis::Vertical)
                                    .viewport_from_layout(),
                            ),
                        ),
                )
                .into_any_element()
        }
    }
}
