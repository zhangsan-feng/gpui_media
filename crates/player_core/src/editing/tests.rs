use super::PlayCoreTimeline;
use crate::PlayCoreExportTrim;
use std::time::Duration;

fn seconds(value: u64) -> Duration {
    Duration::from_secs(value)
}
fn range(start: u64, end: u64) -> PlayCoreExportTrim {
    PlayCoreExportTrim {
        start: seconds(start),
        end: seconds(end),
    }
}

#[test]
fn deleting_middle_closes_gap_and_seeks_to_next_source_span() {
    let mut timeline = PlayCoreTimeline::new(seconds(6));
    timeline.delete(range(2, 4));
    assert_eq!(timeline.duration(), seconds(4));
    assert_eq!(timeline.ranges(), &[range(0, 2), range(4, 6)]);
    assert_eq!(timeline.source_position(seconds(2)), Some(seconds(4)));
}

#[test]
fn deleting_across_previous_join_keeps_only_surviving_sides() {
    let mut timeline = PlayCoreTimeline::new(seconds(6));
    timeline.delete(range(2, 4));
    timeline.delete(range(1, 3));
    assert_eq!(timeline.ranges(), &[range(0, 1), range(5, 6)]);
    assert_eq!(timeline.duration(), seconds(2));
}

#[test]
fn selected_output_range_maps_to_multiple_source_ranges() {
    let mut timeline = PlayCoreTimeline::new(seconds(6));
    timeline.delete(range(2, 4));
    assert_eq!(
        timeline.ranges_for(range(1, 3)),
        vec![range(1, 2), range(4, 5)]
    );
    assert_eq!(timeline.edited_position(seconds(3)), seconds(2));
    assert_eq!(timeline.edited_position(seconds(5)), seconds(3));
}

#[test]
fn deleting_all_creates_empty_timeline_without_reinitializing_source() {
    let mut timeline = PlayCoreTimeline::new(seconds(6));
    timeline.delete(range(0, 20));
    assert_eq!(timeline.duration(), Duration::ZERO);
    assert!(timeline.ranges().is_empty());
    assert_eq!(timeline.source_position(Duration::ZERO), None);
}

#[test]
fn empty_and_reversed_deletions_leave_media_intact() {
    let mut timeline = PlayCoreTimeline::new(seconds(6));
    timeline.delete(range(3, 3));
    timeline.delete(range(4, 2));
    assert_eq!(timeline.ranges(), &[range(0, 6)]);
}
