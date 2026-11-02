mod core;
mod ui;

use crate::gui::home::Page;
use gpui_kit::*;

pub struct CustomSidebarMenu {
    pub select_id: Page,
    previous_select_id: Page,
    collapsed: bool,
}

impl CustomSidebarMenu {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            select_id: Page::MusicPage,
            previous_select_id: Page::MusicPage,
            collapsed: false,
        }
    }
}

impl Render for CustomSidebarMenu {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_sidebar(cx)
    }
}
