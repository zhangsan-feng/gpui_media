use gpui::*;
use gpui_component::scroll::{Scrollbar, ScrollbarAxis, ScrollbarShow};
use gpui_component::{h_flex, v_flex, Root, VirtualListScrollHandle, v_virtual_list};
use std::rc::Rc;
use crate::com::window_center_options;
use crate::component::home::rgb_to_u32;
use crate::drive::video_player::VideoPlayer;
use crate::entity::StreamMedioConvertLayer;
use crate::state::{GlobalState, StateEvent};
use crate::video_platform;

pub struct VideoRecommendPage{
    pub recommend_video:Vec<StreamMedioConvertLayer>,
    vm_scroll_handle: VirtualListScrollHandle,
}


impl VideoRecommendPage{
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> VideoRecommendPage{
        let s  = VideoRecommendPage{
            recommend_video:Vec::new(),
            vm_scroll_handle: VirtualListScrollHandle::new(),
        };

        // s.init_data(window, cx);
        s
    }

    pub fn init_data(&self, window: &mut Window, cx: &mut Context<Self>){
        let global_state = cx.global::<GlobalState>().0.clone();
        let tokio_handler = global_state.read(cx).tokio_handle.clone();
        let mut cx_async = cx.to_async().clone();
        let entity = cx.entity().clone();

        cx.spawn(|_,_:&mut AsyncApp| async move {
            let res = tokio_handler.spawn(async move {
                video_platform::recommend().await
            });
            match res.await {
                Ok(r)=> {
                    entity.update(&mut cx_async, |this, cx|{
                        this.recommend_video = r;
                        cx.notify()
                    })
                }
                Err(e)=>{
                    log::error!("{}", e)
                }
            }

        }).detach();
    }
}

impl VideoRecommendPage {
    const CARD_WIDTH: f32 = 150.;
    const CARD_HEIGHT: f32 = 180.;
    const CARD_GAP: f32 = 12.;
    const CARD_INSET: f32 = 8.;
    const RIGHT_SCROLLBAR_WIDTH: f32 = 15.;
    const LEFT_NAV_WIDTH: f32 = 80.;
    const PAGE_PADDING_X: f32 = 32.;
    const PAGE_GAP_X: f32 = 16.;

    fn compute_columns(&self, window: &Window) -> usize {
        let window_width = window.bounds().size.width.as_f32();
        let usable_width = (window_width
            - Self::LEFT_NAV_WIDTH
            - Self::PAGE_PADDING_X
            - Self::PAGE_GAP_X
            - Self::RIGHT_SCROLLBAR_WIDTH)
            .max(Self::CARD_WIDTH);
        let step = Self::CARD_WIDTH + Self::CARD_GAP;
        ((usable_width + Self::CARD_GAP) / step).floor().max(1.0) as usize
    }

    fn vm_list(&self, cx: &mut Context<Self>, columns_per_row: usize) -> impl IntoElement {
        let row_count = self.recommend_video.len().div_ceil(columns_per_row);
        let row_height = Self::CARD_HEIGHT + Self::CARD_GAP + (Self::CARD_INSET * 2.);

        v_virtual_list(
            cx.entity().clone(),
            "recommend-video-vm-list",
            Rc::new(
                (0..row_count)
                    .map(|_| size(px(Self::CARD_WIDTH), px(row_height)))
                    .collect(),
            ),
            move |view, visible_range, _, cx| {
                visible_range
                    .map(|row_index| {
                        let start = row_index * columns_per_row;
                        let end = ((row_index + 1) * columns_per_row).min(view.recommend_video.len());

                        h_flex()
                            .w_full()
                            .gap_3()
                            .p_2()
                            .children(
                                view.recommend_video[start..end]
                                    .iter()
                                    .cloned()
                                    .map(|data| {
                                        let _img = data.img.clone();
                                        let name = data.name.clone();
                                        v_flex()
                                            .id(data.id.to_string())
                                            .gap_2()
                                            .p_2()
                                            .cursor_pointer()
                                            .border_1()
                                            .border_color(rgb(0xE2E8F0))
                                            .bg(rgb_to_u32(248, 250, 252))
                                            .hover(|mut style| {
                                                style.background = Some(rgb_to_u32(226, 232, 240).into());
                                                style
                                            })
                                            .on_click(cx.listener(move |_, _, window, cx| {
                                                let data = data.clone();

                                                cx.open_window(window_center_options(window, 1300., 700.), move |window, app| {
                                                    let view = app.new(|cx| VideoPlayer::new(window, cx));
                                                    app.new(|cx| Root::new(view, window, cx))
                                                }).expect("TODO: panic message");

                                                let state_handler = cx.global::<GlobalState>().0.clone();
                                                let mut cx_async = cx.to_async().clone();
                                                cx.spawn(|_, _: &mut AsyncApp| async move {
                                                    state_handler.update(&mut cx_async, |_, cx| {
                                                        cx.emit(StateEvent::TogglePlayVideo(data.clone()))
                                                    });
                                                }).detach();
                                            }))
                                            .justify_center()
                                            .text_center()
                                            .w(px(Self::CARD_WIDTH))
                                            .h(px(Self::CARD_HEIGHT))
                                            .child(
                                                img(_img)
                                                    .size_full()
                                                    .object_fit(ObjectFit::Contain),
                                            )
                                            .child(name)
                                    }),
                            )
                    })
                    .collect()
            },
        )
        .track_scroll(&self.vm_scroll_handle)
    }
}

impl Render for VideoRecommendPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let columns_per_row = self.compute_columns(window);
        h_flex()
            .size_full()
            .p_4()
            .gap_4()
            .rounded_2xl()
            .child(
                div()
                    .flex_grow()
                    .h_full()
                    .child(self.vm_list(cx, columns_per_row)),
            )
            .child(
                div()
                    .h_full()
                    .w(px(15.))
                    .child(
                        Scrollbar::vertical(&self.vm_scroll_handle)
                            .scrollbar_show(ScrollbarShow::Always)
                            .axis(ScrollbarAxis::Vertical),
                    ),
            )
    }
}
