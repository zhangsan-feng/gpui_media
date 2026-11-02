use super::CustomSidebarMenu;
use crate::gui::home::Page;
use gpui_kit::*;

impl CustomSidebarMenu {
    pub(super) fn toggle_collapsed(&mut self, cx: &mut Context<Self>) {
        self.collapsed = !self.collapsed;
        log::info!("[sidebar] collapsed={}", self.collapsed);
        cx.notify();
    }

    pub(super) fn select_page(&mut self, page: Page, cx: &mut Context<Self>) {
        self.previous_select_id = self.select_id;
        self.select_id = page;
        cx.notify();
    }
}
