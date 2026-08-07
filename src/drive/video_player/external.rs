use std::sync::{Arc, Mutex};
use gpui::{App, AppContext, EntityId, TitlebarOptions, Window, WindowId};
use gpui_component::Root;
use crate::component::window::window_center_settings;
use crate::drive::video_player::VideoPlayer;

impl VideoPlayer{

    pub(crate) fn open_window(window: &mut Window, cx: &mut App) -> (WindowId, EntityId) {
        let player_entity_id = Arc::new(Mutex::new(None));
        let player_entity_id_for_window = player_entity_id.clone();
        let mut options = window_center_settings(window, 1300., 700.);
        options.titlebar = Some(TitlebarOptions {
            title: None,
            // Hide the extractor titlebar; HomeView renders the compatible custom one.
            appears_transparent: true,
            traffic_light_position: None,
        });
        let handler = cx
            .open_window(
                options,
                move |window, app| {
                    let view = app.new(|cx| VideoPlayer::new(window, cx));
                    *player_entity_id_for_window.lock().unwrap() = Some(view.entity_id());
                    app.new(|cx| Root::new(view, window, cx))
                },
            )
            .expect("open window failed");
        let player_entity_id = player_entity_id
            .lock()
            .unwrap()
            .expect("video player entity was not created");
        (handler.window_id(), player_entity_id)
    }
}