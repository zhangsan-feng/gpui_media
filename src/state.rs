use crate::drive;
use crate::drive::NetworkStatic;
use gpui::{Context, Entity, EntityId, EventEmitter, Global, WindowId};

#[derive(Clone)]
pub struct State {}

pub enum StateEvent {
    TogglePlayMusic(drive::NetworkStatic),
    UpdateMusicPlatyList(Vec<drive::NetworkStatic>),
    TogglePlayVideo(WindowId, EntityId, NetworkStatic),
    UpdateVideoPlayList(WindowId, EntityId, Vec<NetworkStatic>),
}

impl EventEmitter<StateEvent> for State {}
pub struct GlobalState(pub(crate) Entity<State>);
impl Global for GlobalState {}

impl State {
    pub fn new(_: &mut Context<Self>) -> Self {
        State {}
    }
}
