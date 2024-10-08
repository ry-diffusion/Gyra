mod components;
mod error;
mod message;
mod net;
mod plugin;
mod resources;
mod state;

use crate::plugin::{CursorPlugin, NetworkPlugin, PlayPlugin};
use bevy::core::TaskPoolThreadAssignmentPolicy;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::view::RenderLayers;
use bevy::window::PresentMode;
use bevy::{
    log,
    prelude::*,
    render::{
        settings::{Backends, InstanceFlags, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_cosmic_edit::CosmicPrimaryCamera;
use components::MainCamera;
use plugin::{ConnectingPlugin, LobbyPlugin, SettingsPlugin};
use state::AppState;

const SKY_COLOR: Color = Color::srgb(0.69, 0.69, 0.69);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: WgpuSettings {
                        backends: Some(Backends::PRIMARY | Backends::SECONDARY),
                        instance_flags: InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        name: Some("Gyra".to_string()),
                        title: "Gyra".to_string(),
                        present_mode: PresentMode::Mailbox,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<PipelinedRenderingPlugin>(),
        )
        .init_state::<AppState>()
        .add_plugins(SettingsPlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins(LobbyPlugin)
        .add_plugins(ConnectingPlugin)
        .add_plugins(PlayPlugin)
        .add_plugins(CursorPlugin)
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
    info!("Welcome to Gyra!");

    commands
        .spawn(Camera2dBundle { ..default() })
        .insert(MainCamera)
        .insert(CosmicPrimaryCamera);
}
