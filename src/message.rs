use bevy::prelude::Event;
use gyra_proto::{network as proto, smp};

#[derive(Event, Debug)]
pub enum ServerMessage {
    GameReady { base: proto::JoinGame },

    Disconnected { why: String },

    DisconnectedOnLogin { why: String },

    ChatMessage { message: String },

    NewChunk { chunk: smp::Chunk },
}

#[derive(Event)]
pub enum ClientMessage {
    ChatMessage {
        message: String,
    },

    Look {
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    },
}
