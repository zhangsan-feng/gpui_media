mod core;
mod external;
mod ui;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{
    AppContext, Bounds, Context, Entity, EntityId, FocusHandle, KeyBinding, Pixels, Subscription,
    UniformListScrollHandle, Window,
};
use player_core::{
    PlayCore, PlayCoreExportTrim, PlayCoreGlobalState, PlayCoreStateEvent, PlayCoreTimeline,
};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const MODE_TRANSITION_DURATION: Duration = Duration::from_millis(500);

gpui_kit::actions!(media_workspace, [TogglePlayback, DeleteSelection]);

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
    preview_auto_next: bool,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkspaceMode {
    Playback,
    Editing,
}

pub struct MediaWorkspace {
    mode: WorkspaceMode,
    auto_play_next: bool,
    mode_animation_id: u64,
    mode_animation_from: f32,
    mode_animation_started: Option<Instant>,
    play_core: Entity<PlayCore>,
    play_list: Vec<player_core::PlayStatic>,
    current_index: Option<usize>,
    side_panel_state: SidePanelState,
    side_panel_animation_id: u64,
    play_list_state: UniformListScrollHandle,
    network_url_input: Entity<InputState>,
    _network_url_input_subscription: Subscription,
    export_in_progress: bool,
    save_in_progress: bool,
    export_status: Option<ExportStatus>,
    playlist_delete_error: Option<String>,
    playlist_delete_in_progress: bool,
    clip: ClipSelection,
    focus_handle: FocusHandle,
}

impl MediaWorkspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.bind_keys([
            KeyBinding::new("space", TogglePlayback, Some("MediaWorkspace && !Input")),
            KeyBinding::new("delete", DeleteSelection, Some("MediaWorkspace && !Input")),
        ]);
        let play_core = cx.new(|cx| {
            let mut player = PlayCore::new(window, cx);
            player._set_retain_source_at_end(false);
            player
        });
        cx.observe(&play_core, |this, _, cx| {
            this.sync_clip_selection(cx);
            cx.notify();
        })
        .detach();
        Self::start_subscribe(&play_core, window, cx);
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

        log::info!(
            "[media-workspace] initialized shared player and editor; mode=Playback auto_play_next=true"
        );
        Self {
            mode: WorkspaceMode::Playback,
            auto_play_next: true,
            mode_animation_id: 0,
            mode_animation_from: 0.0,
            mode_animation_started: None,
            play_core,
            play_list: Vec::new(),
            current_index: None,
            side_panel_state: SidePanelState::Open,
            side_panel_animation_id: 0,
            play_list_state: UniformListScrollHandle::new(),
            network_url_input,
            _network_url_input_subscription: network_url_input_subscription,
            export_in_progress: false,
            save_in_progress: false,
            export_status: None,
            playlist_delete_error: None,
            playlist_delete_in_progress: false,
            clip: ClipSelection::default(),
            focus_handle: cx.focus_handle(),
        }
    }
    fn start_subscribe(play_core: &Entity<PlayCore>, window: &Window, cx: &mut Context<Self>) {
        let player_id = play_core.entity_id();
        let window_id = window.window_handle().window_id();
        PlayCoreGlobalState::subscribe(cx, move |this, event, cx| {
            let PlayCoreStateEvent::PlayBackFished(event_window_id, event_player_id, finished) =
                event
            else {
                return;
            };
            if *event_player_id == player_id && event_window_id.as_u64() == window_id.as_u64() {
                this.play_next_after_finished(finished, cx);
            }
        })
        .detach();
    }
}

impl gpui_kit::Render for MediaWorkspace {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        self.render_workspace(window, cx)
    }
}
