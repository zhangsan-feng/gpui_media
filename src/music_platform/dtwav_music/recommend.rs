use std::sync::Arc;
use anyhow::Context;
use gpui::http_client::http::{header, HeaderValue};
use log::info;
use regex::Regex;
use scraper::{Html, Selector};
use crate::com::HttpClient;
use crate::entity::{MusicConvertLayer, MusicInfo, PlatformInterface};


fn headers() -> header::HeaderMap {

    let mut headers = header::HeaderMap::new();

    headers.insert("accept", HeaderValue::from_static("*/*"));
    headers.insert("accept-language", HeaderValue::from_static("zh-CN,zh;q=0.9"));
    headers.insert("cache-control", HeaderValue::from_static("no-cache"));
    headers.insert("pragma", HeaderValue::from_static("no-cache"));
    headers.insert("priority", HeaderValue::from_static("u=0, i"));
    headers.insert("referer", HeaderValue::from_static("https://dtwav.com/indexlist/hot.html"));
    headers.insert("sec-ch-ua", HeaderValue::from_static("\"Chromium\";v=\"146\", \"Not-A.Brand\";v=\"24\", \"Google Chrome\";v=\"146\""));
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));
    headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36"));

    headers
}



struct PlatformImpl;
impl PlatformInterface for PlatformImpl {

    fn download(&self, params: &MusicConvertLayer) -> anyhow::Result<MusicInfo> {

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
            let response = HttpClient::new()
                .get_for_html(&*params.music_source, headers())
                .await
                .context("Failed to fetch HTML")?;

            let html_content = response
                .text()
                .await
                .context("Failed to read HTML")?;
                
            let title_re = Regex::new(r"title:\s*'([^']*)'").unwrap();
            let author_re = Regex::new(r"author:\s*'([^']*)'").unwrap();
            let url_re = Regex::new(r"url:\s*'([^']*)'").unwrap();
            let pic_re = Regex::new(r"pic:\s*'([^']*)'").unwrap();

            let title = title_re.captures(&html_content) // &html_content 不需要 *
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .ok_or_else(|| anyhow::anyhow!("未找到 title"))?;

            let author = author_re.captures(&html_content)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let url = url_re.captures(&html_content)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .ok_or_else(|| anyhow::anyhow!("未找到 url"))?;

            let pic = pic_re.captures(&html_content)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .ok_or_else(|| anyhow::anyhow!("未找到 pic"))?;

            info!("解析成功: {} - {} (URL: {} pic:{})", title, author, url, pic);
            let file_name = format!("./music/ {}", url.split("/").last().unwrap());
            HttpClient::new().download_music(file_name.clone(), url, headers()).await.expect("download file error");


            Ok(MusicInfo{
                music_id: "".to_string(),
                music_platform: "dtwav".to_string(),
                music_name: title,
                music_author: "".to_string(),
                music_pic: format!("https://dtwav.com{}", pic),
                music_source: file_name,
            })
        })
        })
    }
}


pub async fn call(page: &str) -> Result<Vec<MusicConvertLayer>, anyhow::Error> {
    let mut call_back = Vec::new();

    let response = HttpClient::new()
        .get_for_html("https://dtwav.com/indexlist/hot.html?typeclass=hot&page=1", headers())
        .await
        .context("Failed to fetch HTML")?;

    let body = response.text().await?;
    let document = Html::parse_document(&body);


    let selector = Selector::parse(".media.thread.tap").expect("Invalid selector");
    let a_selector = Selector::parse("a").expect("Invalid a selector");
    // let img_selector = Selector::parse("img").expect("Invalid a selector");

    for element in document.select(&selector) {

        let link_element = element
            .select(&a_selector)
            .next()
            .ok_or_else(|| anyhow::anyhow!("No <a> tag found in .media-body"))?;


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

        // println!("{} {}", href, text_content);


        let author = text_content.split("[").next().unwrap_or("");


        call_back.push(MusicConvertLayer {
            music_id: "".to_string(),
            music_name: author.to_string(),
            music_source: href.to_string(),
            music_pic: "".to_string(),
            music_platform: "dtwav".to_string(),
            platform: Arc::new(PlatformImpl),
        });
    }

    Ok(call_back)
}