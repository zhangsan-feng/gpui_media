use super::PlayCoreTimeline;
use crate::PlayCoreExportTrim;
use std::time::Duration;

impl PlayCoreTimeline {
    pub fn new(duration: Duration) -> Self {
        Self {
            spans: if duration.is_zero() {
                Vec::new()
            } else {
                vec![PlayCoreExportTrim {
                    start: Duration::ZERO,
                    end: duration,
                }]
            },
        }
    }
    pub fn ranges(&self) -> &[PlayCoreExportTrim] {
        &self.spans
    }
    pub fn duration(&self) -> Duration {
        self.spans.iter().map(|span| span.end - span.start).sum()
    }
    pub fn delete(&mut self, selection: PlayCoreExportTrim) {
        self.remove_range(selection);
    }
    pub fn ranges_for(&self, selection: PlayCoreExportTrim) -> Vec<PlayCoreExportTrim> {
        self.source_ranges(selection)
    }
    pub fn source_position(&self, position: Duration) -> Option<Duration> {
        self.map_to_source(position)
    }
    pub fn edited_position(&self, position: Duration) -> Duration {
        self.map_to_edited(position)
    }
}
