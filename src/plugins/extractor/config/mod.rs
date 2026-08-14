use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::plugins::extractor::config::audio_config::AUDIO_RECOMMEND_CONFIG;
use crate::plugins::extractor::config::video_config::{
    VIDEO_RECOMMEND_CONFIG, VIDEO_SEARCH_CONFIG,
};

mod audio_config;
mod legacy;
mod video_config;

pub(crate) use super::template::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Video,
    Audio,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtractType {
    Css,
    Json,
    Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfig {
    pub selector: String,
    #[serde(default)]
    pub attribute: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSplitConfig {
    pub item_separator: String,
    #[serde(default)]
    pub field_separator: Option<String>,
}

impl FieldConfig {
    pub fn text(selector: &str) -> Self {
        Self {
            selector: selector.to_string(),
            attribute: None,
        }
    }

    pub fn attribute(selector: &str, attribute: &str) -> Self {
        Self {
            selector: selector.to_string(),
            attribute: Some(attribute.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemChildrenConfig {
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub extract_type: Option<ExtractType>,
    pub item_selector: String,
    pub source: FieldConfig,
    pub name: FieldConfig,
    #[serde(default)]
    pub author: Option<FieldConfig>,
    #[serde(default)]
    pub image: Option<FieldConfig>,
    #[serde(default)]
    pub extra: HashMap<String, FieldConfig>,
    #[serde(default = "default_fallback_play_links")]
    pub fallback_play_links: bool,
    #[serde(default)]
    pub item_split: Option<ItemSplitConfig>,
    #[serde(default)]
    pub detail: Option<Box<DetailConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailConfig {
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub extract_type: Option<ExtractType>,
    #[serde(default)]
    pub item_children: Option<Box<ItemChildrenConfig>>,
    #[serde(default)]
    pub play: Option<PlayConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayConfig {
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub extract_type: Option<ExtractType>,
    #[serde(default)]
    pub selector: Option<String>,
    #[serde(default)]
    pub regex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub id: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub base_url: String,
    pub extract_type: ExtractType,
    #[serde(default)]
    pub category: String,
    pub item_children: ItemChildrenConfig,
}

fn default_fallback_play_links() -> bool {
    true
}

pub fn load_from_dir(
    path: impl AsRef<std::path::Path>,
    resource_type: ResourceType,
) -> Vec<PlatformConfig> {
    let Ok(entries) = std::fs::read_dir(path.as_ref()) else {
        return Vec::new();
    };
    let mut configs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            log::warn!("failed to read extractor config {}", path.display());
            continue;
        };
        configs.extend(parse_configs(
            &content,
            resource_type,
            path.display().to_string(),
        ));
    }
    configs.sort_by(|left, right| left.id.cmp(&right.id));
    configs
}

pub fn load_default(resource_type: ResourceType) -> Vec<PlatformConfig> {
    let mut configs = match resource_type {
        ResourceType::Video => {
            let mut configs = parse_configs(
                VIDEO_SEARCH_CONFIG,
                resource_type,
                "embedded video search config".into(),
            );
            configs.extend(parse_configs(
                VIDEO_RECOMMEND_CONFIG,
                resource_type,
                "embedded video recommend config".into(),
            ));
            configs
        }
        ResourceType::Audio => parse_configs(
            AUDIO_RECOMMEND_CONFIG,
            resource_type,
            "embedded audio recommend config".into(),
        ),
    };
    let folder = match resource_type {
        ResourceType::Video => "video",
        ResourceType::Audio => "audio",
    };
    let external = [
        std::path::PathBuf::from("plugins").join(folder),
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("plugins")
            .join(folder),
    ]
    .into_iter()
    .find_map(|path| {
        let values = load_from_dir(path, resource_type);
        (!values.is_empty()).then_some(values)
    })
    .unwrap_or_default();
    if !external.is_empty() {
        let external_ids = external
            .iter()
            .map(|config| config.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        configs.retain(|config| !external_ids.contains(config.id.as_str()));
        configs.extend(external);
    }
    configs.sort_by(|left, right| left.id.cmp(&right.id));
    configs
}

fn parse_configs(
    content: &str,
    resource_type: ResourceType,
    origin: String,
) -> Vec<PlatformConfig> {
    let Ok(value) = serde_json::from_str::<Value>(content) else {
        log::warn!("invalid extractor config JSON: {origin}");
        return Vec::new();
    };
    value
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![value])
        .into_iter()
        .flat_map(|value| parse_config(value, resource_type, &origin))
        .collect()
}

fn parse_config(value: Value, resource_type: ResourceType, origin: &str) -> Vec<PlatformConfig> {
    if value.get("item_children").is_some() {
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>")
            .to_string();
        let Ok(mut config) = serde_json::from_value::<PlatformConfig>(value) else {
            log::warn!("invalid extractor config {origin}, id={id}");
            return Vec::new();
        };
        if config.category.is_empty() {
            config.category = config.id.clone();
        }
        return vec![config];
    }
    legacy::parse(value, resource_type, origin)
}

pub(crate) fn entry_url(config: &PlatformConfig, keyword: Option<&str>) -> String {
    let path = config
        .item_children
        .base_url
        .as_deref()
        .unwrap_or(&config.base_url);
    let url = css::resolve(&config.base_url, path);
    keyword
        .map(|keyword| search_url(&url, keyword))
        .unwrap_or(url)
}

pub(crate) fn is_search_config(config: &PlatformConfig) -> bool {
    config
        .item_children
        .base_url
        .as_deref()
        .is_some_and(|url| url.contains("{{keyword}}"))
}

pub(crate) fn item_extract_type(config: &PlatformConfig) -> ExtractType {
    config
        .item_children
        .extract_type
        .unwrap_or(config.extract_type)
}

pub(crate) fn detail_extract_type(
    config: &PlatformConfig,
    detail: &DetailConfig,
    children: &ItemChildrenConfig,
) -> ExtractType {
    children
        .extract_type
        .or(detail.extract_type)
        .unwrap_or(config.extract_type)
}

pub(crate) fn play_extract_type(config: &PlatformConfig, play: &PlayConfig) -> ExtractType {
    play.extract_type.unwrap_or(config.extract_type)
}

pub(crate) fn headers(config: &PlatformConfig) -> reqwest::header::HeaderMap {
    css::headers(&config.headers)
}

pub(crate) fn video_headers(
    config: &PlatformConfig,
    request_url: &str,
    navigation: bool,
) -> reqwest::header::HeaderMap {
    let mut headers = headers_with_browser_defaults(config, navigation);
    if !headers.contains_key(reqwest::header::REFERER) {
        headers.insert(
            reqwest::header::REFERER,
            reqwest::header::HeaderValue::try_from(format!(
                "{}/",
                config.base_url.trim_end_matches('/')
            ))
            .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("https://localhost/")),
        );
    }
    if !headers.contains_key(reqwest::header::ORIGIN) {
        if let Ok(origin) =
            reqwest::header::HeaderValue::try_from(config.base_url.trim_end_matches('/'))
        {
            headers.insert(reqwest::header::ORIGIN, origin);
        }
    }
    // log::info!(
    //     "[video:http] url={} navigation={} headers={:?}",
    //     request_url,
    //     navigation,
    //     headers.keys().map(|name| name.as_str()).collect::<Vec<_>>()
    // );
    headers
}

fn headers_with_browser_defaults(
    config: &PlatformConfig,
    navigation: bool,
) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    let defaults = [
        (
            "user-agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        ),
        ("accept-language", "zh-CN,zh;q=0.9,en;q=0.8"),
        ("cache-control", "no-cache"),
        ("pragma", "no-cache"),
        (
            "sec-ch-ua",
            r#""Not_A Brand";v="99", "Chromium";v="131", "Google Chrome";v="131""#,
        ),
        ("sec-ch-ua-mobile", "?0"),
        ("sec-ch-ua-platform", "\"Windows\""),
    ];
    for (name, value) in defaults {
        if let (Ok(name), Ok(value)) = (
            reqwest::header::HeaderName::try_from(name),
            reqwest::header::HeaderValue::try_from(value),
        ) {
            headers.insert(name, value);
        }
    }
    let request_headers: Vec<(&str, &str)> = if navigation {
        vec![
            (
                "accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
            ),
            ("sec-fetch-dest", "document"),
            ("sec-fetch-mode", "navigate"),
            ("sec-fetch-site", "none"),
            ("sec-fetch-user", "?1"),
            ("upgrade-insecure-requests", "1"),
        ]
    } else {
        vec![
            ("accept", "application/json, text/plain, */*"),
            ("sec-fetch-dest", "empty"),
            ("sec-fetch-mode", "cors"),
            ("sec-fetch-site", "same-origin"),
        ]
    };
    for (name, value) in request_headers {
        if let (Ok(name), Ok(value)) = (
            reqwest::header::HeaderName::try_from(name),
            reqwest::header::HeaderValue::try_from(value),
        ) {
            headers.insert(name, value);
        }
    }

    for (name, value) in css::headers(&config.headers) {
        if let Some(name) = name {
            headers.insert(name, value);
        }
    }
    headers
}

pub(crate) async fn fetch_document(
    url: &str,
    config: &PlatformConfig,
    extract_type: ExtractType,
) -> anyhow::Result<ExtractedDocument> {
    match extract_type {
        ExtractType::Json => json::fetch(url, config).await.map(ExtractedDocument::Json),
        ExtractType::Css | ExtractType::Regex => css::fetch(url, &headers(config))
            .await
            .map(ExtractedDocument::Html),
    }
}

pub(crate) async fn fetch_video_document(
    url: &str,
    config: &PlatformConfig,
    extract_type: ExtractType,
) -> anyhow::Result<ExtractedDocument> {
    match extract_type {
        ExtractType::Json => {
            json::fetch_with_headers(url, config, video_headers(config, url, false))
                .await
                .map(ExtractedDocument::Json)
        }
        ExtractType::Css | ExtractType::Regex => css::fetch(url, &video_headers(config, url, true))
            .await
            .map(ExtractedDocument::Html),
    }
}

pub(crate) fn parse_items(
    document: &ExtractedDocument,
    items: &ItemChildrenConfig,
    base: &str,
) -> Vec<ExtractedItem> {
    match document {
        ExtractedDocument::Html(body) => css::parse_items(body, items, base),
        ExtractedDocument::Json(document) => json::parse_items(document, items, base),
    }
}

pub(crate) fn extract_play_url(
    document: &ExtractedDocument,
    play: &PlayConfig,
    base: &str,
    config: &PlatformConfig,
) -> Option<String> {
    match (play_extract_type(config, play), document) {
        (ExtractType::Json, ExtractedDocument::Json(document)) => play
            .selector
            .as_deref()
            .and_then(|selector| json::json_string(document, selector)),
        (ExtractType::Css | ExtractType::Regex, ExtractedDocument::Html(body)) => play
            .regex
            .as_deref()
            .and_then(|pattern| regex::extract(body, pattern, base)),
        _ => None,
    }
}

pub(crate) fn fill_template(template: &str, value: &str) -> String {
    json::fill_template(template, value)
}

pub(crate) fn resolve_template(base: &str, template: &str, value: &str) -> String {
    css::resolve(base, &fill_template(template, value))
}

pub(crate) fn search_url(template: &str, keyword: &str) -> String {
    css::search_url(template, keyword)
}
