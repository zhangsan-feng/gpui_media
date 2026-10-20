mod core;
mod external;
#[cfg(test)]
mod tests;

use crate::PlayCoreExportTrim;

#[derive(Clone, Debug, Default)]
pub struct PlayCoreTimeline {
    spans: Vec<PlayCoreExportTrim>,
}
