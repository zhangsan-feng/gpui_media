use crate::platform::Platform;
use gpui::*;
use gpui_component::v_flex;

pub struct Gui {
    message: String,
}

impl Gui {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        let platform = Platform::new();
        let message = platform
            .dependency_error()
            .unwrap_or_else(|| "GStreamer runtime is valid. Restart the launcher.".to_string());
        Self { message }
    }
}

impl Render for Gui {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .p_8()
            .gap_4()
            .bg(rgb(0xF8FAFC))
            .child(
                div()
                    .text_size(px(22.))
                    .text_color(rgb(0x0F172A))
                    .child("GStreamer runtime is incomplete"),
            )
            .child(
                div()
                    .text_size(px(14.))
                    .text_color(rgb(0x475569))
                    .child(self.message.clone()),
            )
            .child(
                div()
                    .text_size(px(13.))
                    .text_color(rgb(0x64748B))
                    .child("Re-extract the complete gpui-medio release package."),
            )
    }
}
