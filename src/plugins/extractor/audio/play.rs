use crate::drive::{NetworkStatic, NetworkStaticInterface};
use crate::plugins::extractor::config::{
    PlatformConfig, extract_play_url, fetch_document, fill_template,
};

pub(crate) struct ConfiguredAudioInterface {
    pub(crate) config: PlatformConfig,
}

impl NetworkStaticInterface for ConfiguredAudioInterface {
    fn download(&self, _params: &NetworkStatic) {}

    fn play(&self, params: &NetworkStatic) -> String {
        let url = self
            .config
            .play_url
            .as_deref()
            .map(|template| fill_template(template, &params.id))
            .unwrap_or_else(|| params.source.clone());

        let config = self.config.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                match fetch_document(&url, &config).await {
                    Ok(document) => extract_play_url(&document, &config, &url).unwrap_or_default(),
                    Err(error) => {
                        log::error!("audio play URL request failed: url={}, {error:#}", url);
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
