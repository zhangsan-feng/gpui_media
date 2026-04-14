use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;
use rand::{random, Rng};
use crate::entity::{MusicConvertLayer, PlatformInterface};


struct UzzImpl;
impl PlatformInterface for UzzImpl{
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

async fn request_web_api(url:&str)-> anyhow::Result<Vec<MusicConvertLayer>>{
    let mut call_back = Vec::new();


    Ok(call_back)
}

pub async fn call() -> anyhow::Result<Vec<MusicConvertLayer>>{

    let mut call_back = Vec::new();
    let rand_int: i64 = rand::thread_rng().gen_range(0..=i64::MAX);
    let ts_millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
    println!("rand_int : {}   ts_secs:{}", rand_int, ts_millis);
    // jQuery + int64 _ + timestamp & _= timestamp

    let base_url = format!("https://m.uzz.me/api.php?callback=jQuery{}_",rand_int );
    let url_list = vec![
        format!("{}{}&types=playlist&id=3778678&_={}", base_url, ts_millis, ts_millis),
        format!("{}{}&types=playlist&id=3779629&_={}", base_url, ts_millis, ts_millis),
        format!("{}{}&types=playlist&id=2884035&_={}", base_url, ts_millis, ts_millis),
        format!("{}{}&types=playlist&id=1978921795&_={}", base_url, ts_millis, ts_millis),
        format!("{}{}&types=playlist&id=991319590&_={}", base_url, ts_millis, ts_millis),
        format!("{}{}&types=playlist&id=19723756&_={}", base_url, ts_millis, ts_millis),
        format!("{}{}&types=playlist&id=5338990334&_={}", base_url, ts_millis, ts_millis),
        format!("{}{}&types=playlist&id=13372522766&_={}", base_url, ts_millis, ts_millis),
        format!("{}{}&types=playlist&id=2312165875&_={}", base_url, ts_millis, ts_millis),
        format!("{}{}&types=playlist&id=8466236201&_={}", base_url, ts_millis, ts_millis),
        ];
    for url in url_list{
        match request_web_api(&*url).await {
            Ok(val) => call_back.extend(val),
            Err(e) => {
                info!("err:{}", e)
            }
        }
    }
    Ok(call_back)
}
