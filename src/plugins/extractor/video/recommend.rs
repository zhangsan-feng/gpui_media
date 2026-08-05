use super::{FetchDocument, append_unique, default_fetcher, default_plugins};
use crate::drive::NetworkStatic;

use crate::plugins::extractor::config::PlatformConfig;
use futures_util::future::join_all;
use std::collections::HashSet;

pub async fn recommend() -> Vec<NetworkStatic> {
    recommend_with_configs(
        default_plugins()
            .into_iter()
            .filter(|config| !config.recommend.is_empty())
            .collect::<Vec<_>>(),
        default_fetcher(),
    )
    .await
}

async fn recommend_with_configs(
    configs: Vec<PlatformConfig>,
    fetcher: FetchDocument,
) -> Vec<NetworkStatic> {
    let result = join_all(
        configs
            .into_iter()
            .map(|config| recommend_one(config, fetcher.clone())),
    )
    .await
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    log::debug!("video recommend completed: items={}", result.len());
    result
}

async fn recommend_one(config: PlatformConfig, fetcher: FetchDocument) -> Vec<NetworkStatic> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for page in &config.recommend {
        let url = page.url.clone();
        match fetcher(url.clone(), config.clone()).await {
            Ok(body) => {
                let items = super::search::build_items(&body, page, &config, &url, fetcher.clone());
                append_unique(&mut result, &mut seen, items);
            }
            Err(error) => log::error!("request {} error: {:#}", config.id, error),
        }
    }
    result
}
