use super::{FetchDocument, default_fetcher, default_plugins, play::ConfiguredVideoInterface};
use crate::drive::NetworkStatic;

use crate::plugins::extractor::config::{self, ExtractedDocument, PageConfig, PlatformConfig};
use futures_util::future::{BoxFuture, join_all};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub async fn search(keyword: String) -> HashMap<String, Vec<NetworkStatic>> {
    search_with_configs(default_plugins(), keyword, default_fetcher()).await
}

async fn search_with_configs(
    configs: Vec<PlatformConfig>,
    keyword: String,
    fetcher: FetchDocument,
) -> HashMap<String, Vec<NetworkStatic>> {
    log::debug!("video search started: keyword={:?}", keyword);
    let tasks: Vec<(String, BoxFuture<'static, Vec<NetworkStatic>>)> = configs
        .into_iter()
        .map(|config| {
            let name = config.id.clone();
            let task = Box::pin(search_one(config, keyword.clone(), fetcher.clone()))
                as BoxFuture<'static, Vec<NetworkStatic>>;
            (name, task)
        })
        .collect();
    let names = tasks
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let values = join_all(tasks.into_iter().map(|(_, task)| task)).await;
    names.into_iter().zip(values).collect::<HashMap<_, _>>()
}

async fn search_one(
    config: PlatformConfig,
    keyword: String,
    fetcher: FetchDocument,
) -> Vec<NetworkStatic> {
    let Some(page) = config.search.as_ref() else {
        return Vec::new();
    };
    let url = config::search_url(&page.url, &keyword);
    match fetcher(url.clone(), config.clone()).await {
        Ok(body) => build_items(&body, page, &config, &url, fetcher),
        Err(error) => {
            log::error!("request {} error: {:#}", config.id, error);
            Vec::new()
        }
    }
}

pub(crate) fn build_items(
    document: &ExtractedDocument,
    page: &PageConfig,
    config: &PlatformConfig,
    page_url: &str,
    fetcher: FetchDocument,
) -> Vec<NetworkStatic> {
    let Some(base) = super::play::base_url(page_url) else {
        return Vec::new();
    };
    config::parse_page(document, page, &base)
        .into_iter()
        .map(|item| NetworkStatic {
            id: Uuid::new_v4().to_string(),
            name: item.name,
            img: item.image,
            author: config.id.clone(),
            category: page.category.clone(),
            headers: config::headers(config),
            extra: item.extra,
            source: item.source,
            func: Arc::new(ConfiguredVideoInterface {
                config: config.clone(),
                fetcher: fetcher.clone(),
            }),
        })
        .collect()
}
