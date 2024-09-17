use bevy::prelude::Event;
use crate::proto;

#[derive(Event)]
pub enum ServerMessage {
    GameReady {
        base: proto::JoinGame,
    },

    ChatMessage {
        message: String
    }
}

#[derive(Event)]
pub enum ClientMessage {
    ChatMessage {
        message: String
    }
}