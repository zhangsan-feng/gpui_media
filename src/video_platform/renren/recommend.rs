use super::headers;
use crate::com::request::HttpClient;
use crate::drive::{NetworkStatic, NetworkStaticInterface};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

const BASE_URL: &str = "https://www.renren.pro";

pub struct RenrenInterface;

impl NetworkStaticInterface for RenrenInterface {
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
                    .expect("request renren play page error");
                let html = response
                    .text()
                    .await
                    .expect("renren play page html parse error")
                    .replace("\\/", "/");
                Regex::new(r#"url:\s*["']([^"']+\.m3u8[^"']*)["']"#)
                    .unwrap()
                    .captures(&html)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .expect("renren play url not found")
            })
        })
    }

    fn detail(&self, params: &NetworkStatic) -> Vec<NetworkStatic> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let response = HttpClient::new()
                    .get_for_html(&abs_url(&params.source), headers())
                    .await
                    .expect("request renren detail page error");
                let html = response
                    .text()
                    .await
                    .expect("renren detail html parse error");
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
    Selector::parse(value).expect("invalid renren selector")
}

fn attr<'a>(element: ElementRef<'a>, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| element.value().attr(name))
}

fn text(element: ElementRef<'_>) -> String {
    element.text().collect::<String>().trim().to_string()
}

fn parse_videos(html: &str) -> Vec<NetworkStatic> {
    let document = Html::parse_document(html);
    let item_selector = selector(".module-item");
    let link_selector = selector(r#"a[href^="/play/"]"#);
    let img_selector = selector(".module-item-pic img");
    let title_selector = selector(".module-item-title, .video-name a");
    let mut seen = HashSet::new();
    let mut videos = Vec::new();

    for item in document.select(&item_selector) {
        let Some(link) = item.select(&link_selector).next() else {
            continue;
        };
        let Some(href) = link.value().attr("href") else {
            continue;
        };

        let source = abs_url(href);
        if !seen.insert(source.clone()) {
            continue;
        }

        let name = attr(link, &["title"])
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .or_else(|| {
                item.select(&title_selector)
                    .next()
                    .map(text)
                    .filter(|name| !name.is_empty())
            })
            .unwrap_or_else(|| "未命名视频".to_string());

        let img = item
            .select(&img_selector)
            .next()
            .and_then(|img| attr(img, &["data-src", "src"]))
            .map(abs_url)
            .unwrap_or_default();

        videos.push(NetworkStatic {
            id: Uuid::new_v4().to_string(),
            name,
            img,
            author: "renren".to_string(),
            category: String::new(),
            headers: Default::default(),
            source,
            func: Arc::new(RenrenInterface),
        });
    }

    videos
}

fn parse_detail(html: &str, params: &NetworkStatic) -> Vec<NetworkStatic> {
    let document = Html::parse_document(html);
    let selector = selector(".module-blocklist a[href]");
    let mut seen = HashSet::new();

    let videos: Vec<_> = document
        .select(&selector)
        .filter_map(|element| element.value().attr("href"))
        .filter(|href| href.contains("/play/"))
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
        .collect();

    if videos.is_empty() {
        vec![params.clone()]
    } else {
        videos
    }
}

pub async fn recommend() -> Vec<NetworkStatic> {
    let response = match HttpClient::new().get_for_html(BASE_URL, headers()).await {
        Ok(response) => response,
        Err(err) => {
            log::info!("request {} error: {}", BASE_URL, err);
            return Vec::new();
        }
    };

    match response.text().await {
        Ok(html) => parse_videos(&html),
        Err(err) => {
            log::info!("read {} html error: {}", BASE_URL, err);
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_detail_falls_back_to_current_item_when_playlist_missing() {
        let current = NetworkStatic {
            id: "1".to_string(),
            name: "test".to_string(),
            img: String::new(),
            author: "renren".to_string(),
            category: String::new(),
            headers: Default::default(),
            source: "https://www.renren.pro/play/abc".to_string(),
            func: Arc::new(RenrenInterface),
        };

        let videos = parse_detail("<html></html>", &current);

        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].source, current.source);
    }
}
