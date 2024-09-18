use bevy::prelude::*;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Lobby,

    Connecting,
    
    Playing,
}
