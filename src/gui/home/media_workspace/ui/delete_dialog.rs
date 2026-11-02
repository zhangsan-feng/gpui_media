use super::*;
use gpui_kit::component::{
    WindowExt,
    button::{Button, ButtonVariants},
    dialog::{Cancel, Confirm, DialogFooter},
};
use player_core::PlayStatic;
use std::path::PathBuf;

impl MediaWorkspace {
    pub(in crate::gui::home::media_workspace) fn open_playlist_delete_dialog(
        &self,
        item: PlayStatic,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let workspace = cx.entity().downgrade();
        log::info!(
            "[media-editor:playlist-delete] opening app confirmation: item_id={} path={}",
            item.id,
            path.display()
        );
        window.open_dialog(cx, move |dialog, window, _| {
            let workspace = workspace.clone();
            let confirmed_item = item.clone();
            let confirmed_path = path.clone();
            let canceled_id = item.id.clone();
            let canceled_path = path.clone();
            let file_title = item.title.clone();
            // Soft wrap opportunities keep long Windows paths readable inside the card.
            let display_path = path
                .display()
                .to_string()
                .chars()
                .fold(String::new(), |mut text, character| {
                    text.push(character);
                    if matches!(character, '\\' | '/' | '_' | '-') {
                        text.push('\u{200b}');
                    }
                    text
                });
            dialog
                .title(div().text_size(px(18.)).child("删除磁盘文件？"))
                .w(px(520.))
                .margin_top(window.viewport_size().height * 0.22)
                .rounded_xl()
                .p_5()
                .overlay_closable(false)
                .content(move |content, _, _| {
                    content
                        .gap_3()
                        .py_2()
                        .text_size(px(14.))
                        .child(
                            div()
                                .text_color(rgb_to_u32(100, 116, 139))
                                .child("文件将从磁盘中永久删除，此操作无法撤销。"),
                        )
                        .child(
                            v_flex()
                                .gap_2()
                                .p_3()
                                .min_w_0()
                                .rounded_lg()
                                .bg(rgb_to_u32(248, 250, 252))
                                .child(
                                    div()
                                        .w_full()
                                        .whitespace_normal()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(file_title.clone()),
                                )
                                .child(
                                    div()
                                        .w_full()
                                        .whitespace_normal()
                                        .text_size(px(12.))
                                        .text_color(rgb_to_u32(100, 116, 139))
                                        .child(display_path.clone()),
                                ),
                        )
                })
                .footer(
                    DialogFooter::new()
                        .pt_3()
                        .child(
                            Button::new("playlist-delete-cancel")
                                .label("取消")
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(Box::new(Cancel), cx);
                                }),
                        )
                        .child(
                            Button::new("playlist-delete-confirm")
                                .danger()
                                .label("删除文件")
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(
                                        Box::new(Confirm { secondary: false }),
                                        cx,
                                    );
                                }),
                        ),
                )
                .on_ok(move |_, _, cx| {
                    let _ = workspace.update(cx, |this, cx| {
                        this.delete_confirmed_playlist_item(
                            confirmed_item.clone(),
                            confirmed_path.clone(),
                            cx,
                        );
                    });
                    true
                })
                .on_cancel(move |_, _, _| {
                    log::info!(
                        "[media-editor:playlist-delete] canceled app confirmation: item_id={} path={}",
                        canceled_id,
                        canceled_path.display()
                    );
                    true
                })
        });
    }
}
