use crate::message::{ClientMessage, ServerMessage};
use crate::plugin::play::chat::ChatComponent;
use crate::plugin::transport::NetworkTransport;
use crate::proto::Proto;
use crate::resources::DisconnectedReason;
use crate::state::AppState;
use bevy::prelude::*;

mod chat;
mod chat_proto;

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
            .add_plugins(chat::chat_plugin)
            .add_systems(OnExit(AppState::Playing), cleanup);
    }
}

pub fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Play!");
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
) {
    for message in server_reader.read() {
        match message {
            ServerMessage::DisconnectedOnLogin { why: _ } => {}

            ServerMessage::GameReady { base } => {
                info!("Game is ready!");
            }

            ServerMessage::Disconnected { why } => {
                commands.insert_resource(DisconnectedReason { why: why.clone() });
                commands.remove_resource::<NetworkTransport>();
                app_state.set(AppState::Lobby);
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
