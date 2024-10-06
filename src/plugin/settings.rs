use bevy::{
    prelude::*,
    window::WindowCloseRequested,
};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    env::var,
    fs::{exists, read_to_string, write},
    path::PathBuf,
};

use crate::resources::{CurrentServerAddress, GamePaths, PlayerAccount};

pub struct SettingsPlugin;

#[derive(Debug, Deserialize, Serialize)]
struct SettingsProto {
    pub server_address: String,
    pub username: String,
}

fn guess_root() -> PathBuf {
    let root = var("GYRA_ROOT");
    if let Ok(root) = root {
        return PathBuf::from(root);
    }

    ProjectDirs::from("io.github", "ry-diffusion", "Gyra")
        .expect("Could not find a valid project directory. Set GYRA_ROOT to override.")
        .data_dir()
        .to_path_buf()
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let root = guess_root();
        app.insert_resource(CurrentServerAddress {
            address: "127.0.0.1".to_string(),
        })
        .insert_resource(GamePaths {
            root: root.clone(),
            settings_path: root.join("settings.toml"),
        })
        .insert_resource(PlayerAccount {
            username: "GyraPlayer".to_string(),
        })
        .add_systems(PreStartup, startup)
        .add_systems(PreUpdate, shutdown);
    }
}

fn parse_settings(path: PathBuf) -> crate::error::Result<SettingsProto> {
    let data = read_to_string(path)?;
    toml::from_str(&data).map_err(Into::into)
}

fn store_settings(path: PathBuf, settings: SettingsProto) -> crate::error::Result<()> {
    write(path, toml::to_string(&settings)?)?;
    Ok(())
}

fn startup(
    paths: Res<GamePaths>,
    mut current_server: ResMut<CurrentServerAddress>,
    mut account: ResMut<PlayerAccount>,
) {
    let GamePaths {
        root,
        settings_path,
    } = &*paths;

    log::info!("Game root is: {root:?}");
    if !exists(root).unwrap_or(false) {
        log::info!("Creating game root directory.");
        if let Err(e) = std::fs::create_dir_all(root) {
            log::error!("Could not create game root directory: {e:?}");
            log::warn!("Initial steps failed, game may not work correctly.");
        }
    }

    log::info!("Loading settings from: {settings_path:?}");

    match parse_settings(paths.settings_path.clone()) {
        Ok(settings) => {
            current_server.address = settings.server_address;
            account.username = settings.username;
        }
        Err(e) => {
            log::error!("Could not read settings: {e:?}");
            log::info!("Using default settings.");
        }
    }
}

use bevy::log;

fn shutdown(
    mut exits: EventReader<AppExit>,
    paths: Res<GamePaths>,
    current_server: Res<CurrentServerAddress>,
    account: Res<PlayerAccount>,
    mut closed_events: EventReader<WindowCloseRequested>,
) {
    let should_save = closed_events.read().count() > 0 || exits.read().count() > 0;
    if should_save {
        let proto = SettingsProto {
            server_address: current_server.address.clone(),
            username: account.username.clone(),
        };

        if let Err(e) = store_settings(paths.settings_path.clone(), proto) {
            log::error!("Could not save settings: {e:?}");
        }

        log::info!("Stored settings. Shutting down!")
    }
}
