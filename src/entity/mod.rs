use std::sync::Arc;


#[derive(Clone)]
pub struct MusicConvertLayer{
    pub music_id:String,
    pub music_name:String,
    pub music_author:String,
    pub music_pic:String,
    pub music_platform:String,
    pub music_time:String,
    pub music_source:String,
    pub music_file:String,
    pub func: Arc<dyn PlatformInterface>,
}

pub trait PlatformInterface: Send + Sync{
    fn download(&self, params: &MusicConvertLayer) ->anyhow::Result<MusicConvertLayer>;
}

impl MusicConvertLayer{
    pub fn download(&self) -> anyhow::Result<MusicConvertLayer> {
        self.func.download(self)
    }
}

pub struct DefaultPlatformInterface;
impl PlatformInterface for DefaultPlatformInterface{
    fn download(&self, params: &MusicConvertLayer) ->anyhow::Result<MusicConvertLayer> {
        anyhow::bail!("No platform implemented yet")
    }
}