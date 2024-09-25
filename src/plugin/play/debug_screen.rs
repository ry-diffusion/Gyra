use bevy::prelude::*;
use crate::plugin::play::player;
use crate::state::AppState;

#[derive(Resource, Debug)]
pub struct DebugScreenActive;

#[derive(Component)]
struct LeftMenu;

#[derive(Component)]
struct PositionText;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn)
        .add_systems(Update, debug_screen_handler)
        .add_systems(
            Update,
            update_info.run_if(resource_exists::<DebugScreenActive>).run_if(in_state(AppState::Playing)),
        );
}

fn update_info(
    mut position_text: Query<&mut Text, With<PositionText>>,
    player_transform: Query<&Transform, With<player::Player>>,
) {
    let mut position_text = position_text.single_mut();
    let transform = player_transform.single(); 
    let pos = transform.translation;
    position_text.sections[0].value = format!("Position: {pos}");
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

                ..Default::default()
            });

            p.spawn(TextBundle {
                text: Text::from_section(
                    "Position: not playing.",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),

                ..Default::default()
            }).insert(PositionText);
        }).insert(LeftMenu);
    
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
