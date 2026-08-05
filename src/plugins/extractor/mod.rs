pub mod audio;
mod config;
mod template;
#[cfg(test)]
mod test;
pub mod video;

pub use config::{ExtractType, FieldConfig, PageConfig, PlatformConfig, ResourceType};
