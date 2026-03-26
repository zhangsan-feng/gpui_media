use std::sync::Arc;
use std::time::Duration;
use gpui::Task;
use rodio::{MixerDeviceSink, Player};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct MusicInfo {
    pub music_id: String,
    pub music_platform: String,
    pub music_name: String,
    pub music_author: String,
    pub music_pic: String,
    pub music_source: String,
}



pub trait PlatformInterface: Send + Sync{
    fn download(&self, params: &MusicConvertLayer) ->anyhow::Result<MusicInfo> ;
}


#[derive(Clone)]
pub struct MusicConvertLayer{
    pub music_id:String,
    pub music_name:String,
    pub music_source:String,
    pub music_pic:String,
    pub music_platform:String,
    pub platform: Arc<dyn PlatformInterface>,
}

impl MusicConvertLayer{
    pub fn download(&self) -> anyhow::Result<MusicInfo> {
        self.platform.download(self)
    }
}