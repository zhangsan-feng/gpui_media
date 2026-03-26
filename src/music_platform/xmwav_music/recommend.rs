
use gpui::http_client::http::{header, HeaderValue};
use crate::com::HttpClient;
use crate::entity;
use crate::entity::{MusicConvertLayer, MusicInfo, PlatformInterface};

fn headers() -> header::HeaderMap {

    let mut headers = header::HeaderMap::new();

    headers.insert("accept", HeaderValue::from_static("*/*"));
    headers.insert("accept-language", HeaderValue::from_static("zh-CN,zh;q=0.9"));
    headers.insert("cache-control", HeaderValue::from_static("no-cache"));
    headers.insert("pragma", HeaderValue::from_static("no-cache"));
    headers.insert("priority", HeaderValue::from_static("u=0, i"));
    headers.insert("sec-ch-ua", HeaderValue::from_static("\"Chromium\";v=\"146\", \"Not-A.Brand\";v=\"24\", \"Google Chrome\";v=\"146\""));
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));
    headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36"));


    headers
}



struct PlatformImpl;
impl PlatformInterface for PlatformImpl {
    fn download(&self, params: &MusicConvertLayer) -> anyhow::Result<MusicInfo>  {

        todo!()
    }
}


pub async fn call() -> Vec<MusicConvertLayer>{
    match HttpClient::new().get("https://xmwav.net/xmlist/dy.html?classname=dy&page=1", headers()).await {
        Ok(response) => {

        }
        Err(e) => {}
    }

    Vec::new()
}