
use gpui::http_client::http::{header, HeaderValue};
use crate::com::HttpClient;
use crate::entity::{MusicConvertLayer, PlatformInterface};
use crate::music_platform::xmwav_music::headers;
//
// pub struct XmWavImpl;
// impl PlatformInterface for XmWavImpl {
//     fn download(&self, params: &MusicConvertLayer) -> anyhow::Result<MusicInfo>  {
//
//         todo!()
//     }
// }


pub async fn call() -> anyhow::Result<Vec<MusicConvertLayer>> {
    match HttpClient::new().get_for_html("https://xmwav.net/xmlist/dy.html?classname=dy&page=1", headers()).await {
        Ok(response) => {

        }
        Err(e) => {}
    }

    Ok(Vec::new())
}