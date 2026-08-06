use super::{
    DetailConfig, ExtractType, FieldConfig, ItemChildrenConfig, PlatformConfig, PlayConfig,
    ResourceType,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct LegacyPlatformConfig {
    id: String,
    #[serde(default)]
    resource_type: Option<ResourceType>,
    extract_type: ExtractType,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    search: Option<LegacyPageConfig>,
    #[serde(default)]
    recommend: Vec<LegacyPageConfig>,
    #[serde(default)]
    play_regex: Option<String>,
    #[serde(default)]
    play_url: Option<String>,
    #[serde(default)]
    play_selector: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyPageConfig {
    url: String,
    #[serde(default)]
    category: String,
    item_selector: String,
    name: FieldConfig,
    #[serde(default)]
    author: Option<FieldConfig>,
    #[serde(default)]
    image: Option<FieldConfig>,
    detail_url: FieldConfig,
    #[serde(default)]
    extra: HashMap<String, FieldConfig>,
    #[serde(default)]
    children_url: Option<String>,
    #[serde(default)]
    children: Option<LegacyChildrenConfig>,
}

#[derive(Debug, Deserialize)]
struct LegacyChildrenConfig {
    item_selector: String,
    #[serde(default)]
    extract_type: Option<ExtractType>,
    name: FieldConfig,
    #[serde(default)]
    author: Option<FieldConfig>,
    #[serde(default)]
    image: Option<FieldConfig>,
    play_url: FieldConfig,
}

pub(super) fn parse(
    value: Value,
    resource_type: ResourceType,
    origin: &str,
) -> Vec<PlatformConfig> {
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>")
        .to_string();
    let Ok(mut config) = serde_json::from_value::<LegacyPlatformConfig>(value) else {
        log::warn!("invalid legacy extractor config {origin}, id={id}");
        return Vec::new();
    };
    if config
        .resource_type
        .is_some_and(|value| value != resource_type)
    {
        return Vec::new();
    }

    let mut result = Vec::new();
    if let Some(page) = config.search.take() {
        result.push(convert_page(&config, page));
    }
    let recommend = std::mem::take(&mut config.recommend);
    result.extend(
        recommend
            .into_iter()
            .map(|page| convert_page(&config, page)),
    );
    result
}

fn convert_page(config: &LegacyPlatformConfig, page: LegacyPageConfig) -> PlatformConfig {
    let play = convert_play(config);
    let detail = if page.children.is_some() || play.is_some() {
        Some(Box::new(DetailConfig {
            base_url: Some(
                page.children_url
                    .clone()
                    .unwrap_or_else(|| "{{source}}".to_string()),
            ),
            extract_type: page
                .children
                .as_ref()
                .and_then(|children| children.extract_type),
            item_children: page.children.map(convert_children).map(Box::new),
            play,
        }))
    } else {
        None
    };
    let base_url = origin_url(&page.url);
    PlatformConfig {
        id: config.id.clone(),
        headers: config.headers.clone(),
        base_url,
        extract_type: config.extract_type,
        category: if page.category.is_empty() {
            config.id.clone()
        } else {
            page.category
        },
        item_children: ItemChildrenConfig {
            base_url: Some(page.url),
            extract_type: None,
            item_selector: page.item_selector,
            source: page.detail_url,
            name: page.name,
            author: page.author,
            image: page.image,
            extra: page.extra,
            fallback_play_links: false,
            item_split: None,
            detail,
        },
    }
}

fn convert_children(children: LegacyChildrenConfig) -> ItemChildrenConfig {
    ItemChildrenConfig {
        base_url: None,
        extract_type: children.extract_type,
        item_selector: children.item_selector,
        source: children.play_url,
        name: children.name,
        author: children.author,
        image: children.image,
        extra: HashMap::new(),
        fallback_play_links: true,
        item_split: None,
        detail: None,
    }
}

fn convert_play(config: &LegacyPlatformConfig) -> Option<PlayConfig> {
    if config.play_regex.is_none() && config.play_selector.is_none() && config.play_url.is_none() {
        return None;
    }
    let extract_type = if config.play_selector.is_some() {
        Some(ExtractType::Json)
    } else if config.play_regex.is_some() {
        Some(ExtractType::Regex)
    } else {
        Some(config.extract_type)
    };
    Some(PlayConfig {
        base_url: config
            .play_url
            .clone()
            .or_else(|| Some("{{source}}".to_string())),
        extract_type,
        selector: config.play_selector.clone(),
        regex: config.play_regex.clone(),
    })
}

fn origin_url(value: &str) -> String {
    reqwest::Url::parse(value)
        .map(|url| {
            format!(
                "{}://{}{}",
                url.scheme(),
                url.host_str().unwrap_or_default(),
                url.port()
                    .map(|port| format!(":{port}"))
                    .unwrap_or_default()
            )
        })
        .unwrap_or_else(|_| value.to_string())
}
