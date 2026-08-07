use super::{FetchDocument, default_fetcher, default_plugins, play::filter_playable};
use crate::drive::NetworkStatic;

use crate::plugins::extractor::config::{self, PlatformConfig};
use futures_util::future::join_all;

pub async fn recommend() -> Vec<NetworkStatic> {
    recommend_with_configs(default_plugins(), default_fetcher()).await
}

async fn recommend_with_configs(
    configs: Vec<PlatformConfig>,
    fetcher: FetchDocument,
) -> Vec<NetworkStatic> {
    let result = join_all(
        configs
            .into_iter()
            .filter(|config| !config::is_search_config(config))
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
    let url = config::entry_url(&config, None);
    match fetcher(
        url.clone(),
        config.clone(),
        config::item_extract_type(&config),
    )
    .await
    {
        Ok(body) => {
            let items = super::search::build_items(&body, &config, &url, fetcher);
            let playable = filter_playable(items).await;
            log::debug!(
                "video recommend source={} playable={}",
                config.id,
                playable.len()
            );
            playable
        }
        Err(error) => {
            log::error!("request {} error: {:#}", config.id, error);
            Vec::new()
        }
    }
}
