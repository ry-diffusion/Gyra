use crate::message::ServerMessage;
use crate::state::AppState;
use bevy::prelude::*;

pub struct PlayPlugin;

#[derive(Component)]
pub struct ChatComponent;

impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), startup)
            .add_systems(Update, handle_server_messages.run_if(in_state(AppState::Playing)))
            .add_systems(OnExit(AppState::Playing), cleanup);
    }
}

pub fn startup(mut commands: Commands) {
    info!("Play!");

    commands
        .spawn(TextBundle {
            text: Text::from_section(
                "Chat",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..Default::default()
                },
            ),
            ..Default::default()
        })
        .insert(ChatComponent);
}

fn handle_server_messages(mut server_reader: EventReader<ServerMessage>,
                          mut text_query: Query<&mut Text, With<ChatComponent>>
) {
    for message in server_reader.read() {
        match message {
            ServerMessage::GameReady { base } => {
                info!("Game is ready!");
            }
            ServerMessage::ChatMessage { message } => {
                info!("Chat message: {}", message);
                for mut text in text_query.iter_mut() {
                    text.sections[0].value = message.clone();
                }
            }
        }
    }
}

pub fn cleanup(mut commands: Commands) {}
