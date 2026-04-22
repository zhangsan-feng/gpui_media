

// https://api.bilibili.com/x/web-interface/region/feed/rcmd  display_id 是分页
// https://api.bilibili.com/x/web-interface/wbi/view/detail
// https://api.bilibili.com/x/player/wbi/playurl
// https://api.bilibili.com/x/web-interface/region/feed/rcmd?display_id=1&request_cnt=15&from_region=1003&device=web&plat=30&web_location=333.40138&w_rid=277aa6186a8bcbc80716480d4ade95cd&wts=1776152615

use std::fmt::format;
use std::sync::Arc;
use log::info;
use url::Url;
use crate::com::{call_js, HttpClient};
use crate::entity::{MusicConvertLayer, PlatformInterface};
use crate::music_platform::bilibili::entity::{Detail, Recommend};
use crate::music_platform::bilibili::sign::{headers, SignStruct};

struct BilibiliImpl;
impl PlatformInterface for BilibiliImpl {
    fn download(&self, params: &MusicConvertLayer) -> anyhow::Result<MusicConvertLayer> {

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {

                let res :Option<Detail>= match HttpClient::new().get(&*params.music_source, headers()).await {
                    Ok(val) => {
                        // info!("{}", val);
                        serde_json::from_value(val).unwrap_or_else(|e| {
                            info!("序列化失败: {}", e);
                            None
                        })

                    },
                    Err(err) => {
                        info!("序列化失败: {}", err);
                        None
                    }
                };
                // info!("{:?}", res.unwrap().data.dash.audio.get(0).unwrap().base_url);
                if !res.is_some(){
                    return Err(anyhow::anyhow!("res is None"));
                }
                let file_link = res.unwrap().data.dash.audio.get(0).unwrap().base_url.to_string();
                let download_file = format!("./music/{}_{}.mp3", params.music_name, params.music_id);
                HttpClient::new()
                    .download_music(download_file.clone(), file_link, headers())
                    .await
                    .expect("download file error");

                Ok(MusicConvertLayer{
                    music_id: params.music_id.clone(),
                    music_name: params.music_name.clone(),
                    music_author: params.music_author.clone(),
                    music_pic: params.music_pic.clone(),
                    music_platform: params.music_platform.clone(),
                    music_time: params.music_time.clone(),
                    music_source: params.music_source.clone(),
                    music_file: download_file,
                    func: params.func.clone(),
                })
            })
        })
    }
}

async fn request_web_api(url:&str) -> anyhow::Result<Vec<MusicConvertLayer>> {

    let mut call_back = vec![];
    
    let res:Option<Recommend> = match HttpClient::new().get(url, headers()).await {
        Ok(val) => {
            // info!("{}", val);
            serde_json::from_value(val)?
        },
        Err(err) => {
            info!("序列化失败: {}", err);
            None
        }
    };
    if res.is_some() {
        for data in res.unwrap().data.archives{
            // https://api.bilibili.com/x/player/wbi/playurl
            // params = {
            //     "qn":"32",
            //     "fnver":"0",
            //     "fnval":"4048",
            //     "fourk":"1",
            //     "voice_balance":"1",
            //     "gaia_source":"pre-load",
            //     "isGaiaAvoided":"true",
            //     "avid":aid,
            //     "bvid":bvid,
            //     "cid":cid,
            //     "web_location":"1315873",
            //     # "dm_img_list":"",
            //     # "dm_img_str":"",
            //     # "dm_cover_img_str":"",
            //     # "dm_img_inter":"",
            //     "w_rid":"",
            //     "wts":""
            // }

            let mut url = Url::parse("https://api.bilibili.com/x/player/wbi/playurl")?;
            {
                let mut params = url.query_pairs_mut();
                params.append_pair("avid", data.aid.to_string().as_str());
                params.append_pair("bvid", data.bvid.to_string().as_str());
                params.append_pair("cid", data.cid.to_string().as_str());
                params.append_pair("qn", "80");
                params.append_pair("fnver", "0");
                params.append_pair("fnval", "4048");
                params.append_pair("fourk", "1");
                params.append_pair("gaia_source", "");
                params.append_pair("from_client", "BROWSER");
                params.append_pair("is_main_page", "true");
                params.append_pair("need_fragment", "false");
                params.append_pair("isGaiaAvoided", "false");
                params.append_pair("client_attr", "0");
                params.append_pair("version_name", "4.9.76");
                params.append_pair("app_id", "100");
                params.append_pair("voice_balance", "1");
                params.append_pair("web_location", "1315873");

                let sign_params:SignStruct = match call_js(include_str!("./sign.js"), "gen_w_rid", vec![]){
                    Ok(v) => {
                        // info!("{:?}", v);
                        serde_json::from_value(v)?
                    },
                    Err(e) => anyhow::bail!(e),
                };
                // info!("{:?}", sign_params);
                params.append_pair("w_rid", sign_params.w_rid.as_str());
                params.append_pair("wts", sign_params.wts.as_str());

            }
            let music_name = data.title.to_string().chars().take(24).filter(|&c| c.is_ascii_alphabetic() || c >= '\u{4e00}' && c <= '\u{9fff}').collect();
            // info!("{}", url.to_string());
            call_back.push(
                MusicConvertLayer{
                    music_id: data.bvid.to_string(),
                    music_name:music_name,
                    music_author: data.author.name,
                    music_pic:data.cover.to_string(),
                    music_platform: "bilibili".to_string(),
                    music_time: "".to_string(),
                    music_source: url.to_string(),
                    music_file: "".to_string(),
                    func: Arc::new((BilibiliImpl)),
                }
            )
        }
    }


    Ok(call_back)
}


pub async fn call() -> anyhow::Result<Vec<MusicConvertLayer>> {
    let mut call_back = vec![];
    for i in 1..5{
        
        let mut url = Url::parse("https://api.bilibili.com/x/web-interface/region/feed/rcmd")?;
        {
            let mut params = url.query_pairs_mut();
            params.append_pair("display_id", i.to_string().as_str());
            params.append_pair("request_cnt", "20");
            params.append_pair("from_region", "1003");
            params.append_pair("device", "web");
            params.append_pair("plat", "30");
            params.append_pair("web_location", "333.40138");
            params.append_pair("plat", "30");

            let sign_params:SignStruct = match call_js(include_str!("./sign.js"), "gen_w_rid", vec![]){
                Ok(v) => {
                    // info!("{:?}", v);
                    serde_json::from_value(v)?
                },
                Err(e) => anyhow::bail!(e),
            };
            // info!("{:?}", sign_params);
            params.append_pair("w_rid", sign_params.w_rid.as_str());
            params.append_pair("wts", sign_params.wts.as_str());

        }
        // info!("{}", url.as_ref());

        match request_web_api(url.as_ref()).await{
            Ok(val) => call_back.extend(val),
            Err(e) => info!("error: {}", e),
        }
    }


    Ok(call_back)
}