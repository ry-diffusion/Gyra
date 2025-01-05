use std::net::Shutdown;

use godot::prelude::*;

use crate::{essentials::GdResult, game::NetworkGame};

pub enum GameState {
    MainMenu,
    Logging { username: String, game: NetworkGame },

    Playing { game: NetworkGame },
}

#[derive(GodotClass)]
#[class(base=Object)]
pub struct Gyra {
    base: Base<Object>,

    state: GameState,
    player_name: String,
}

#[godot_api]
impl IObject for Gyra {
    fn init(owner: Base<Object>) -> Self {
        Gyra {
            base: owner,
            state: GameState::MainMenu,
            player_name: String::from("GyraPlayer"),
        }
    }
}

#[godot_api]
impl Gyra {
    #[func]
    pub fn say_hello(&self) {
        godot_print!("Hello from Rust!");
    }

    #[func]
    pub fn set_player_name(&mut self, name: String) {
        self.player_name = name;
    }

    #[func]
    pub fn login(&mut self) -> GdResult {
        let GameState::Logging { game, username, .. } = &mut self.state else {
            return GdResult::err("The game is not in the logging state");
        };

        match game.transport.login(username.to_string()) {
            Ok(_) => GdResult::ok("Login sequence started"),
            Err(e) => GdResult::err(e.to_string()),
        }
    }

    #[func]
    pub fn poll_login(&mut self) -> GdResult {
        let GameState::Logging { game, .. } = &mut self.state else {
            return GdResult::err("The game is not in the logging state");
        };

        match game.poll_login() {
            Ok(_) => GdResult::ok("Log ok"),
            Err(e) => GdResult::err(e.to_string()),
        }
    }

    #[func]
    pub fn query_login_status(&self) -> GdResult {
        match &self.state {
            GameState::Logging { game, .. } => GdResult::ok(dict! {
                "status": game.transport_state().to_string(),
                "switch_scene": false,
            }),
            _ => GdResult::ok(dict! {
                "status": "ready",
                "switch_scene": true,
            }),
        }
    }

    #[func]
    pub fn connect_to(&mut self, address: String) -> GdResult {
        godot_print!("Connecting to server at {}", address);

        let result = NetworkGame::connect(address);
        match result {
            Ok(game) => {
                self.state = GameState::Logging {
                    username: self.player_name.clone(),
                    game,
                };
                GdResult::ok("Client should switch the scene")
            }
            Err(e) => GdResult::err(e.to_string()),
        }
    }

    #[func]
    pub fn reset_connection(&mut self) -> GdResult {
        match &mut self.state {
            GameState::Logging { game, .. } => {
                game.transport.stream.shutdown(Shutdown::Both).ok();
                self.state = GameState::MainMenu;
                GdResult::ok("Connection reset")
            }

            GameState::Playing { game } => {
                game.transport.stream.shutdown(Shutdown::Both).ok();
                self.state = GameState::MainMenu;
                GdResult::ok("Connection reset")
            }

            _ => GdResult::err("The game is not in the logging state"),
        }
    }
}
