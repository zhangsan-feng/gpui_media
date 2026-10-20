use super::PlayCoreTimeline;
use crate::PlayCoreExportTrim;
use std::time::Duration;

impl PlayCoreTimeline {
    pub(super) fn remove_range(&mut self, selection: PlayCoreExportTrim) {
        if selection.start >= selection.end {
            return;
        }
        let mut offset = Duration::ZERO;
        let mut remaining = Vec::new();
        for span in &self.spans {
            let next = offset + (span.end - span.start);
            let start = selection.start.max(offset);
            let end = selection.end.min(next);
            if start >= end {
                remaining.push(*span);
            } else {
                if start > offset {
                    remaining.push(PlayCoreExportTrim {
                        start: span.start,
                        end: span.start + (start - offset),
                    });
                }
                if end < next {
                    remaining.push(PlayCoreExportTrim {
                        start: span.start + (end - offset),
                        end: span.end,
                    });
                }
            }
            offset = next;
        }
        self.spans = remaining;
    }

    pub(super) fn source_ranges(&self, selection: PlayCoreExportTrim) -> Vec<PlayCoreExportTrim> {
        if selection.start >= selection.end {
            return Vec::new();
        }
        let mut offset = Duration::ZERO;
        let mut ranges = Vec::new();
        for span in &self.spans {
            let next = offset + (span.end - span.start);
            let start = selection.start.max(offset);
            let end = selection.end.min(next);
            if start < end {
                ranges.push(PlayCoreExportTrim {
                    start: span.start + (start - offset),
                    end: span.start + (end - offset),
                });
            }
            offset = next;
        }
        ranges
    }

    pub(super) fn map_to_source(&self, position: Duration) -> Option<Duration> {
        let mut offset = Duration::ZERO;
        for span in &self.spans {
            let next = offset + (span.end - span.start);
            if position < next {
                return Some(span.start + position.saturating_sub(offset));
            }
            offset = next;
        }
        self.spans.last().map(|span| span.end)
    }

    pub(super) fn map_to_edited(&self, position: Duration) -> Duration {
        let mut offset = Duration::ZERO;
        for span in &self.spans {
            if position < span.start {
                return offset;
            }
            if position <= span.end {
                return offset + (position - span.start);
            }
            offset += span.end - span.start;
        }
        offset
    }
}
