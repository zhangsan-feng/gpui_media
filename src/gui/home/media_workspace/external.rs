use super::MediaWorkspace;
use gpui_kit::{Context, EntityId};
use player_core::PlayStatic;

impl MediaWorkspace {
    pub fn _play_core_id(&self) -> EntityId {
        self.play_core.entity_id()
    }

    pub fn _set_play_list(&mut self, play_list: Vec<PlayStatic>, cx: &mut Context<Self>) {
        if self.save_in_progress {
            return;
        }
        let current_id = self
            .current_index
            .and_then(|index| self.play_list.get(index))
            .map(|item| item.id.clone());
        self.play_list = play_list;
        self.play_list_state
            .scroll_to_item(0, gpui_kit::ScrollStrategy::Top);
        log::debug!(
            "[media-editor:playlist] replaced list: count={} row_height=44",
            self.play_list.len()
        );
        self.current_index =
            current_id.and_then(|id| self.play_list.iter().position(|item| item.id == id));
        cx.notify();
    }

    pub fn _append_play_item(&mut self, item: PlayStatic, cx: &mut Context<Self>) {
        if self.save_in_progress {
            return;
        }
        self.play_list.push(item);
        cx.notify();
    }
}
