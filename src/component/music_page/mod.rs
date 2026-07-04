use crate::drive;
use crate::drive::music_player::MusicPlayer;
use crate::music_platform;
use crate::state::{GlobalState, StateEvent};
use gpui::*;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::{
    StyleSized, StyledExt, VirtualListScrollHandle, h_flex, v_flex, v_virtual_list,
};
use log::info;
use std::rc::Rc;

#[derive(Clone)]
pub struct MusicRecommendPage {
    music_data: Vec<drive::NetworkStatic>,
    hovered_id: Option<String>,
    is_loading: bool,
    vm_scroll_handle: VirtualListScrollHandle,
    music_player_page: Entity<MusicPlayer>,
    music_search_keyword: Entity<InputState>,
}

impl MusicRecommendPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> MusicRecommendPage {
        let mut s = MusicRecommendPage {
            music_data: Vec::new(),
            hovered_id: None,
            is_loading: false,
            vm_scroll_handle: VirtualListScrollHandle::new(),
            music_player_page: cx.new(|cx| MusicPlayer::new(window, cx)),
            music_search_keyword: cx
                .new(|cx| InputState::new(window, cx).placeholder("input search music")),
        };
        s.init_data(cx);
        s
    }

    pub fn init_data(&mut self, cx: &mut Context<Self>) {
        let global_state = cx.global::<GlobalState>().0.read(cx).clone();
        let entity = cx.entity().clone();
        let mut cx_async = cx.to_async().clone();
        let state_handler = cx.global::<GlobalState>().0.clone();

        self.is_loading = true;

        cx.spawn(|_, _: &mut AsyncApp| async move {
            let res = global_state
                .tokio_handle
                .spawn(async move { music_platform::music_recommend().await });

            match res.await {
                Ok(Ok(r)) => {
                    entity.update(&mut cx_async, |this, cx| {
                        this.is_loading = false;
                        this.music_data = r.clone();

                        cx.notify()
                    });
                    state_handler.update(&mut cx_async, |_, cx| {
                        cx.emit(StateEvent::UpdateMusicPlatyList(r));
                    });
                }
                Ok(Err(e)) => info!("http error: {:?}", e),
                Err(e) => info!("tokio runtime error: {:?}", e),
            }
        })
        .detach();
    }

    fn vm_btn_play_music(
        &self,
        data: drive::NetworkStatic,
        index: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        Button::new(("music-play-index-", index))
            .label("播放")
            .on_click({
                let c = data.clone();
                cx.listener(move |_, _, _, cx| {
                    let mut cx_async = cx.to_async().clone();
                    let state_handler = cx.global::<GlobalState>().0.clone();
                    let c = c.clone();
                    cx.spawn(|_, _: &mut AsyncApp| async move {
                        state_handler.update(&mut cx_async, |_, cx| {
                            cx.emit(StateEvent::TogglePlayMusic(c.clone()))
                        });
                    })
                    .detach()
                })
            })
    }

    fn vm_list(&self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_virtual_list(
            cx.entity().clone(),
            "recommend-music-vm-list",
            Rc::new(
                self.music_data
                    .iter()
                    .map(|_| size(px(100.), px(40.)))
                    .collect(),
            ),
            |view, visible_range, _, cx| {
                visible_range
                    .map(|index| {
                        let data = view.music_data[index].clone();
                        h_flex()
                            .justify_between()
                            .w_full()
                            .child(
                                div()
                                    .gap_2()
                                    .justify_between()
                                    .h_flex()
                                    .child(img(data.img.clone()).size(px(24.)).rounded_full())
                                    .child(data.author.clone())
                                    .child(data.name.clone()),
                            )
                            .child(view.vm_btn_play_music(data, index, cx))
                    })
                    .collect()
            },
        )
        .track_scroll(&self.vm_scroll_handle)
    }
}

impl Render for MusicRecommendPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_2()
            .p_2()
            .child(
                div()
                    .flex_grow(1.)
                    .gap_2()
                    .p_2()
                    .border_color(rgb(0xE2E8F0))
                    .border_1()
                    .rounded_2xl(),
            )
            .child(div().child(self.music_player_page.clone().into_any_element()))
    }
}
