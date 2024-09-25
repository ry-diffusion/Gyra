use bevy::{
    prelude::*,
    tasks::{block_on, poll_once, IoTaskPool, Task},
    utils::HashMap,
};
use bevy_cosmic_edit::{
    Attrs, AttrsOwned, BufferExtras, ColorExtras, CosmicBuffer, CosmicEditBundle, CosmicEditor,
    CosmicFontSystem, CosmicSource, CosmicTextChanged, DefaultAttrs, Edit, FocusedWidget, InputSet,
    MaxChars, MaxLines, Metrics,
};
use serde_json::Value;

use crate::resources::DisconnectedReason;
use crate::{
    net::query::QueryClient,
    resources::{CurrentServerAddress, PlayerAccount},
    state::AppState,
};

#[derive(Component)]
struct LobbyText;

#[derive(Component)]
struct LobbyUI;

#[derive(Component)]
struct ServerText;

#[derive(Component)]
struct NicknameText;

#[derive(Component)]
struct JoinButton;

#[derive(Resource, Default)]
struct ServerInfoFetcher {
    // IP -> ServerInfo
    fetching_info: HashMap<String, Task<crate::error::Result<ServerInfo>>>,
}

#[derive(Resource, Debug)]
enum ServerInfo {
    Disconnected,
    Found {
        description: String,
        players: u32,
        max_players: u32,
        latency: u64,
    },
}

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Lobby), lobby_startup)
            .add_systems(OnExit(AppState::Lobby), lobby_cleanup)
            .add_systems(
                Update,
                (
                    update_ids.run_if(in_state(AppState::Lobby)).after(InputSet),
                    update_server_info.run_if(in_state(AppState::Lobby)),
                    server_info_updater.run_if(in_state(AppState::Lobby)),
                    server_info_gather.run_if(in_state(AppState::Lobby)),
                    update_server_info.run_if(in_state(AppState::Lobby)),
                    join_handler.run_if(in_state(AppState::Lobby)),
                ),
            );
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);

pub fn lobby_startup(
    mut commands: Commands,
    mut font_system: ResMut<CosmicFontSystem>,
    account: Res<PlayerAccount>,
    current_server: Res<CurrentServerAddress>,
    disconnect_reason: Option<Res<DisconnectedReason>>,
) {
    let attrs = Attrs::new().color(bevy::color::palettes::basic::GRAY.to_cosmic());

    let nickname_edit = commands
        .spawn(CosmicEditBundle {
            default_attrs: DefaultAttrs(AttrsOwned::new(attrs)),
            max_lines: MaxLines(1),
            max_chars: MaxChars(16),

            buffer: CosmicBuffer::new(&mut font_system, Metrics::new(20., 20.)).with_rich_text(
                &mut font_system,
                vec![(account.username.as_str(), attrs)],
                attrs,
            ),

            ..Default::default()
        })
        .insert(NicknameText)
        .id();

    let server_edit = commands
        .spawn(CosmicEditBundle {
            default_attrs: DefaultAttrs(AttrsOwned::new(attrs)),

            max_lines: MaxLines(1),

            buffer: CosmicBuffer::new(&mut font_system, Metrics::new(20., 20.)).with_rich_text(
                &mut font_system,
                vec![(current_server.address.as_str(), attrs)],
                attrs,
            ),

            ..Default::default()
        })
        .insert(ServerText)
        .id();

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(30.0),
                row_gap: Val::Px(30.0),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb_u8(151, 74, 12)),
            ..default()
        })
        .with_children(|parent| {
            if let Some(reason) = disconnect_reason {
                parent
                    .spawn(TextBundle {
                        style: Style {
                            align_self: AlignSelf::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        text: Text::from_section(
                            format!("Got disconnected: {}", reason.why),
                            TextStyle {
                                color: Color::Srgba(bevy::color::palettes::tailwind::RED_500),
                                font_size: 24.0,
                                ..Default::default()
                            },
                        ),
                        ..default()
                    });
            }

            parent
                .spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    text: Text::from_section(
                        "Please insert a server and a nickname",
                        TextStyle {
                            font_size: 24.0,
                            ..Default::default()
                        },
                    ),
                    ..default()
                })
                .insert(LobbyText);

            let input_style = Style {
                width: Val::Percent(50.0),
                height: Val::Px(40.0),
                border: UiRect::all(Val::Px(5.0)),
                ..default()
            };

            let input_button = ButtonBundle {
                border_color: BorderColor(Color::BLACK),
                border_radius: BorderRadius::all(Val::Px(5.0)),
                style: input_style.clone(),
                ..default()
            };

            parent
                .spawn(input_button.clone())
                .insert(CosmicSource(nickname_edit));

            parent.spawn(input_button).insert(CosmicSource(server_edit));

            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(120.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "JOIN",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    ));
                })
                .insert(JoinButton);
        })
        .insert(LobbyUI);

    commands.insert_resource(FocusedWidget(Some(server_edit)));
    commands.insert_resource(ServerInfo::Disconnected);
    commands.insert_resource(ServerInfoFetcher::default());
}

fn query_status_of_server(address: &str) -> Result<ServerInfo, crate::error::Error> {
    let client = QueryClient::connect(address)?;
    let status = client.query_status()?;
    let info = serde_json::from_str::<Value>(&status.server_info)?;
    Ok(ServerInfo::Found {
        description: info["description"]
            .as_str()
            .unwrap_or("No description")
            .to_string(),
        players: info["players"]["online"].as_u64().unwrap_or(0) as _,
        max_players: info["players"]["max"].as_u64().unwrap_or(0) as _,
        latency: status.latency,
    })
}

fn server_info_updater(
    current_server: Res<CurrentServerAddress>,
    mut tasks: ResMut<ServerInfoFetcher>,
) {
    if current_server.is_changed() {
        let pool = IoTaskPool::get();

        let address = current_server.address.clone();
        let task = pool.spawn(async move { query_status_of_server(&address) });

        tasks
            .fetching_info
            .insert(current_server.address.clone(), task);
    }
}

fn server_info_gather(
    mut tasks: ResMut<ServerInfoFetcher>,
    current_server: Res<CurrentServerAddress>,
    mut server_info: ResMut<ServerInfo>,
) {
    let current_addr = current_server.address.clone();
    let mut to_drop = Vec::new();
    for (addr, mut task) in tasks.fetching_info.iter_mut() {
        let status = if task.is_finished() {
            trace!("canceling task for {addr}");
            to_drop.push(addr.clone());
            Some(block_on(task))
        } else {
            block_on(poll_once(&mut task))
        };

        match status {
            Some(Ok(info)) => {
                info!("Server info: {info:?}");
                *server_info = info;

                to_drop.push(addr.clone());
            }
            Some(Err(err)) => {
                error!("Error: {err:?}");
                *server_info = ServerInfo::Disconnected;
                to_drop.push(addr.clone());
            }
            None => {
                trace!("Task not finished yet");
            }
        }

        if *addr != current_addr {
            trace!("retain task for {addr}");
            to_drop.push(addr.clone());
        }
    }

    for addr in to_drop {
        tasks.fetching_info.remove(&addr);
    }
}

fn update_ids(
    server_text_query: Query<&CosmicEditor, With<ServerText>>,
    nickname_text_query: Query<&CosmicEditor, With<NicknameText>>,
    mut evr: EventReader<CosmicTextChanged>,
    mut current_server: ResMut<CurrentServerAddress>,
    mut account: ResMut<PlayerAccount>,
) {
    for _ in evr.read() {
        for input in server_text_query.iter() {
            let current_text: String = input.editor.with_buffer(|buf| buf.get_text());
            current_server.address = current_text;
        }

        for input in nickname_text_query.iter() {
            let current_text: String = input.editor.with_buffer(|buf| buf.get_text());
            account.username = current_text;
        }
    }
}

fn update_server_info(
    mut lobby_text_query: Query<&mut Text, With<LobbyText>>,
    server_info: Res<ServerInfo>,
) {
    for mut text in lobby_text_query.iter_mut() {
        match *server_info {
            ServerInfo::Disconnected => {
                text.sections[0].value = "Please insert a valid server and a nickname".to_string();
            }
            ServerInfo::Found {
                ref description,
                players,
                max_players,
                latency,
            } => {
                text.sections[0].value =
                    format!("{description} - {players}/{max_players} - {latency}ms");
            }
        }
    }
}

fn join_handler(
    mut app_state: ResMut<NextState<AppState>>,

    mut interaction_query: Query<
        (&Interaction, &mut BorderColor, &Children),
        (Changed<Interaction>, With<JoinButton>),
    >,
    mut text_query: Query<&mut Text>,
    server_info: Res<ServerInfo>,
) {
    for (interaction, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        let can_press = match *server_info {
            ServerInfo::Found { .. } => true,
            ServerInfo::Disconnected => false,
        };
        match *interaction {
            Interaction::Pressed if can_press => {
                text.sections[0].value = "JOIN".to_string();
                border_color.0 = Color::Srgba(bevy::color::palettes::tailwind::GREEN_200);
                app_state.set(AppState::Connecting);
            }
            Interaction::Hovered if can_press => {
                text.sections[0].value = "join?".to_string();
                border_color.0 = Color::Srgba(bevy::color::palettes::tailwind::CYAN_300);
            }
            Interaction::None => {
                text.sections[0].value = "JOIN".to_string();
                border_color.0 = Color::BLACK;
            }
            _ => {
                text.sections[0].value = "can't join :c".to_string();
                border_color.0 = Color::Srgba(bevy::color::palettes::tailwind::RED_800);
            }
        }
    }
}

fn lobby_cleanup(query: Query<Entity, With<LobbyUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}
