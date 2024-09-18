use std::path::PathBuf;

use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct DisconnectedReason {
    pub why: String,
}


#[derive(Resource, Debug)]
pub struct GamePaths {
    pub root: PathBuf,
    pub settings_path: PathBuf,
}

#[derive(Resource, Debug)]
pub struct CurrentServerAddress {
    pub address: String,
}

#[derive(Resource, Debug)]
pub struct PlayerAccount {
    pub username: String,
}
