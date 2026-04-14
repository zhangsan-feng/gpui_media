

// https://api.bilibili.com/x/web-interface/region/feed/rcmd  display_id 是分页
// https://api.bilibili.com/x/web-interface/wbi/view/detail
// https://api.bilibili.com/x/web-interface/region/feed/rcmd?display_id=1&request_cnt=15&from_region=1003&device=web&plat=30&web_location=333.40138&w_rid=277aa6186a8bcbc80716480d4ade95cd&wts=1776152615

use crate::entity::{MusicConvertLayer, PlatformInterface};

struct BilibiliImpl;
impl PlatformInterface for BilibiliImpl {
    fn download(&self, params: &MusicConvertLayer) -> anyhow::Result<MusicConvertLayer> {
        Ok(MusicConvertLayer{
            music_id: "".to_string(),
            music_name: "".to_string(),
            music_author: "".to_string(),
            music_pic: "".to_string(),
            music_platform: "".to_string(),
            music_time: "".to_string(),
            music_source: "".to_string(),
            music_file: "".to_string(),
            func: params.func.clone(),
        })
    }
}

async fn request_web_api() -> anyhow::Result<Vec<MusicConvertLayer>> {
    Ok(vec![])
}


pub async fn call() -> anyhow::Result<Vec<MusicConvertLayer>> {
    Ok(vec![])
}