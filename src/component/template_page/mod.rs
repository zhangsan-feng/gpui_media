mod html;
mod api;
mod entity;

use std::rc::Rc;
use gpui::*;
use gpui_component::{h_flex, v_virtual_list, StyledExt, VirtualListScrollHandle};

pub struct TemplatePage{
    template_data:Vec<String>,
    vm_scroll_handle:VirtualListScrollHandle
}

impl TemplatePage{
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self{
        Self{
            template_data:Vec::from([]),
            vm_scroll_handle:VirtualListScrollHandle::new()
        }
    }


    fn vm_list(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement{
        v_virtual_list(
            cx.entity().clone(),
            "template-vm-list",
            Rc::new(
                self.template_data
                    .iter()
                    .map(|_| size(px(100.), px(40.)))
                    .collect(),
            ),
            |view, visible_range, _, cx| {
                visible_range
                    .map(|index| {
                        div()
                    })
                    .collect()
            },
        ).track_scroll(&self.vm_scroll_handle)
    }

    fn template_list(){}

    fn add_template_window(){}

    fn edit_template_window(){}

    fn remove_template(){}

}










impl Render for TemplatePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .p_2()
            .gap_2()

    }
}
