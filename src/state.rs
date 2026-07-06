use gpui::{Context, Entity, EventEmitter, Global};

use crate::drive;
use crate::drive::NetworkStatic;
use reqwest_client::runtime;

#[derive(Clone)]
pub struct State {

}

pub enum StateEvent {
    TogglePlayMusic(drive::NetworkStatic),
    UpdateMusicPlatyList(Vec<drive::NetworkStatic>),
    TogglePlayVideo(NetworkStatic),
    UpdateVideoPlayList(Vec<NetworkStatic>),
}

impl EventEmitter<StateEvent> for State {}
pub struct GlobalState(pub(crate) Entity<State>);
impl Global for GlobalState {}

impl State {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let tokio_runtime_handle = tokio::runtime::Handle::try_current().unwrap_or_else(|_| {
            log::debug!("no tokio runtime found");
            runtime().handle().clone()
        });

        State {
            
        }
    }
}
