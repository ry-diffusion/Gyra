use crate::plugin::play::player;
use crate::state::AppState;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct DebugScreenActive;

#[derive(Component)]
struct LeftMenu;

#[derive(Component)]
struct PositionText;

#[derive(Component)]
struct FpsText;

pub fn plugin(app: &mut App) {
    app.add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, spawn)
        .add_systems(
            Update,
            (
                debug_screen_handler,
                update_info
                    .run_if(resource_exists::<DebugScreenActive>)
                    .run_if(in_state(AppState::Playing)),
                update_fps.run_if(resource_exists::<DebugScreenActive>),
            ),
        );
}

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut fps_text: Query<&mut Text, With<FpsText>>) {
    if let Some(value) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        let mut fps_text = fps_text.single_mut();
        fps_text.sections[1].value = format!("{value:>4.0}");

        fps_text.sections[1].style.color = if value >= 120.0 {
            // Above 120 FPS, use green color
            Color::srgb(0.0, 1.0, 0.0)
        } else if value >= 60.0 {
            // Between 60-120 FPS, gradually transition from yellow to green
            Color::srgb((1.0 - (value - 60.0) / (120.0 - 60.0)) as f32, 1.0, 0.0)
        } else if value >= 30.0 {
            // Between 30-60 FPS, gradually transition from red to yellow
            Color::srgb(1.0, ((value - 30.0) / (60.0 - 30.0)) as f32, 0.0)
        } else {
            // Below 30 FPS, use red color
            Color::srgb(1.0, 0.0, 0.0)
        }
    }
}

fn update_info(
    mut position_text: Query<&mut Text, With<PositionText>>,
    player_transform: Query<&Transform, With<player::Player>>,
) {
    let mut position_text = position_text.single_mut();

    let transform = player_transform.single();
    let pos = transform.translation;
    position_text.sections[1].value = format!(" {:>2.0} {:>2.0} {:>2.0}", pos.x, pos.y, pos.z);
}

fn spawn(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            style: Style {
                max_width: Val::Percent(30.0),
                max_height: Val::Percent(50.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            p.spawn(TextBundle {
                text: Text::from_section(
                    concat!("Gyra v", env!("CARGO_PKG_VERSION")),
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),

                ..default()
            });

            p.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        "Pos",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::GREEN_200),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            ..default()
                        },
                    ),
                ]),

                ..default()
            })
            .insert(PositionText);

            p.spawn(TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        "FPS",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::from(bevy::color::palettes::tailwind::RED_200),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        " N/A",
                        TextStyle {
                            font_size: 12.0,
                            ..default()
                        },
                    ),
                ]),

                ..default()
            })
            .insert(FpsText);
        })
        .insert(LeftMenu);

    commands.insert_resource(DebugScreenActive);
}

fn debug_screen_handler(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    is_active: Option<Res<DebugScreenActive>>,
    mut container: Query<&mut Visibility, With<LeftMenu>>,
) {
    if keys.pressed(KeyCode::F3) {
        let mut visibility = container.single_mut();
        if is_active.is_some() {
            *visibility = Visibility::Hidden;
            commands.remove_resource::<DebugScreenActive>();
        } else {
            *visibility = Visibility::Visible;
            commands.insert_resource(DebugScreenActive);
        }
    }
}
