use super::SidePanelState;
use super::VideoPlayer;
use gpui::{Context, EntityId, Window};
use player_core::PlayStatic;
use reqwest::header::HeaderMap;
use std::time::Duration;
use uuid::Uuid;

impl VideoPlayer {
    pub(super) fn toggle_side_panel(&mut self, cx: &mut Context<Self>) {
        let next_state = match self.side_panel_state {
            SidePanelState::Open | SidePanelState::Opening => SidePanelState::Closing,
            SidePanelState::Closed | SidePanelState::Closing => SidePanelState::Opening,
        };
        self.side_panel_state = next_state;
        self.side_panel_animation_id = self.side_panel_animation_id.wrapping_add(1);
        let animation_id = self.side_panel_animation_id;

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;
            this.update(cx, |this, cx| {
                if this.side_panel_animation_id != animation_id {
                    return;
                }

                this.side_panel_state = match next_state {
                    SidePanelState::Opening => SidePanelState::Open,
                    SidePanelState::Closing => SidePanelState::Closed,
                    SidePanelState::Open | SidePanelState::Closed => return,
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    pub fn _play_core_id(&self) -> EntityId {
        self.play_core.entity_id()
    }

    pub fn _set_play_list(&mut self, play_list: Vec<PlayStatic>, cx: &mut Context<Self>) {
        let current_id = self
            .current_index
            .and_then(|index| self.play_list.get(index))
            .map(|item| item.id.clone());
        self.play_list = play_list;
        self.play_list_state.reset(self.play_list.len());
        self.current_index =
            current_id.and_then(|id| self.play_list.iter().position(|item| item.id == id));
        cx.notify();
    }

    pub fn _append_play_item(&mut self, item: PlayStatic, cx: &mut Context<Self>) {
        let index = self.play_list.len();
        self.play_list.push(item);
        self.play_list_state.splice(index..index, 1);
        cx.notify();
    }

    pub(super) fn _append_and_play(&mut self, item: PlayStatic, cx: &mut Context<Self>) {
        self._append_play_item(item, cx);
        self._play_item(self.play_list.len() - 1, cx);
    }

    pub(super) fn _play_url(&mut self, url: String, cx: &mut Context<Self>) {
        let title = url
            .rsplit('/')
            .next()
            .and_then(|part| part.split(['?', '#']).next())
            .filter(|part| !part.trim().is_empty())
            .unwrap_or("网络媒体")
            .to_string();
        self._append_and_play(
            PlayStatic {
                id: Uuid::new_v4().to_string(),
                title,
                url,
                headers: HeaderMap::new(),
            },
            cx,
        );
    }

    pub(super) fn _play_network_url(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let url = self.network_url_input.read(cx).value().trim().to_string();
        if url.is_empty() {
            return;
        }

        self._play_url(url, cx);
        self.network_url_input
            .update(cx, |input, cx| input.set_value("", window, cx));
    }

    pub fn _play_item(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(item) = self.play_list.get(index).cloned() else {
            return;
        };
        self.current_index = Some(index);
        let _ = self
            .play_core
            .update(cx, |player, cx| player._play_source(item, cx));
        cx.notify();
    }

    pub(crate) fn _play_previous(&mut self, cx: &mut Context<Self>) {
        let Some(index) = self.current_index else {
            return;
        };
        if let Some(previous) = index.checked_sub(1) {
            self._play_item(previous, cx);
        }
    }

    pub(crate) fn _play_next(&mut self, cx: &mut Context<Self>) {
        let Some(index) = self.current_index else {
            return;
        };
        if index + 1 < self.play_list.len() {
            self._play_item(index + 1, cx);
        }
    }
}
