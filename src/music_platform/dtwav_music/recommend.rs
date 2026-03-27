use crate::com::HttpClient;
use crate::entity::{MusicConvertLayer,  PlatformInterface};
use anyhow::Context;
use gpui::http_client::http::{HeaderValue, header};
use log::info;
use regex::Regex;
use scraper::{Html, Selector};
use std::sync::Arc;
use uuid::{uuid, Uuid};
use crate::music_platform::dtwav_music::headers;

pub struct DtWavImpl;
impl PlatformInterface for DtWavImpl {
    fn download(&self, params: &MusicConvertLayer) -> anyhow::Result<MusicConvertLayer> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let response = HttpClient::new()
                    .get_for_html(&*params.music_source, headers())
                    .await
                    .context("Failed to fetch HTML")?;

                let html_content = response.text().await.context("Failed to read HTML")?;
                // println!("{}", html_content);

                let title_re = Regex::new(r"title:\s*'([^']*)'").unwrap();
                let author_re = Regex::new(r"author:\s*'([^']*)'").unwrap();
                let url_re = Regex::new(r"url:\s*'([^']*)'").unwrap();
                // let pic_re = Regex::new(r"pic:\s*'([^']*)'").unwrap();

                let title = title_re
                    .captures(&html_content) // &html_content 不需要 *
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(||"".to_string());

                let author = author_re
                    .captures(&html_content)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(||"".to_string());

                let url = url_re
                    .captures(&html_content)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .ok_or_else(|| anyhow::anyhow!("未找到 url"))?;

                // let pic = pic_re
                //     .captures(&html_content)
                //     .and_then(|c| c.get(1))
                //     .map(|m| m.as_str().to_string())
                //     .unwrap_or_else(||"".to_string());

                println!("解析成功: {} - {} (URL: {} pic:)", title, author, url);
                let download_file = format!("./music/{}_{}",title, url.split("/").last().unwrap());
                HttpClient::new()
                    .download_music(download_file.clone(), url, headers())
                    .await
                    .expect("download file error");

                Ok(MusicConvertLayer {
                    music_id: params.music_id.clone(),
                    music_platform: "dtwav".to_string(),
                    music_name: title,
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



pub async fn request_web(url: &str)-> anyhow::Result<Vec<MusicConvertLayer>>{
    let mut call_back = Vec::new();

    let response = HttpClient::new()
        .get_for_html(url, headers())
        .await
        .context("Failed to fetch HTML")?;

    let body = response.text().await?;
    let document = Html::parse_document(&body);

    let selector = Selector::parse(".media.thread.tap").expect("Invalid selector");
    let a_selector = Selector::parse("a").expect("Invalid a selector");
    let img_selector = Selector::parse("img").expect("Invalid a selector");

    for element in document.select(&selector) {
        let link_element = element
            .select(&a_selector)
            .next()
            .ok_or_else(|| anyhow::anyhow!("No <a> tag found in .media-body"))?;

        let img_element = element
            .select(&img_selector)
            .next()
            .ok_or_else(|| anyhow::anyhow!("No <a> tag found in .media-body"))?;
        let img = img_element.value().attr("src").ok_or_else(|| anyhow::anyhow!("No src found in .media-body"))?;
        let img = format!("https://dtwav.com{}",img);

        let href = link_element
            .value()
            .attr("href")
            .ok_or_else(|| anyhow::anyhow!("No href attribute found"))?;

        let text_content: String = link_element
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        info!("{} {} {}", href, text_content, img);

        let author = text_content.split("[").next().unwrap_or("");

        call_back.push(MusicConvertLayer {
            music_id: Uuid::new_v4().to_string(),
            music_name: author.to_string(),
            music_source: href.to_string(),
            music_pic: img,
            music_platform: "dtwav".to_string(),
            func: Arc::new(DtWavImpl),
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
        "https://dtwav.com/index.html?page=1",
        "https://dtwav.com/indexlist/hot.html?typeclass=hot&page=1"
    ];
    for url in url_list {
        match request_web(url).await {
            Ok(val)=>{
                call_back.extend(val)
            }
            Err(e)=>{
                info!("err:{}",e)
            }
        }
    }

    Ok(call_back)
}
