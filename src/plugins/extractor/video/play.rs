use super::FetchDocument;
use crate::drive::{NetworkStatic, NetworkStaticInterface};
use crate::plugins::extractor::config::{self, PlatformConfig};

pub(crate) struct ConfiguredVideoInterface {
    pub(crate) config: PlatformConfig,
    pub(crate) fetcher: FetchDocument,
}

impl NetworkStaticInterface for ConfiguredVideoInterface {
    fn download(&self, _params: &NetworkStatic) {}

    fn play(&self, params: &NetworkStatic) -> String {
        if params.source.contains(".m3u8") || params.source.contains(".mp4") {
            return params.source.clone();
        }
        let Some(detail) = self.config.item_children.detail.as_ref() else {
            return params.source.clone();
        };
        let Some(play) = detail.play.as_ref() else {
            return params.source.clone();
        };
        let url = play
            .base_url
            .as_deref()
            .map(|template| config::resolve_template(&params.source, template, &params.source))
            .unwrap_or_else(|| params.source.clone());
        let config = self.config.clone();
        let fetcher = self.fetcher.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let extract_type = config::play_extract_type(&config, play);
                let Ok(document) = fetcher(url.clone(), config.clone(), extract_type).await else {
                    log::error!("video play request failed: url={url}");
                    return String::new();
                };
                config::extract_play_url(&document, play, &url, &config).unwrap_or_default()
            })
        })
    }

    fn detail(&self, params: &NetworkStatic) -> Vec<NetworkStatic> {
        if params.source.contains("/vod/play")
            || params.source.contains("vodplay")
            || params.source.contains(".m3u8")
        {
            return vec![params.clone()];
        }
        let Some(detail) = self.config.item_children.detail.as_ref() else {
            return vec![params.clone()];
        };
        let Some(children) = detail.item_children.as_ref() else {
            return vec![params.clone()];
        };
        let detail_url = detail
            .base_url
            .as_deref()
            .map(|template| config::resolve_template(&params.source, template, &params.source))
            .unwrap_or_else(|| params.source.clone());
        let child_url = children
            .base_url
            .as_deref()
            .map(|template| config::resolve_template(&detail_url, template, &params.source))
            .unwrap_or_else(|| detail_url.clone());
        let config = self.config.clone();
        let fetcher = self.fetcher.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let extract_type = config::detail_extract_type(&config, detail, children);
                let Ok(document) = fetcher(child_url.clone(), config.clone(), extract_type).await
                else {
                    log::error!("video detail request failed: url={child_url}");
                    return vec![params.clone()];
                };
                let values = config::parse_items(&document, children, &child_url);
                if values.is_empty() {
                    return vec![params.clone()];
                }
                values
                    .into_iter()
                    .map(|item| NetworkStatic {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: item.name,
                        img: if item.image.is_empty() {
                            params.img.clone()
                        } else {
                            item.image
                        },
                        author: params.author.clone(),
                        category: params.category.clone(),
                        headers: params.headers.clone(),
                        extra: item.extra,
                        source: item.source,
                        func: params.func.clone(),
                    })
                    .collect()
            })
        })
    }
}
