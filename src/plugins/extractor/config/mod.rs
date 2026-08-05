use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::plugins::extractor::config::audio_config::AUDIO_RECOMMEND_CONFIG;
use crate::plugins::extractor::config::video_config::{
    VIDEO_RECOMMEND_CONFIG, VIDEO_SEARCH_CONFIG,
};

mod audio_config;
mod video_config;

pub(crate) use super::template::ExtractedDocument;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
    pub attribute: Option<String>,
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
pub struct ChildrenConfig {
    pub item_selector: String,
    #[serde(default)]
    pub extract_type: Option<ExtractType>,
    pub name: FieldConfig,
    #[serde(default)]
    pub author: Option<FieldConfig>,
    pub image: Option<FieldConfig>,
    pub play_url: FieldConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageConfig {
    pub url: String,
    pub category: String,
    pub item_selector: String,
    pub name: FieldConfig,
    #[serde(default)]
    pub author: Option<FieldConfig>,
    pub image: Option<FieldConfig>,
    pub detail_url: FieldConfig,
    #[serde(default)]
    pub extra: HashMap<String, FieldConfig>,
    #[serde(default)]
    pub children_url: Option<String>,
    pub children: Option<ChildrenConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub id: String,
    pub resource_type: ResourceType,
    pub extract_type: ExtractType,
    pub headers: HashMap<String, String>,
    pub search: Option<PageConfig>,
    pub recommend: Vec<PageConfig>,
    pub play_regex: Option<String>,
    #[serde(default)]
    pub play_url: Option<String>,
    #[serde(default)]
    pub play_selector: Option<String>,
}

pub fn load_from_dir(
    path: impl AsRef<std::path::Path>,
    resource_type: ResourceType,
) -> Vec<PlatformConfig> {
    let Ok(entries) = std::fs::read_dir(path) else {
        return Vec::new();
    };
    let mut configs = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .filter_map(|path| {
            let content = std::fs::read_to_string(&path).ok()?;
            let value = serde_json::from_str::<serde_json::Value>(&content).ok()?;
            if let Some(items) = value.as_array() {
                return Some(
                    items
                        .iter()
                        .filter_map(|item| {
                            serde_json::from_value::<PlatformConfig>(item.clone()).ok()
                        })
                        .filter(|config| config.resource_type == resource_type)
                        .collect::<Vec<_>>(),
                );
            }
            let config = serde_json::from_value::<PlatformConfig>(value).ok()?;
            Some(
                (config.resource_type == resource_type)
                    .then_some(config)
                    .into_iter()
                    .collect(),
            )
        })
        .flatten()
        .collect::<Vec<_>>();
    configs.sort_by(|left, right| left.id.cmp(&right.id));
    configs
}

pub fn load_default(resource_type: ResourceType) -> Vec<PlatformConfig> {
    let folder = match resource_type {
        ResourceType::Video => "video",
        ResourceType::Audio => "audio",
    };
    let mut configs = match resource_type {
        ResourceType::Video => {
            let mut configs = parse_configs(VIDEO_SEARCH_CONFIG, resource_type);
            configs.extend(parse_configs(VIDEO_RECOMMEND_CONFIG, resource_type));
            configs
        }
        ResourceType::Audio => parse_configs(AUDIO_RECOMMEND_CONFIG, resource_type),
    };
    let external = [
        std::path::PathBuf::from("plugins").join(folder),
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("plugins")
            .join(folder),
    ]
    .into_iter()
    .find_map(|path| {
        let configs = load_from_dir(path, resource_type);
        (!configs.is_empty()).then_some(configs)
    })
    .unwrap_or_default();
    configs.extend(external);
    configs.sort_by(|left, right| left.id.cmp(&right.id));
    configs
}

fn parse_configs(content: &str, resource_type: ResourceType) -> Vec<PlatformConfig> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return Vec::new();
    };
    let values = value.as_array().cloned().unwrap_or_else(|| vec![value]);
    values
        .into_iter()
        .filter_map(|value| serde_json::from_value::<PlatformConfig>(value).ok())
        .filter(|config| config.resource_type == resource_type)
        .collect()
}

pub(crate) fn headers(config: &PlatformConfig) -> reqwest::header::HeaderMap {
    super::template::css::headers(&config.headers)
}

pub(crate) async fn fetch_document(
    url: &str,
    config: &PlatformConfig,
) -> anyhow::Result<super::template::ExtractedDocument> {
    fetch_document_with_type(url, config, config.extract_type).await
}

pub(crate) async fn fetch_document_with_type(
    url: &str,
    config: &PlatformConfig,
    extract_type: ExtractType,
) -> anyhow::Result<super::template::ExtractedDocument> {
    match extract_type {
        ExtractType::Json => super::template::json::fetch(url, config)
            .await
            .map(super::template::ExtractedDocument::Json),
        ExtractType::Css | ExtractType::Regex => super::template::css::fetch(url, &headers(config))
            .await
            .map(super::template::ExtractedDocument::Html),
    }
}

pub(crate) fn children_extract_type(
    config: &PlatformConfig,
    children: &ChildrenConfig,
) -> ExtractType {
    children.extract_type.unwrap_or(config.extract_type)
}

pub(crate) fn search_url(template: &str, keyword: &str) -> String {
    super::template::css::search_url(template, keyword)
}

pub(crate) fn base_url(value: &str) -> Option<String> {
    reqwest::Url::parse(value).ok().map(|url| {
        format!(
            "{}://{}{}",
            url.scheme(),
            url.host_str().unwrap_or_default(),
            url.port()
                .map(|port| format!(":{port}"))
                .unwrap_or_default()
        )
    })
}

pub(crate) fn parse_page(
    document: &super::template::ExtractedDocument,
    page: &PageConfig,
    base: &str,
) -> Vec<super::template::ExtractedItem> {
    match document {
        super::template::ExtractedDocument::Html(body) => {
            super::template::css::parse_page(body, page, base)
        }
        super::template::ExtractedDocument::Json(document) => {
            super::template::json::parse_page(document, page, base)
        }
    }
}

pub(crate) fn parse_children(
    document: &super::template::ExtractedDocument,
    children: &ChildrenConfig,
    base: &str,
) -> Vec<(String, String, String)> {
    match document {
        super::template::ExtractedDocument::Html(body) => {
            super::template::css::parse_children(body, children, base)
        }
        super::template::ExtractedDocument::Json(document) => {
            super::template::json::parse_children(document, children, base)
        }
    }
}

pub(crate) fn extract_play_url(
    document: &super::template::ExtractedDocument,
    config: &PlatformConfig,
    base: &str,
) -> Option<String> {
    match (&config.extract_type, document) {
        (ExtractType::Json, super::template::ExtractedDocument::Json(document)) => config
            .play_selector
            .as_deref()
            .and_then(|selector| super::template::json::json_string(document, selector)),
        (ExtractType::Css | ExtractType::Regex, super::template::ExtractedDocument::Html(body)) => {
            config
                .play_regex
                .as_deref()
                .and_then(|pattern| super::template::regex::extract(body, pattern, base))
        }
        _ => None,
    }
}

pub(crate) fn field_value(value: &Value, field: &FieldConfig) -> Option<String> {
    super::template::json::field_value(value, field)
}

pub(crate) fn json_path<'a>(value: &'a Value, selector: &str) -> Option<&'a Value> {
    super::template::json::json_path(value, selector)
}

pub(crate) fn fill_template(template: &str, value: &str) -> String {
    super::template::json::fill_template(template, value)
}
