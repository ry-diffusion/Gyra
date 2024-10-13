use bevy::prelude::*;
use gyra_codec::packet::When;

use crate::message::ServerMessage;
use crate::plugin::network::transport::NetworkTransport;
use crate::plugin::ChangedState;
use crate::resources::DisconnectedReason;
use crate::{resources::CurrentServerAddress, state::AppState};

pub struct ConnectingPlugin;

#[derive(Component)]
pub struct ProgressText;

#[derive(Component)]
struct ConnectingUI;

impl Plugin for ConnectingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Connecting), startup)
            .add_systems(Update, update_status.run_if(in_state(AppState::Connecting)))
            .add_systems(
                Update,
                handle_server_messages.run_if(in_state(AppState::Connecting)),
            )
            .add_systems(OnExit(AppState::Connecting), cleanup);
    }
}

fn startup(mut commands: Commands, current_server: Res<CurrentServerAddress>) {
    commands.remove_resource::<NetworkTransport>();
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                column_gap: Val::Px(30.0),
                row_gap: Val::Px(30.0),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb_u8(151, 74, 12)),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                text: Text::from_section(
                    "Get ready! We're connecting to the server...",
                    TextStyle {
                        font_size: 24.0,
                        ..Default::default()
                    },
                ),
                ..default()
            });

            parent
                .spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    text: Text::from_section(
                        "starting connection..",
                        TextStyle {
                            font_size: 16.0,
                            ..Default::default()
                        },
                    ),
                    ..default()
                })
                .insert(ProgressText);
        })
        .insert(ConnectingUI);

    let world = NetworkTransport::connect(current_server.address.clone()).unwrap();
    commands.insert_resource(world);
}

fn update_status(
    mut query: Query<&mut Text, With<ProgressText>>,
    mut receiver: EventReader<ChangedState>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for state in receiver.read() {
        if state.to == When::Play {
            app_state.set(AppState::Playing);
            return;
        }

        let mut text = query.iter_mut().next().unwrap();
        text.sections[0].value = state.to.to_string()
    }
}

fn cleanup(query: Query<Entity, With<ConnectingUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn handle_server_messages(
    mut server_reader: EventReader<ServerMessage>,
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for message in server_reader.read() {
        match message {
            ServerMessage::DisconnectedOnLogin { why } => {
                commands.insert_resource(DisconnectedReason { why: why.clone() });
                commands.remove_resource::<NetworkTransport>();
                app_state.set(AppState::Lobby);
            }

            msg => {
                warn!("Unexpected message: {msg:?}");
            }
        }
    }
}
