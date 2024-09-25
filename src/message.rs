use bevy::math::Vec3;
use bevy::prelude::Event;
use gyra_proto::{network as proto, smp};

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
        message: String,
    },

    NewChunk {
        chunk: smp::ChunkColumn,
    },

    PlayerPositionAndLook {
        position: Vec3,
        yaw: f32,
        pitch: f32,
    },
}

#[derive(Event)]
pub enum ClientMessage {
    ChatMessage {
        message: String,
    },

    Moved {
        x: f64,
        feet_y: f64,
        z: f64,
        on_ground: bool
    },

    Look {
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    },
}
