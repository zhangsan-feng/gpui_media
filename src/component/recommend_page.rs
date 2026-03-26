use crate::music_platform;
use crate::music_platform::kugou_music::entity;
use crate::state::{GlobalState, StateEvent};
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use log::info;
use gpui_component::button::Button;
use crate::component::home::rgb_to_u32;
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
        let mut cx_async = cx.to_async().clone();
        let entity = cx.entity().clone();

        cx.spawn(|_, _: &mut AsyncApp| async move {
            let res = global_state
                .tokio_handle
                .spawn(async move { music_platform::music_recommend("1").await });

            match res.await {
                Ok(Ok(r)) => {
                    entity.update(&mut cx_async, |this, cx|{
                        this.music_data = r;
                        cx.notify()
                    })
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
            .h(px(500.))
            .justify_center()
            .child(
                div()
                    .overflow_y_scrollbar()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap_6()
                    .p_4()
                    .children(self.music_data.iter().enumerate().map(|(index, data)| {
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .gap_2()
                            .w_full()
                            .border_color(rgb_to_u32(194, 213, 242))
                            .child(img(data.music_pic.clone()).size(px(50.)))
                            .child(data.music_name.clone())
                            .child(
                                Button::new(("music-play-index-", index))
                                    .label("播放")
                                    .on_click({
                                        let c = data.clone();
                                        cx.listener(move |_, _, _ , cx|{

                                            let c = c.clone();
                                            let mut async_cx = cx.to_async().clone();
                                            let state_handle = cx.global::<GlobalState>().0.clone();
                                            let tokio_handler = state_handle.read(cx).clone().tokio_handle;

                                            let res = tokio_handler.spawn(async move {
                                                c.download()
                                            });
                                            cx.spawn(|_, _:&mut AsyncApp| async move {
                                                match res.await {
                                                    Ok(Ok(val)) => {
                                                        state_handle.update(&mut async_cx, |_, cx| {
                                                            cx.emit(StateEvent::TogglePlayMusic(val));
                                                        });
                                                    },
                                                    Ok(Err(e)) => {
                                                        info!("http error: {:?}", e);
                                                    },
                                                    Err(e) => {
                                                        info!("tokio runtime error: {:?}", e);
                                                    }
                                                }
                                            }).detach();

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
