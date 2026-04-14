
use chrono::Utc;

use gpui::http_client::http::{header, HeaderValue};
use crate::com::call_js;
use crate::music_platform::kuwo_music::sign;

pub fn headers() -> header::HeaderMap {

    let mut headers = header::HeaderMap::new();
    let timestamp = Utc::now().timestamp().to_string();
    let hash = md5::compute(timestamp);
    let md5_string = format!("{:x}", hash);

    let cookie = format!("Hm_Iuvt_cdb524f42f23cer9b268564v7y735ewrq2324={}", md5_string);
    // let cookie = "Hm_Iuvt_cdb524f42f23cer9b268564v7y735ewrq2324=65CmNcbJBe5NdNbc5TDHKzZA2dsSDBrz";
    headers.insert("Accept", HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert("Accept-Language", HeaderValue::from_static("zh-CN,zh;q=0.9"));
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Pragma", HeaderValue::from_static("no-cache"));
    headers.insert("Referer", HeaderValue::from_static("https://www.kuwo.cn/"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
    // headers.insert("Secret", HeaderValue::from_static("1565c8cf5f440a619c4095df2ff4d34e3d9bb59f7c80c6463be408cac2c417f6017af6b8"));
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36"));
    headers.insert("sec-ch-ua", HeaderValue::from_static("\"Chromium\";v=\"146\", \"Not-A.Brand\";v=\"24\", \"Google Chrome\";v=\"146\""));
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
    headers.insert(header::COOKIE, cookie.parse().unwrap());
    let secret = sign::secret(cookie).unwrap();
    headers.insert("Secret", secret.parse().unwrap());
    headers
}


pub fn secret(cookie: String) -> Result<String, anyhow::Error> {
    call_js(
        "./src/music_platform/kugou_music/sign.js",
        "getSecret",
        vec![cookie],
    )
}

pub fn uuid() -> Result<String, anyhow::Error> {
    call_js(
        "./src/music_platform/kugou_music/sign.js",
        "getReqId",
        vec![],
    )
}
