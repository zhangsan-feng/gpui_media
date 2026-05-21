
use gpui::{ Context, Entity, EventEmitter, Global,};

use reqwest_client::runtime;
use crate::entity;
use crate::entity::{NetworkStatic};

#[derive(Clone)]
pub struct State {
    pub tokio_handle:tokio::runtime::Handle,

}


pub enum StateEvent {
    TogglePlayMusic(entity::NetworkStatic),
    UpdateMusicPlatyList(Vec<entity::NetworkStatic>),
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

        State{
            tokio_handle: tokio_runtime_handle,
        }
    }

}
