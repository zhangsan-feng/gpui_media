use super::default_plugins;
use super::play::ConfiguredAudioInterface;
use crate::drive::NetworkStatic;

use crate::plugins::extractor::config::{
    ExtractedDocument, FieldConfig, PageConfig, PlatformConfig, base_url, children_extract_type,
    fetch_document, fetch_document_with_type, field_value, fill_template, headers, json_path,
    parse_page,
};
use futures_util::future::join_all;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;

pub async fn recommend() -> Vec<NetworkStatic> {
    let configs = default_plugins()
        .into_iter()
        .filter(|config| !config.recommend.is_empty())
        .collect::<Vec<_>>();
    let results = join_all(configs.into_iter().map(recommend_one)).await;
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for items in results {
        for item in items {
            if seen.insert(format!("{}:{}", item.author, item.id)) {
                result.push(item);
            }
        }
    }

    log::debug!("audio recommend completed: items={}", result.len());
    result
}

async fn recommend_one(config: PlatformConfig) -> Vec<NetworkStatic> {
    let mut result = Vec::new();

    for page in config.recommend.clone() {
        match fetch_document(&page.url, &config).await {
            Ok(ExtractedDocument::Json(document)) => {
                result.extend(expand_page(&document, &page, &config).await)
            }
            Ok(document) => result.extend(expand_html_page(&document, &page, &config)),
            Err(error) => log::error!(
                "audio recommend request failed: extractor={}, url={}, error={:#}",
                config.id,
                page.url,
                error
            ),
        }
    }

    result
}

fn expand_html_page(
    document: &ExtractedDocument,
    page: &PageConfig,
    config: &PlatformConfig,
) -> Vec<NetworkStatic> {
    let Some(base) = base_url(&page.url) else {
        return Vec::new();
    };
    parse_page(document, page, &base)
        .into_iter()
        .map(|item| {
            build_audio_static(
                item.source,
                item.name,
                item.image,
                item.extra,
                &page.category,
                config,
            )
        })
        .collect()
}

async fn expand_page(
    document: &Value,
    page: &PageConfig,
    config: &PlatformConfig,
) -> Vec<NetworkStatic> {
    let Some(items) = json_path(document, &page.item_selector).and_then(Value::as_array) else {
        log::debug!(
            "audio page items not found: extractor={}, selector={}",
            config.id,
            page.item_selector
        );
        return Vec::new();
    };

    let tasks = items
        .iter()
        .cloned()
        .map(|item| expand_item(item, page.clone(), config.clone()))
        .collect::<Vec<_>>();
    join_all(tasks).await.into_iter().flatten().collect()
}

async fn expand_item(item: Value, page: PageConfig, config: PlatformConfig) -> Vec<NetworkStatic> {
    let parent_id = field_value(&item, &page.detail_url).unwrap_or_default();
    let parent_author = page
        .author
        .as_ref()
        .and_then(|field| field_value(&item, field))
        .unwrap_or_default();
    let parent_image = page
        .image
        .as_ref()
        .and_then(|field| field_value(&item, field))
        .unwrap_or_default();

    let Some(children) = page.children.as_ref() else {
        return build_items(
            std::slice::from_ref(&item),
            &page.detail_url,
            &page.name,
            page.author.as_ref(),
            page.image.as_ref(),
            &page.category,
            &config,
            &parent_author,
            &parent_image,
        );
    };

    let children_document = if let Some(url_template) = page.children_url.as_ref() {
        let url = fill_template(url_template, &parent_id);
        let extract_type = children_extract_type(&config, children);
        match fetch_document_with_type(&url, &config, extract_type).await {
            Ok(ExtractedDocument::Json(document)) => document,
            Err(error) => {
                log::error!(
                    "audio children request failed: extractor={}, url={}, error={:#}",
                    config.id,
                    url,
                    error
                );
                return Vec::new();
            }
            Ok(ExtractedDocument::Html(body)) => {
                let document = ExtractedDocument::Html(body);
                let Some(base) = base_url(&url) else {
                    return Vec::new();
                };
                return super::config::parse_children(&document, children, &base)
                    .into_iter()
                    .map(|(source, name, image)| {
                        build_audio_static(
                            source,
                            name,
                            image,
                            Default::default(),
                            &page.category,
                            &config,
                        )
                    })
                    .collect();
            }
        }
    } else {
        item
    };

    let Some(children_values) =
        json_path(&children_document, &children.item_selector).and_then(Value::as_array)
    else {
        return Vec::new();
    };

    children_values
        .iter()
        .filter_map(|child| {
            build_audio_item(
                child,
                &children.name,
                children.author.as_ref(),
                children.image.as_ref(),
                &children.play_url,
                &page.category,
                &config,
                &parent_author,
                &parent_image,
            )
        })
        .collect()
}

fn build_items(
    items: &[Value],
    source_field: &FieldConfig,
    name_field: &FieldConfig,
    author_field: Option<&FieldConfig>,
    image_field: Option<&FieldConfig>,
    category: &str,
    config: &PlatformConfig,
    fallback_author: &str,
    fallback_image: &str,
) -> Vec<NetworkStatic> {
    items
        .iter()
        .filter_map(|item| {
            build_audio_item(
                item,
                name_field,
                author_field,
                image_field,
                source_field,
                category,
                config,
                fallback_author,
                fallback_image,
            )
        })
        .collect()
}

fn build_audio_item(
    item: &Value,
    name_field: &FieldConfig,
    author_field: Option<&FieldConfig>,
    image_field: Option<&FieldConfig>,
    source_field: &FieldConfig,
    category: &str,
    config: &PlatformConfig,
    fallback_author: &str,
    fallback_image: &str,
) -> Option<NetworkStatic> {
    let source = field_value(item, source_field)?;
    let name = field_value(item, name_field).unwrap_or_else(|| "未命名音频".to_string());
    let author = author_field
        .and_then(|field| field_value(item, field))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback_author.to_string());
    let image = image_field
        .and_then(|field| field_value(item, field))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback_image.to_string());

    Some(NetworkStatic {
        id: source.clone(),
        name,
        img: image,
        author,
        category: category.to_string(),
        headers: headers(config),
        extra: Default::default(),
        source,
        func: Arc::new(ConfiguredAudioInterface {
            config: config.clone(),
        }),
    })
}

fn build_audio_static(
    source: String,
    name: String,
    image: String,
    extra: std::collections::HashMap<String, Value>,
    category: &str,
    config: &PlatformConfig,
) -> NetworkStatic {
    NetworkStatic {
        id: source.clone(),
        name,
        img: image,
        author: config.id.clone(),
        category: category.to_string(),
        headers: headers(config),
        extra,
        source,
        func: Arc::new(ConfiguredAudioInterface {
            config: config.clone(),
        }),
    }
}
