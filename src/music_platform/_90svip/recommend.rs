
use std::sync::Arc;
use anyhow::{Context};
use std::result::Result::Ok;
use log::info;
use regex::Regex;
use crate::{com::HttpClient, entity::{MusicConvertLayer, PlatformInterface}, music_platform::_90svip::{entity::RecpmmondMusicEntity, sign::headers}};



struct V90VipImpl;
impl PlatformInterface for V90VipImpl {
    fn download(&self, params: &MusicConvertLayer) -> anyhow::Result<MusicConvertLayer> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {

                let download_file = format!(
                    "./music/{}_{}.mp3",
                    params.music_name,
                   params.music_id
                );
                HttpClient::new()
                    .download_music(download_file.clone(), params.music_source.clone(), headers())
                    .await
                    .expect("download file error");

                Ok(MusicConvertLayer {
                    music_id: params.music_id.clone(),
                    music_platform: params.music_platform.clone(),
                    music_name: params.music_name.clone(),
                    music_author: params.music_author.clone(),
                    music_pic: params.music_pic.clone(),
                    music_source: params.music_source.to_string(),
                    music_file: download_file,
                    func: params.func.clone(),
                    music_time: params.music_time.clone(),
                })
            })
        })
    }
}


async fn request_web_api(url:&str) -> anyhow::Result<Vec<MusicConvertLayer>> {
    let mut call_back = Vec::new();
    let response = HttpClient::new()
        .get_for_html(url, headers())
        .await
        .context("Failed to fetch HTML")?;

    let body = response.text().await?;
    let re = Regex::new(r"\[\{(.*?)\}\]")?;
    let data = re
        .captures(&body)
        .and_then(|c| c.get(0))
        .map(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("未找到 url"))?;
    let val : Vec<RecpmmondMusicEntity> = serde_json::from_str(data)?;
    // println!("{:?}", val);


    for data in val{
        if data.song_sheet_name == "小白音乐"{
            continue
        }

        for (index, _) in data.song_id.iter().enumerate() {

            let music_id = match data.song_ids.get(index) {
                Some(music_id) => music_id,
                None => continue,
            };
            let music_name = match data.song_names.get(index) {
                Some(music_name) => music_name,
                None => continue,
            };
            let music_author = match data.artist_names.get(index) {
                Some(music_author) => music_author,
                None => continue,
            };

            let sign = match data.sign.get(index) {
                Some(sign) => sign,
                None => continue,
            };
            let music_platform = match data.song_types.get(index) {
                Some(music_platform) => music_platform,
                None => continue,
            };
            let music_pic_id = match data.album_covers.get(index) {
                Some(music_pic_id) => music_pic_id,
                None => continue,
            };
            let music_source = format!(
                "https://myhkw.cn/api/url?song={}&type={}&id=99999&sign={}",
                music_id, music_platform, sign
            );
            let music_pic = format!(
                "https://myhkw.cn/api/pic?song={}&pic={}&type={}&id=99999&sign={}",
                music_id, music_pic_id, music_platform, sign
            );
            println!("{} {} {}", music_source, music_pic, music_name);


            call_back.push(MusicConvertLayer{
                music_id: music_id.to_string(),
                music_name: music_name.to_string(),
                music_author: music_author.to_string(),
                music_pic: music_pic,
                music_platform: music_platform.to_string(),
                music_time: "".to_string(),
                music_source: music_source,
                music_file: "".to_string(),
                func:Arc::new(V90VipImpl),
            });
        }
    }

    Ok(call_back)
}

// https://myhkw.cn/api/lyrics?song=2603500959&type=wy&id=99999&sign=2c/5z5DNZV/M2&ksc=2603500959&_=1776151073362
// format!("https://myhkw.cn/api/lyrics?song={}&type={}&id=99999&sign={}ksc=2603500959&_={}", )
pub async fn call() -> anyhow::Result<Vec<MusicConvertLayer>>{

    let mut call_back = Vec::new();
    let url_list = vec![
        "https://myhkw.cn/cache/playlist/99999.js"
    ];
    for url in url_list{
        match request_web_api(url).await {
            Ok(val) => call_back.extend(val),
            Err(e) => {
                info!("err:{}", e)
            }
        }
    }
    Ok(call_back)
}
