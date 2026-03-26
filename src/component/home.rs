use gpui::*;
use gpui_component::{h_flex, v_flex};
use crate::component::music_player::MusicPlayer;
use crate::component::recommend_page::RecommendPage;


#[derive(PartialEq, Clone, Copy)]
pub enum Page{
    RecommendPage,
    SearchPage,
    LocalPage
}


pub struct HomeView {
    select_id:Page,
    recommend_page:Entity<RecommendPage>,
    music_play_component:Entity<MusicPlayer>,
}

impl HomeView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> HomeView {
        let s =   HomeView {
            select_id:Page::RecommendPage,
            recommend_page:cx.new(|cx|{ RecommendPage::new(window, cx)}),
            music_play_component:cx.new(|cx|{MusicPlayer::new(window, cx)})
        };
        s
    }
}

pub fn rgb_to_u32(r: u8, g: u8, b: u8) -> Rgba {
    let color: u32 = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
    rgb(color)
}


impl HomeView {
    fn render_nav_item(&self, label: &'static str, page: Page, cx: &Context<Self>) -> impl Element {
        let is_selected = self.select_id == page.clone();

        div()
            .id(label)
            .child(label)
            .w_full()
            .h_10()
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .cursor_pointer()
            .hover(move |mut style| {
                if !is_selected {
                    style.background = Some(rgb_to_u32(220, 225, 233).into());
                }
                style
            })
            .on_click(cx.listener(move |this,_, _, cx| {
                this.select_id = page;
            }))
            .bg(if is_selected {
                rgb_to_u32(211, 227, 253)
            } else {
                rgb_to_u32(233, 238, 246)
            })
    }
}


impl Render for HomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .size_full()
            .child(
                v_flex()
                    .justify_center()
                    .p_4()
                    .gap_2()
                    .h_full()
                    .w(px(300.))
                    .bg(rgb_to_u32(233, 238, 246))
                    // .rounded_2xl()
                    .child(self.render_nav_item("歌曲推荐", Page::RecommendPage, cx))
                    .child(self.render_nav_item("歌曲搜索", Page::SearchPage, cx))
                    .child(self.render_nav_item("本地导入", Page::LocalPage, cx))
            )
            .child(
                    v_flex()
                    .size_full()
                    .child(
                        div()
                            .flex_grow()
                            .child(
                                match self.select_id {
                                        Page::RecommendPage => {
                                            self.recommend_page.clone().into_any_element()
                                        }
                                        Page::SearchPage => {
                                            div().into_any_element()
                                        }
                                        Page::LocalPage => {
                                            div().into_any_element()
                                        }
                                    }
                            )
                    )
                    // .child(Divider::horizontal().w_full())
                    .child(
                        self.music_play_component.clone()
                    )

            )


    }
}
