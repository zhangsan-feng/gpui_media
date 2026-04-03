use anyhow::Context;
use std::sync::Arc;

use crate::com::HttpClient;
use crate::entity::{MusicConvertLayer, PlatformInterface};
use crate::music_platform::xmwav_music::headers;
use log::info;
use regex::Regex;
use scraper::{Html, Selector};
use uuid::Uuid;

pub struct XmWavImpl;
impl PlatformInterface for XmWavImpl {
    fn download(&self, params: &MusicConvertLayer) -> anyhow::Result<MusicConvertLayer> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let response = HttpClient::new()
                    .get_for_html(&*params.music_source, headers())
                    .await
                    .context("Failed to fetch HTML")?;

                let html_content = response.text().await.context("Failed to read HTML")?;

                let url_re = Regex::new(r#"mp3:\s*"([^"]*)""#).unwrap();

                let url = url_re
                    .captures(&html_content)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .ok_or_else(|| anyhow::anyhow!("xmwav 未找到 artist"))?;

                // info!("url: {} ", url);
                let download_file = format!(
                    "./music/{}_{}",
                    params.music_name,
                    url.split("/").last().unwrap()
                );
                HttpClient::new()
                    .download_music(download_file.clone(), url, headers())
                    .await
                    .expect("download file error");

                Ok(MusicConvertLayer {
                    music_id: params.music_id.clone(),
                    music_platform: "dtwav".to_string(),
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

pub async fn request_web(url: &str) -> anyhow::Result<Vec<MusicConvertLayer>> {
    let mut call_back = Vec::new();

    let response = HttpClient::new()
        .get_for_html(url, headers())
        .await
        .context("Failed to fetch HTML")?;

    let body = response.text().await?;
    // info!("{}", body);
    let document = Html::parse_document(&body);

    let selector = Selector::parse(".list.bgb ul > a ").expect("Invalid selector");
    let h2_selector = Selector::parse("h2").expect("Invalid h2 selector");

    for element in document.select(&selector) {
        let href = element
            .value()
            .attr("href")
            .ok_or_else(|| anyhow::anyhow!("a tag not find href"))?;

        let h2_element = element
            .select(&h2_selector)
            .next()
            .ok_or_else(|| anyhow::anyhow!("a tag not find h2"))?;

        let text_content: String = h2_element
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        info!(" {} {}", href, text_content);

        let music_name = text_content.split("[").next().unwrap_or("");

        call_back.push(MusicConvertLayer {
            music_id: Uuid::new_v4().to_string(),
            music_name: music_name.to_string(),
            music_source: href.to_string(),
            music_pic: "".to_string(),
            music_platform: "xmwav".to_string(),
            func: Arc::new(XmWavImpl),
            music_author: "".to_string(),
            music_file: "".to_string(),
            music_time: "".to_string(),
        });
    }

    Ok(call_back)
}

pub async fn call() -> anyhow::Result<Vec<MusicConvertLayer>> {
    let mut call_back = Vec::new();

    let url_list = vec![
        "https://xmwav.net/xmlist/dy.html?classname=dy&page=1",
        "https://xmwav.net/xmlist/rd.html?classname=rd&page=1",
        "https://xmwav.net/xmlist/hk.html?classname=hk&page=1",
        "https://xmwav.net/xmlist/rh.html?classname=rh&page=1",
        "https://xmwav.net/xmlist/om.html?classname=om&page=1",
        "https://xmwav.net/xmlist/dj.html?classname=dj&page=1",
    ];

    for url in url_list {
        match request_web(url).await {
            Ok(val) => call_back.extend(val),
            Err(e) => {
                info!("err:{}", e)
            }
        }
    }

    Ok(call_back)
}
