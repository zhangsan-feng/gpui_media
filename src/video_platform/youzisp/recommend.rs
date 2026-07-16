use super::headers;
use crate::com::request::HttpClient;
use crate::drive::{NetworkStatic, NetworkStaticInterface};
use futures_util::future::join_all;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

const BASE_URL: &str = "https://youzisp.tv";
const PATHS: [(&str, &str); 4] = [
    ("/vodshow/dianying-----------.html", "电影"),
    ("/vodshow/dianshiju-----------.html", "电视剧"),
    ("/vodshow/zongyi-----------.html", "今日推荐"),
    ("/vodshow/dongman-----------.html", "动漫"),
];

pub struct YouziInterface;

impl NetworkStaticInterface for YouziInterface {
    fn download(&self, _params: &NetworkStatic) {}

    fn play(&self, params: &NetworkStatic) -> String {
        if params.source.contains(".m3u8") {
            return params.source.clone();
        }

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let response = HttpClient::new()
                    .get_for_html(&abs_url(&params.source), headers())
                    .await
                    .expect("request youzisp play page error");
                let html = response
                    .text()
                    .await
                    .expect("youzisp play page html parse error")
                    .replace("\\/", "/");
                Regex::new(r#"https?://[^\s"'<>]+\.m3u8[^\s"'<>]*"#)
                    .unwrap()
                    .captures(&html)
                    .and_then(|c| c.get(0))
                    .map(|m| m.as_str().to_string())
                    .expect("youzisp play url not found")
            })
        })
    }

    fn detail(&self, params: &NetworkStatic) -> Vec<NetworkStatic> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let response = HttpClient::new()
                    .get_for_html(&abs_url(&params.source), headers())
                    .await
                    .expect("request youzisp detail page error");
                let html = response
                    .text()
                    .await
                    .expect("youzisp detail html parse error");
                parse_detail(&html, params)
            })
        })
    }
}

fn abs_url(path: &str) -> String {
    if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{}/{}", BASE_URL, path.trim_start_matches('/'))
    }
}

fn selector(value: &str) -> Selector {
    Selector::parse(value).expect("invalid youzisp selector")
}

fn attr<'a>(element: ElementRef<'a>, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| element.value().attr(name))
}

fn parse_videos(html: &str, seen: &mut HashSet<String>, category: &str) -> Vec<NetworkStatic> {
    let document = Html::parse_document(html);
    let item_selector = selector("a.module-item[href]");
    let img_selector = selector(".module-item-pic img");
    let title_selector = selector(".module-poster-item-title");
    let mut videos = Vec::new();

    for item in document.select(&item_selector) {
        let Some(href) = item.value().attr("href") else {
            continue;
        };
        if !href.contains("/voddetail/") && !href.contains("/vodplay/") {
            continue;
        }

        let source = abs_url(href);
        if !seen.insert(source.clone()) {
            continue;
        }

        let name = item
            .value()
            .attr("title")
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .or_else(|| {
                item.select(&title_selector)
                    .next()
                    .map(|title| title.text().collect::<String>().trim().to_string())
                    .filter(|name| !name.is_empty())
            })
            .unwrap_or_else(|| "未命名视频".to_string());

        let img = item
            .select(&img_selector)
            .next()
            .and_then(|img| attr(img, &["data-original", "data-src", "src"]))
            .map(abs_url)
            .unwrap_or_default();

        videos.push(NetworkStatic {
            id: Uuid::new_v4().to_string(),
            name,
            img,
            author: "youzisp".to_string(),
            category: category.to_string(),
            headers: Default::default(),
            source,
            func: Arc::new(YouziInterface),
        });
    }

    videos
}

fn parse_detail(html: &str, params: &NetworkStatic) -> Vec<NetworkStatic> {
    let document = Html::parse_document(html);
    let selector = selector(".module-play-list-content > a[href]");
    let mut seen = HashSet::new();

    document
        .select(&selector)
        .filter_map(|element| element.value().attr("href"))
        .map(abs_url)
        .filter(|source| seen.insert(source.clone()))
        .map(|source| NetworkStatic {
            id: Uuid::new_v4().to_string(),
            name: params.name.clone(),
            img: params.img.clone(),
            author: params.author.clone(),
            category: params.category.clone(),
            headers: params.headers.clone(),
            source,
            func: params.func.clone(),
        })
        .collect()
}

pub async fn recommend() -> Vec<NetworkStatic> {
    let mut call_back = Vec::new();
    let mut seen = HashSet::new();

    let responses = join_all(PATHS.map(|(path, category)| async move {
        let url = abs_url(path);
        let result = match HttpClient::new().get_for_html(&url, headers()).await {
            Ok(response) => response.text().await.map_err(anyhow::Error::from),
            Err(err) => Err(err),
        };
        (category, url, result)
    }))
    .await;

    for (category, url, result) in responses {
        match result {
            Ok(html) => call_back.extend(parse_videos(&html, &mut seen, category)),
            Err(err) => log::info!("request {} error: {}", url, err),
        }
    }

    call_back
}
