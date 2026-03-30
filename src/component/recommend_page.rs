use crate::music_platform;
use crate::state::{GlobalState, StateEvent};
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use log::info;
use gpui_component::button::Button;
use gpui_component::StyledExt;
use crate::entity::MusicConvertLayer;


#[derive(Clone)]
pub struct RecommendPage {
    music_data: Vec<MusicConvertLayer>,
    hovered_id: Option<String>,
}

impl RecommendPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> RecommendPage {
        let s = RecommendPage {
            music_data: Vec::new(),
            hovered_id: None,
        };
        s.init_component_data(cx);
        s
    }

    pub fn init_component_data(&self, cx: &mut Context<Self>) {
        let global_state = cx.global::<GlobalState>().0.read(cx).clone();
        let entity = cx.entity().clone();
        let mut cx_async = cx.to_async().clone();
        let state_handle = cx.global::<GlobalState>().0.clone();

        cx.spawn(|_, _: &mut AsyncApp| async move {
            let res = global_state
                .tokio_handle
                .spawn(async move { music_platform::music_recommend().await });

            match res.await {
                Ok(Ok(r)) => {
                    entity.update(&mut cx_async, |this, cx|{
                        this.music_data = r.clone();
                        cx.notify()
                    });
                    state_handle.update(&mut cx_async, |_, cx| {
                        cx.emit(StateEvent::UpdatePlatyList(r));
                    });
                },
                Ok(Err(e)) => info!("http error: {:?}", e),
                Err(e) => info!("tokio runtime error: {:?}", e),
            }
        })
        .detach();
    }
}

impl Render for RecommendPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {

        div()
            .h(px(580.))
            .justify_center()
            .child(
                div()
                    .overflow_y_scrollbar()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap_2()
                    .p_4()
                    .children(self.music_data.iter().enumerate().map(|(index, data)| {
                        div()
                            .flex()
                            .justify_between()
                            .w_full()
                            .pr_2()
                            .child(
                                div()
                                    .gap_2()
                                    .justify_between()
                                    .h_flex()
                                    .child(img(data.music_pic.clone()).size(px(24.)).rounded_full())
                                    .child(data.music_author.clone())
                                    .child(data.music_platform.clone())
                                    .child(data.music_name.clone())
                            )
                            .child(
                                Button::new(("music-play-index-", index))
                                    .label("播放")
                                    .on_click({
                                        let c = data.clone();
                                        cx.listener(move |_, _, _ , cx|{

                                            let mut cx_async = cx.to_async().clone();
                                            let state_handle = cx.global::<GlobalState>().0.clone();
                                            let c = c.clone();
                                            cx.spawn(|_, _:&mut AsyncApp| async move {
                                                state_handle.update(&mut cx_async, |_, cx| {
                                                    cx.emit(StateEvent::TogglePlayMusic(c.clone()))
                                                });
                                            }).detach()
                                        })

                                    })
                            )
                    }))
            )

        // .child(
        //     div()
        //         .p_4()
        //         .gap_4()
        //         .justify_center()
        //         .flex()
        //         .child(
        //             Button::new("").label("prev")
        //         )
        //         .child(
        //             Button::new("").label("next")
        //         )
        // )
    }
}
