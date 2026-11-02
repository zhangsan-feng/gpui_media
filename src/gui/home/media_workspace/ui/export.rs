use super::super::MediaWorkspace;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::{Disableable, IconName};
use gpui_kit::*;
use player_core::{PlayCoreMediaType, PlayCoreTranscodeFormat};

impl MediaWorkspace {
    pub(in crate::gui::home::media_workspace) fn render_export_menu(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = self.play_core.read(cx)._view_state();
        let busy = self.export_in_progress;
        let whole_disabled =
            busy || state.player.url.is_empty() || state.media_type == PlayCoreMediaType::Unknown;
        let clip_disabled = whole_disabled
            || self.clip_range(cx).is_none()
            || !state.player.url.starts_with("file://");
        let batch_disabled = busy || self.play_list.is_empty();
        let formats = if state.media_type == PlayCoreMediaType::Audio {
            vec![
                PlayCoreTranscodeFormat::Mp3,
                PlayCoreTranscodeFormat::FLAC,
                PlayCoreTranscodeFormat::WAV,
                PlayCoreTranscodeFormat::AAC,
            ]
        } else {
            vec![
                PlayCoreTranscodeFormat::Mp4,
                PlayCoreTranscodeFormat::Mkv,
                PlayCoreTranscodeFormat::MOV,
            ]
        };
        let editor = cx.entity();
        Button::new("media-editor-export")
            .label(if busy { "导出中…" } else { "导出" })
            .icon(IconName::ArrowDown)
            .primary()
            .dropdown_caret(true)
            .disabled(busy || (state.player.url.is_empty() && self.play_list.is_empty()))
            .dropdown_menu_with_anchor(Anchor::BottomRight, move |mut menu, _, _| {
                for (scope, label, disabled) in [
                    (0, "导出剪辑结果（拼接）", clip_disabled),
                    (1, "导出当前完整素材", whole_disabled),
                    (2, "批量导出素材库", batch_disabled),
                ] {
                    if scope > 0 {
                        menu = menu.separator();
                    }
                    menu = menu.item(PopupMenuItem::label(label));
                    for format in &formats {
                        let format = *format;
                        let editor = editor.clone();
                        menu = menu.item(
                            PopupMenuItem::new(super::super::core::export_format_label(format))
                                .disabled(disabled)
                                .on_click(move |_, _, app| {
                                    let _ = editor.update(app, |this, cx| match scope {
                                        0 => this.export_clip(format, cx),
                                        1 => this.export_media(format, cx),
                                        _ => this.export_playlist(format, cx),
                                    });
                                }),
                        );
                    }
                }
                menu
            })
            .into_any_element()
    }
}
