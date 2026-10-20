mod core;
mod external;
mod ui;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{
    AppContext, Context, Entity, FocusHandle, KeyBinding, Subscription, UniformListScrollHandle,
    Window,
};
use player_core::{PlayCore, PlayCoreGlobalState, PlayCoreStateEvent, PlayStatic};

gpui_kit::actions!(medio_player, [TogglePlayback]);

#[derive(Clone, Copy, PartialEq, Eq)]
enum SidePanelState {
    Open,
    Closed,
    Opening,
    Closing,
}

pub struct MedioPlayer {
    play_core: Entity<PlayCore>,
    play_list: Vec<PlayStatic>,
    current_index: Option<usize>,
    side_panel_state: SidePanelState,
    side_panel_animation_id: u64,
    play_list_state: UniformListScrollHandle,
    network_url_input: Entity<InputState>,
    _network_url_input_subscription: Subscription,
    playlist_delete_error: Option<String>,
    playlist_delete_in_progress: bool,
    focus_handle: FocusHandle,
}

impl MedioPlayer {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.bind_keys([KeyBinding::new(
            "space",
            TogglePlayback,
            Some("MedioPlayer && !Input"),
        )]);

        let play_core = cx.new(|cx| PlayCore::new(window, cx));
        cx.observe(&play_core, |_, _, cx| cx.notify()).detach();
        Self::start_subscribe(&play_core, window, cx);

        let network_url_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("输入网络视频或音频链接，按回车播放")
        });
        let network_url_input_subscription = cx.subscribe_in(
            &network_url_input,
            window,
            |this, _input, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.play_network_url(window, cx);
                }
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
            playlist_delete_error: None,
            playlist_delete_in_progress: false,
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
            if *event_player_id != player_id || event_window_id.as_u64() != window_id.as_u64() {
                return;
            }
            this.play_next_after_finished(finished, cx);
        })
        .detach();
    }
}

impl gpui_kit::Render for MedioPlayer {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        self.render_workspace(window, cx)
    }
}
