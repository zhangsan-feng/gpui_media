mod gui;
mod platform;
use crate::gui::Gui;
use crate::platform::Platform;
use gpui::{AppContext, Render, Styled, px, size};
use gpui_component::Root;

#[tokio::main]
async fn main() {
    let platform = Platform::new();
    if platform.check_dependencies() {
        let _ = platform.start_app();
        std::process::exit(0);
    }

    let app = gpui_platform::application();
    app.run(move |cx| {
        let mut window_options = gpui::WindowOptions::default();
        let window_size = size(px(600.), px(280.));
        window_options.window_bounds = Some(gpui::WindowBounds::centered(window_size, cx));
        window_options.window_min_size = Some(window_size);
        window_options.is_resizable = false;
        window_options.titlebar = Some(gpui::TitlebarOptions {
            title: None,
            appears_transparent: false,
            traffic_light_position: None,
        });
        cx.open_window(window_options, move |window, app| {
            gpui_component::init(app);
            let view = app.new(|cx| Gui::new(window, cx));
            app.new(|cx| Root::new(view, window, cx))
        })
        .expect("Failed to create dependency window");
    });
}
