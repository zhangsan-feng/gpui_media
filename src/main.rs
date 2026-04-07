mod com;
mod component;
mod entity;
mod music_platform;
mod state;

use gpui::*;
use gpui_component::*;
use log::{Level, info};
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use crate::state::{GlobalState, State};
use reqwest_client::ReqwestClient;
use rust_embed::RustEmbed;

pub fn logger_init(logger_path: &str) {
    let date = chrono::Local::now().format("%Y-%m-%d");
    // let logfile_path = format!("{}{}.log", logger_path, date);
    // let mut logfile_path = PathBuf::from(logger_path);
    // logfile_path.push(format!("{}.log", date));
    //
    // if let Some(parent) = logfile_path.parent() {
    //     if !parent.exists() {
    //         let _ = std::fs::create_dir_all(parent);
    //     }
    // }

    // println!("{}", logfile_path);

    match std::fs::create_dir("./music") {
        Ok(e) => {}
        Err(e) => {}
    }

    let mut dispatch = fern::Dispatch::new();
    dispatch = dispatch
        .format(|out, message, record| {
            let file = record.file().unwrap_or("<unknown>");
            let line = record.line().unwrap_or(0);
            out.finish(format_args!(
                "[{}] [{}] [{}:{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                file,
                line,
                message
            ))
        })
        .filter(|metadata| metadata.level() == Level::Info)
        .level(log::LevelFilter::Info)
        .level(log::LevelFilter::Error)
        .level(log::LevelFilter::Trace)
        .level(log::LevelFilter::Warn)
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout());
    // .chain(fern::log_file(logfile_path).expect("Failed to create log file"));
    dispatch.apply().unwrap();

    info!("init logger success")
}

/*

https://longbridge.github.io/gpui-component/


*/

#[derive(RustEmbed)]
#[folder = "./src/icon"]
struct AssetFiles;

struct MergedAssets {
    local_directories: Vec<PathBuf>,
}

impl AssetSource for MergedAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        for dir in &self.local_directories {
            let full_path = dir.join(path);
            if full_path.exists() {
                let bytes = std::fs::read(full_path)?;
                return Ok(Some(Cow::Owned(bytes)));
            }
        }

        let clean_path = path.trim_start_matches("icon/").trim_start_matches("/");
        if let Some(file) = AssetFiles::get(clean_path) {
            return Ok(Some(file.data));
        }

        Ok(None)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut all_files = std::collections::HashSet::new();

        for dir in &self.local_directories {
            let full_path = dir.join(path);
            if full_path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(full_path) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            all_files.insert(name.to_string());
                        }
                    }
                }
            }
        }

        let clean_path = path.trim_start_matches("icon/").trim_start_matches("/");
        for file_path in AssetFiles::iter() {
            if file_path.starts_with(clean_path) {
                all_files.insert(file_path.to_string());
            }
        }

        Ok(all_files.into_iter().map(SharedString::from).collect())
    }
}

fn main() {
    logger_init("./");
    let http_client = ReqwestClient::user_agent("gpui").unwrap();
    let assets = MergedAssets {
        local_directories: vec![PathBuf::from("/"), PathBuf::from("./src/icon")],
    };

    let app = gpui_platform::application()
        .with_http_client(Arc::new(http_client))
        .with_assets(assets);

    app.run(move |cx| {
        let mut window_options = WindowOptions::default();
        window_options.window_bounds = Some(WindowBounds::centered(size(px(1200.), px(700.)), cx));
        window_options.window_min_size = Some(size(px(1200.), px(700.)));

        cx.open_window(window_options, |window, app| {
            // window.set_background_appearance(WindowBackgroundAppearance::Transparent);
            gpui_component::init(app);
            // let transparent = Theme::global(app).transparent;
            // Theme::global_mut(app).background = transparent;
            // state::new_state(app);

            let state_entity = app.new(|cx| State::new(cx));
            app.set_global(GlobalState(state_entity));
            let view = app.new(|cx| component::home::HomeView::new(window, cx));
            app.new(|cx| Root::new(view, window, cx))
        })
        .expect("Failed to create app");
    });
}
