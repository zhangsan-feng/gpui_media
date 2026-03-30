use std::fs::DirEntry;
use std::sync::Arc;
use gpui::*;
use log::info;
use uuid::Uuid;
use crate::entity::{MusicConvertLayer, PlatformInterface};
use crate::state::{GlobalState, StateEvent};




struct LocalMusicImpl;
impl PlatformInterface for LocalMusicImpl {
    fn download(&self, params: &MusicConvertLayer) -> Result<MusicConvertLayer> {
        Ok(params.clone())
    }
}


pub struct LoadLocalMusicPage {
    load_music_path: Vec<DirEntry>,
}

impl LoadLocalMusicPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>)->LoadLocalMusicPage {
        LoadLocalMusicPage {
            load_music_path: vec![],
        }
    }

    pub fn handler_local_music_file(&self, cx: &mut Context<Self>){

        let mut cx_async = cx.to_async().clone();
        let state_handle = cx.global::<GlobalState>().0.clone();
        let music_file = &self.load_music_path;

        let mut call_data = Vec::new();

        for file in music_file {
            call_data.push(MusicConvertLayer{
                music_id: Uuid::new_v4().to_string(),
                music_name:file.file_name().to_string_lossy().to_string(),
                music_author: "".to_string(),
                music_pic: "".to_string(),
                music_platform: "".to_string(),
                music_time: "".to_string(),
                music_source: "".to_string(),
                music_file: file.path().to_string_lossy().to_string(),
                func: Arc::new(LocalMusicImpl),
            })
        }

        info!("call_data:{}",call_data.len());
        cx.spawn(|_,_:&mut AsyncApp| async  move{
            state_handle.update(&mut cx_async, |_, cx| {
                cx.emit(StateEvent::UpdatePlatyList(call_data));
            });

        }).detach();


    }
}

impl Render for LoadLocalMusicPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .p_4()
            .child(
                div()
                    .text_center()
                    .text_size(px(40.))
                    .w_full()
                    .h(px(80.))
                    .border_2()
                    .border_color(rgb(0x9999AF))
                    .cursor_pointer()
                    .overflow_hidden()
                    .child("+")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            let mut async_cx = cx.to_async();
                            let entity_view = cx.entity();

                            cx.spawn(|_, _: &mut AsyncApp| async move {
                                let paths = entity_view.update(&mut async_cx, |this, cx| {
                                    cx.prompt_for_paths(PathPromptOptions {
                                        files: true,
                                        directories: true,
                                        multiple: true,
                                        prompt: None,
                                    })
                                });

                                // info!("{:#?} ", paths);

                                if let Ok(result) = paths.await {
                                    if let Ok(Some(path_vec)) = result {
                                        if let Some(first_path) = path_vec.first() {
                                            let path_str = first_path.to_string_lossy().to_string();
                                            info!("{}", path_str);
                                            if let Ok(dir) = std::fs::read_dir(path_str) {
                                                for file in dir {
                                                    if let Ok(f) = file{
                                                        if !f.path().is_file(){
                                                            continue
                                                        }
                                                        entity_view.update(&mut async_cx, |this, cx| {
                                                            this.load_music_path.push(f)
                                                        });

                                                    }
                                                }
                                            }
                                            entity_view.update(&mut async_cx, |this, cx| {
                                               this.handler_local_music_file(cx);
                                            });


                                        }
                                    }
                                }
                            }).detach();
                        })
                    )
            )

    }
}