mod components;
mod error;
mod message;
mod net;
mod plugin;
mod resources;
mod state;

use crate::plugin::{CursorPlugin, NetworkPlugin, PlayPlugin};
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::window::PresentMode;
use bevy::{
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
use std::env::args;

const SKY_COLOR: Color = Color::srgb(0.69, 0.69, 0.69);

/*
args:
    -dx12: use dx12 backend
    -vulkan: use vulkan backend
    -metal: use metal backend
    -gl: use opengl backend

-auto: use the first primary backend that is available
*/
fn get_backend_from_env() -> Backends {
    let args: Vec<String> = args().collect();
    let mut backend = Backends::empty();

    for arg in args {
        match arg.as_str() {
            "-dx12" => backend |= Backends::DX12,
            "-vulkan" => backend |= Backends::VULKAN,
            "-metal" => backend |= Backends::METAL,
            "-gl" => backend |= Backends::GL,
            "-auto" => {
                backend |= Backends::DX12;
                backend |= Backends::VULKAN;
                backend |= Backends::METAL;
                backend |= Backends::GL;
            }
            _ => {}
        }
    }

    if backend.is_empty() {
        #[cfg(windows)]
        {
            backend |= Backends::DX12;
        }

        #[cfg(not(windows))]
        {
            backend |= Backends::VULKAN | Backends::METAL | Backends::GL;
        }
    }

    backend
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: WgpuSettings {
                        backends: Some(get_backend_from_env()),
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
                        present_mode: PresentMode::Immediate,
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
