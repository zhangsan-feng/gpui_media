use crate::drive::{NetworkStatic, NetworkStaticInterface};
use crate::plugins::extractor::config::{self, PlatformConfig};

pub(crate) struct ConfiguredAudioInterface {
    pub(crate) config: PlatformConfig,
}

impl NetworkStaticInterface for ConfiguredAudioInterface {
    fn download(&self, _params: &NetworkStatic) {}

    fn play(&self, params: &NetworkStatic) -> String {
        let Some(detail) = self.config.item_children.detail.as_ref() else {
            return params.source.clone();
        };
        let Some(play) = detail.play.as_ref() else {
            return params.source.clone();
        };
        let url = play
            .base_url
            .as_deref()
            .map(|template| {
                config::resolve_template(&self.config.base_url, template, &params.source)
            })
            .unwrap_or_else(|| params.source.clone());
        let config = self.config.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let extract_type = config::play_extract_type(&config, play);
                match config::fetch_document(&url, &config, extract_type).await {
                    Ok(document) => {
                        config::extract_play_url(&document, play, &url, &config).unwrap_or_default()
                    }
                    Err(error) => {
                        log::error!("audio play URL request failed: url={url}, {error:#}");
                        String::new()
                    }
                }
            })
        })
    }

    fn detail(&self, params: &NetworkStatic) -> Vec<NetworkStatic> {
        vec![params.clone()]
    }
}
