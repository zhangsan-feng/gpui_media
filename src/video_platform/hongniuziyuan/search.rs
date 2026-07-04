use crate::com::HttpClient;
use crate::drive::{NetworkStatic, NetworkStaticInterface};
use regex::Regex;
use reqwest::{Url, header};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

const BASE_URL: &str = "https://hongniuziyuan.net";
const AUTHOR: &str = "hongniuziyuan";

pub struct HongniuziyuanInterface;

impl NetworkStaticInterface for HongniuziyuanInterface {
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
                    .expect("request hongniuziyuan play page error");
                let html = response
                    .text()
                    .await
                    .expect("hongniuziyuan play page html parse error")
                    .replace("\\/", "/");

                Regex::new(r#"https?://[^\s"'<>]+\.m3u8[^\s"'<>]*"#)
                    .unwrap()
                    .captures(&html)
                    .and_then(|capture| capture.get(0))
                    .map(|matched| matched.as_str().to_string())
                    .expect("hongniuziyuan play url not found")
            })
        })
    }

    fn detail(&self, params: &NetworkStatic) -> Vec<NetworkStatic> {
        if params.source.contains("/vod/play") || params.source.contains("vodplay") {
            return vec![params.clone()];
        }

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let response = HttpClient::new()
                    .get_for_html(&abs_url(&params.source), headers())
                    .await
                    .expect("request hongniuziyuan detail page error");
                let html = response
                    .text()
                    .await
                    .expect("hongniuziyuan detail html parse error");
                let videos = parse_detail_html(&html, params);

                if videos.is_empty() {
                    vec![params.clone()]
                } else {
                    videos
                }
            })
        })
    }
}

pub async fn search(keyword: String) -> Vec<NetworkStatic> {
    let url = search_url(&keyword);
    let response = match HttpClient::new().get_for_html(&url, headers()).await {
        Ok(response) => response,
        Err(err) => {
            log::info!("request hongniuziyuan search error: {}", err);
            return Vec::new();
        }
    };

    match response.text().await {
        Ok(html) => parse_search_html(&html),
        Err(err) => {
            log::info!("read hongniuziyuan search html error: {}", err);
            Vec::new()
        }
    }
}

fn headers() -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    headers.insert("accept-language", "zh-CN,zh;q=0.9".parse().unwrap());
    headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36".parse().unwrap());
    headers
}

fn search_url(keyword: &str) -> String {
    let mut url = Url::parse(&format!("{}/index.php/vod/search.html", BASE_URL))
        .expect("invalid hongniuziyuan search url");
    url.query_pairs_mut().append_pair("wd", keyword);
    url.to_string()
}

fn parse_search_html(html: &str) -> Vec<NetworkStatic> {
    let link_re = Regex::new(
        r#"(?is)<a\b(?P<attrs>[^>]*\bhref\s*=\s*["'](?P<href>[^"']*(?:/vod/detail|/vod/play|voddetail|vodplay)[^"']*)["'][^>]*)>(?P<body>.*?)</a>"#,
    )
    .unwrap();
    let img_re = Regex::new(r#"(?is)<img\b(?P<attrs>[^>]*)>"#).unwrap();
    let mut seen = HashSet::new();
    let mut videos = Vec::new();

    for capture in link_re.captures_iter(html) {
        let href = capture
            .name("href")
            .map(|matched| matched.as_str())
            .unwrap_or_default();
        let source = abs_url(href);
        if !seen.insert(source.clone()) {
            continue;
        }

        let attrs = capture
            .name("attrs")
            .map(|matched| matched.as_str())
            .unwrap_or_default();
        let body = capture
            .name("body")
            .map(|matched| matched.as_str())
            .unwrap_or_default();
        let name = attr(attrs, "title")
            .or_else(|| attr(attrs, "alt"))
            .or_else(|| {
                let stripped = strip_tags(body);
                (!stripped.is_empty()).then_some(stripped)
            })
            .unwrap_or_else(|| "unknown video".to_string());

        let img = img_re
            .captures(body)
            .and_then(|img| img.name("attrs"))
            .and_then(|attrs| {
                attr(attrs.as_str(), "data-original")
                    .or_else(|| attr(attrs.as_str(), "data-src"))
                    .or_else(|| attr(attrs.as_str(), "src"))
            })
            .map(|path| abs_url(&path))
            .unwrap_or_default();

        videos.push(NetworkStatic {
            id: Uuid::new_v4().to_string(),
            name,
            img,
            author: AUTHOR.to_string(),
            category: AUTHOR.to_string(),
            headers: Default::default(),
            source,
            func: Arc::new(HongniuziyuanInterface),
        });
    }

    videos
}

fn parse_detail_html(html: &str, params: &NetworkStatic) -> Vec<NetworkStatic> {
    let link_re = Regex::new(
        r#"(?is)<a\b[^>]*\bhref\s*=\s*["'](?P<href>[^"']*(?:/vod/play|vodplay|\.m3u8)[^"']*)["'][^>]*>(?P<body>.*?)</a>"#,
    )
    .unwrap();
    let mut seen = HashSet::new();

    link_re
        .captures_iter(html)
        .filter_map(|capture| {
            let href = capture.name("href")?;
            let body = capture
                .name("body")
                .map(|matched| matched.as_str())
                .unwrap_or("");
            Some((abs_url(href.as_str()), episode_name(&params.name, body)))
        })
        .filter(|source| seen.insert(source.clone()))
        .map(|(source, name)| NetworkStatic {
            id: Uuid::new_v4().to_string(),
            name,
            img: params.img.clone(),
            author: params.author.clone(),
            category: params.category.clone(),
            headers: params.headers.clone(),
            source,
            func: params.func.clone(),
        })
        .collect()
}

fn episode_name(video_name: &str, body: &str) -> String {
    let episode = strip_tags(body);
    if episode.is_empty() || video_name.contains(&episode) {
        video_name.to_string()
    } else {
        format!("{} {}", video_name, episode)
    }
}

fn abs_url(path: &str) -> String {
    if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{}/{}", BASE_URL, path.trim_start_matches('/'))
    }
}

fn attr(attrs: &str, name: &str) -> Option<String> {
    let re = Regex::new(&format!(
        r#"(?is)\b{}\s*=\s*["']([^"']+)["']"#,
        regex::escape(name)
    ))
    .unwrap();

    re.captures(attrs)
        .and_then(|capture| capture.get(1))
        .map(|matched| html_unescape(matched.as_str().trim()))
        .filter(|value| !value.is_empty())
}

fn strip_tags(value: &str) -> String {
    let tag_re = Regex::new(r"(?is)<[^>]+>").unwrap();
    let space_re = Regex::new(r"\s+").unwrap();
    let text = tag_re.replace_all(value, "");
    html_unescape(space_re.replace_all(text.trim(), " ").trim())
}

fn html_unescape(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}
