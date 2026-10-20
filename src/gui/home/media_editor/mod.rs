mod core;
mod external;
mod ui;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{
    AppContext, Bounds, Context, Entity, EntityId, FocusHandle, KeyBinding, Pixels, Subscription,
    UniformListScrollHandle, Window,
};
use player_core::{PlayCore, PlayCoreExportTrim, PlayCoreTimeline};
use std::collections::VecDeque;
use std::time::Duration;

gpui_kit::actions!(media_editor, [TogglePlayback, DeleteSelection]);

#[derive(Default)]
struct ClipSelection {
    source_key: String,
    timeline: Option<PlayCoreTimeline>,
    selection: Option<PlayCoreExportTrim>,
    timeline_bounds: Option<Bounds<Pixels>>,
    error: Option<String>,
    selection_anchor: Option<Duration>,
    selection_dragging: bool,
    selection_resize: Option<TimelineTarget>,
    preview_ranges: VecDeque<PlayCoreExportTrim>,
    active_preview: Option<PlayCoreExportTrim>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TimelineTarget {
    Playhead,
    Selection,
    SelectionStart,
    SelectionEnd,
}

#[derive(Clone)]
struct TimelineDrag {
    owner: EntityId,
    target: TimelineTarget,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SidePanelState {
    Open,
    Closed,
    Opening,
    Closing,
}

#[derive(Clone, Debug)]
enum ExportStatus {
    Working(String),
    Success(String),
    Error(String),
}

pub struct MediaEditor {
    play_core: Entity<PlayCore>,
    play_list: Vec<player_core::PlayStatic>,
    current_index: Option<usize>,
    side_panel_state: SidePanelState,
    side_panel_animation_id: u64,
    play_list_state: UniformListScrollHandle,
    network_url_input: Entity<InputState>,
    _network_url_input_subscription: Subscription,
    export_in_progress: bool,
    export_status: Option<ExportStatus>,
    playlist_delete_error: Option<String>,
    playlist_delete_in_progress: bool,
    clip: ClipSelection,
    focus_handle: FocusHandle,
}

impl MediaEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.bind_keys([
            KeyBinding::new("space", TogglePlayback, Some("MediaEditor && !Input")),
            KeyBinding::new("delete", DeleteSelection, Some("MediaEditor && !Input")),
        ]);
        let play_core = cx.new(|cx| {
            let mut player = PlayCore::new(window, cx);
            player._set_retain_source_at_end(true);
            player
        });
        cx.observe(&play_core, |this, _, cx| {
            this.sync_clip_selection(cx);
            cx.notify();
        })
        .detach();
        let network_url_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("网络媒体链接，回车导入"));
        let network_url_input_subscription = cx.subscribe_in(
            &network_url_input,
            window,
            |this, _input, event: &InputEvent, window, cx| {
                if !matches!(event, InputEvent::PressEnter { .. }) {
                    return;
                }

                this._play_network_url(window, cx);
            },
        );

        Self {
            play_core,
            play_list: Vec::new(),
            current_index: None,
            side_panel_state: SidePanelState::Open,
            side_panel_animation_id: 0,
            play_list_state: UniformListScrollHandle::new(),
            network_url_input,
            _network_url_input_subscription: network_url_input_subscription,
            export_in_progress: false,
            export_status: None,
            playlist_delete_error: None,
            playlist_delete_in_progress: false,
            clip: ClipSelection::default(),
            focus_handle: cx.focus_handle(),
        }
    }
}

impl gpui_kit::Render for MediaEditor {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        self.render_workspace(window, cx)
    }
}
