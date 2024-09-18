use bevy::prelude::Event;
use crate::proto;

#[derive(Event, Debug)]
pub enum ServerMessage {
    GameReady {
        base: proto::JoinGame,
    },
    
    Disconnected {
        why: String,
    },
    
    DisconnectedOnLogin {
        why: String,
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