mod play;
mod recommend;
mod search;

use crate::plugins::extractor::{PlatformConfig, ResourceType, config};

pub use recommend::recommend;
pub use search::search;

pub fn default_plugins() -> Vec<PlatformConfig> {
    config::load_default(ResourceType::Audio)
}
