mod components;
mod error;
mod net;
mod plugin;
mod proto;
mod resources;
mod state;
mod message;

use bevy::{
    prelude::*,
    render::{
        settings::{Backends, InstanceFlags, WgpuSettings},
        RenderPlugin,
    },
};

use components::MainCamera;
use plugin::{ConnectingPlugin, LobbyPlugin, SettingsPlugin};
use state::AppState;
use crate::plugin::{NetworkPlugin, PlayPlugin};

const SKY_COLOR: Color = Color::srgb(0.69, 0.69, 0.69);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(RenderPlugin {
                render_creation: WgpuSettings {
                    backends: Some(Backends::VULKAN),
                    instance_flags: InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
                    ..default()
                }
                .into(),
                ..default()
            }),
        )
        .init_state::<AppState>()
        .add_plugins(SettingsPlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins(LobbyPlugin)
        .add_plugins(ConnectingPlugin)
        .add_plugins(PlayPlugin)
        .add_plugins(bevy_cosmic_edit::CosmicEditPlugin::default())
        .add_systems(
            Update,
            (
                bevy_cosmic_edit::change_active_editor_ui,
                bevy_cosmic_edit::deselect_editor_on_esc,
            ),
        )
        .insert_resource(ClearColor(SKY_COLOR))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    log::info!("Welcome to Gyra!");
    commands
        .spawn(Camera3dBundle { ..default() })
        .insert(MainCamera);
}

/// set up a simple 3D scene
fn _setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgb_u8(124, 144, 255)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
