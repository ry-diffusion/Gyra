use crate::message::{ClientMessage, ServerMessage};
use crate::plugin::play::chat::ChatComponent;
use crate::plugin::transport::NetworkTransport;
use crate::plugin::CursorState;
use crate::resources::DisconnectedReason;
use crate::state::AppState;
use bevy::prelude::*;

mod block_builder;
mod chat;
mod chat_proto;
mod chunk_builder;
mod chunk_cons;
mod debug_screen;
mod player;
mod world;

pub struct PlayPlugin;

impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), startup)
            .add_systems(
                FixedUpdate,
                (
                    handle_server_messages.run_if(in_state(AppState::Playing)),
                    chat_message_sender.run_if(in_state(AppState::Playing)),
                ),
            )
            .add_plugins(world::plugin)
            .add_plugins(chat::plugin)
            .add_plugins(player::plugin)
            .add_plugins(debug_screen::plugin)
            .add_plugins(chunk_builder::plugin)
            .add_systems(OnExit(AppState::Playing), cleanup);
    }
}

pub fn startup(mut cursor_state: ResMut<CursorState>) {
    info!("Play!");
    cursor_state.is_locked = true;
}

pub fn chat_message_sender(
    mut chat_reader: EventReader<chat::ChatMessage>,
    mut client_message_writer: EventWriter<ClientMessage>,
) {
    for message in chat_reader.read() {
        client_message_writer.send(ClientMessage::ChatMessage {
            message: message.message.clone(),
        });
    }
}

fn handle_server_messages(
    mut server_reader: EventReader<ServerMessage>,
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
    mut chat_writer: EventWriter<chat::NewRawChatMessage>,
    mut chunk_writer: EventWriter<chunk_builder::ChunkReceived>,
    mut player_transform: Query<&mut Transform, With<player::Player>>,
    mut last_location: Local<Option<Vec3>>,
) {
    for message in server_reader.read() {
        match message {
            ServerMessage::DisconnectedOnLogin { why: _ } => {}

            ServerMessage::PlayerPositionAndLook { position, .. } => {
                let mut transform = player_transform.single_mut();

                if let Some(last_location) = last_location.as_ref() {
                    let distance = position.distance(*last_location);
                    if distance >= 5.0 {
                        transform.translation = position.to_owned();
                    }
                } else {
                    transform.translation = position.to_owned();
                }

                *last_location = Some(position.to_owned());
            }

            ServerMessage::GameReady { base } => {
                info!("Game is ready!");
            }

            ServerMessage::Disconnected { why } => {
                commands.insert_resource(DisconnectedReason { why: why.clone() });
                commands.remove_resource::<NetworkTransport>();
                app_state.set(AppState::Lobby);
            }

            ServerMessage::NewChunk { chunk } => {
                chunk_writer.send(chunk_builder::ChunkReceived {
                    smp_chunk: chunk.clone(),
                });
            }

            ServerMessage::ChatMessage { message } => {
                info!("Chat message: {}", message);
                chat_writer.send(chat::NewRawChatMessage {
                    raw_object: message.clone(),
                });
            }
        }
    }
}

pub fn cleanup(mut commands: Commands, chat: Query<Entity, With<ChatComponent>>) {
    for e in chat.iter() {
        commands.entity(e).despawn_recursive();
    }
}
