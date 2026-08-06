use super::default_plugins;
use super::play::ConfiguredAudioInterface;
use crate::drive::NetworkStatic;
use crate::plugins::extractor::config::{self, ExtractedDocument, ExtractedItem, PlatformConfig};
use futures_util::future::join_all;
use std::collections::HashSet;
use std::sync::Arc;

pub async fn recommend() -> Vec<NetworkStatic> {
    let configs = default_plugins()
        .into_iter()
        .filter(|config| !config::is_search_config(config))
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
    let url = config::entry_url(&config, None);
    let extract_type = config::item_extract_type(&config);
    let Ok(document) = config::fetch_document(&url, &config, extract_type).await else {
        log::error!(
            "audio recommend request failed: extractor={}, url={url}",
            config.id
        );
        return Vec::new();
    };
    let items = config::parse_items(&document, &config.item_children, &url);
    let results = join_all(
        items
            .into_iter()
            .map(|item| expand_item(item, config.clone(), url.clone())),
    )
    .await;
    results.into_iter().flatten().collect()
}

async fn expand_item(
    item: ExtractedItem,
    config: PlatformConfig,
    entry_url: String,
) -> Vec<NetworkStatic> {
    let Some(detail) = config.item_children.detail.as_ref() else {
        return vec![build_audio_static(item, &config)];
    };
    let Some(children) = detail.item_children.as_ref() else {
        return vec![build_audio_static(item, &config)];
    };

    let detail_url = detail
        .base_url
        .as_deref()
        .map(|template| config::resolve_template(&entry_url, template, &item.source));
    let child_url = children
        .base_url
        .as_deref()
        .and_then(|template| {
            detail_url
                .as_deref()
                .map(|base| config::resolve_template(base, template, &item.source))
        })
        .or_else(|| detail_url.clone());

    let Some(child_url) = child_url else {
        let Some(raw) = item.raw.as_ref() else {
            return Vec::new();
        };
        let document = ExtractedDocument::Json(raw.clone());
        return config::parse_items(&document, children, &entry_url)
            .into_iter()
            .map(|item| build_audio_static(item, &config))
            .collect();
    };

    let extract_type = config::detail_extract_type(&config, detail, children);
    let Ok(document) = config::fetch_document(&child_url, &config, extract_type).await else {
        log::error!(
            "audio children request failed: extractor={}, url={child_url}",
            config.id
        );
        return Vec::new();
    };
    config::parse_items(&document, children, &child_url)
        .into_iter()
        .map(|item| build_audio_static(item, &config))
        .collect()
}

fn build_audio_static(item: ExtractedItem, config: &PlatformConfig) -> NetworkStatic {
    NetworkStatic {
        id: item.source.clone(),
        name: item.name,
        img: item.image,
        author: if item.author.is_empty() {
            config.id.clone()
        } else {
            item.author
        },
        category: config.category.clone(),
        headers: config::headers(config),
        extra: item.extra,
        source: item.source,
        func: Arc::new(ConfiguredAudioInterface {
            config: config.clone(),
        }),
    }
}
