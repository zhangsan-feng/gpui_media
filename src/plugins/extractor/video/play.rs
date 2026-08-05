use super::FetchDocument;
use crate::drive::{NetworkStatic, NetworkStaticInterface};
use crate::plugins::extractor::config::{self, ExtractType, PlatformConfig};

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
        if self.config.play_regex.is_none()
            && !matches!(self.config.extract_type, ExtractType::Json)
        {
            return params.source.clone();
        }
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let document = (self.fetcher)(params.source.clone(), self.config.clone())
                    .await
                    .expect("extractor play page request failed");
                config::extract_play_url(&document, &self.config, &params.source)
                    .unwrap_or_default()
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
        let Some(page) = self
            .config
            .search
            .as_ref()
            .or_else(|| self.config.recommend.first())
        else {
            return vec![params.clone()];
        };
        let Some(children) = &page.children else {
            return vec![params.clone()];
        };
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let extract_type = config::children_extract_type(&self.config, children);
                let Ok(document) = config::fetch_document_with_type(
                    params.source.as_str(),
                    &self.config,
                    extract_type,
                )
                .await
                else {
                    return vec![params.clone()];
                };
                let Some(base) = base_url(&params.source) else {
                    return vec![params.clone()];
                };
                let values = config::parse_children(&document, children, &base);
                if values.is_empty() {
                    return vec![params.clone()];
                }
                values
                    .into_iter()
                    .map(|(source, name, image)| NetworkStatic {
                        id: uuid::Uuid::new_v4().to_string(),
                        name,
                        img: if image.is_empty() {
                            params.img.clone()
                        } else {
                            image
                        },
                        author: params.author.clone(),
                        category: params.category.clone(),
                        headers: params.headers.clone(),
                        extra: params.extra.clone(),
                        source,
                        func: params.func.clone(),
                    })
                    .collect()
            })
        })
    }
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
