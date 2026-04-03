use crate::entity::MusicConvertLayer;
use crate::music_platform::dtwav_music::recommend::request_web;
use log::info;

pub async fn call(keyword: &str) -> anyhow::Result<Vec<MusicConvertLayer>> {
    let mut call_back = Vec::new();

    match request_web(format!("https://dtwav.com/src/?keyword={}", keyword).as_str()).await {
        Ok(val) => call_back.extend(val),
        Err(e) => {
            info!("err:{}", e)
        }
    }
    Ok(call_back)
}
