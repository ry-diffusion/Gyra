use crate::essentials::error::Error;
use crate::game::Action;
use crate::{
    essentials::GdResult,
    game::{GameState, NetworkGame},
};
use godot::classes::Engine;
use godot::prelude::*;
use gyra_net::codec::packet::When;
use std::collections::VecDeque;
use std::{mem::replace, net::Shutdown};

pub enum GyraState {
    MainMenu,
    Logging { username: String, game: NetworkGame },

    Playing { game: NetworkGame },
}

#[derive(GodotClass)]
#[class(base=Object)]
pub struct Gyra {
    base: Base<Object>,

    state: GyraState,
    player_name: String,
}

#[godot_api]
impl IObject for Gyra {
    fn init(owner: Base<Object>) -> Self {
        Gyra {
            base: owner,
            state: GyraState::MainMenu,
            player_name: String::from("GyraPlayer"),
        }
    }
}

impl Gyra {
    pub fn singleton() -> Gd<Gyra> {
        let engine = Engine::singleton();

        unsafe {
            engine
                .get_singleton("GyraSingleton")
                .unwrap_unchecked()
                .cast()
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
        let GyraState::Logging { game, username, .. } = &mut self.state else {
            return GdResult::err("The game is not in the logging state");
        };

        match game.transport.login(username.to_string()) {
            Ok(_) => GdResult::ok("Login sequence started"),
            Err(e) => GdResult::err(e.to_string()),
        }
    }

    #[func]
    pub fn poll_login(&mut self) -> GdResult {
        let game = match &mut self.state {
            GyraState::Logging { game, .. } => game,
            GyraState::Playing { game, .. } => game,
            _ => return GdResult::err("The game is not in a valid state"),
        };

        if game.transport_state() == When::Play {
            return if let GameState::InGame { .. } = &game.state {
                GdResult::ok("can_play")
            } else {
                match game.poll_play() {
                    Ok(_) => GdResult::ok("gpoll_ok"),
                    Err(e) if e.is_eagain() => GdResult::ok("gpoll_again"),

                    Err(e) => GdResult::err(e.to_string()),
                }
            };
        }

        match game.poll_login() {
            Ok(_) => GdResult::ok("poll_ok"),
            Err(e) => GdResult::err(e.to_string()),
        }
    }

    fn fetch_actions(&mut self) -> crate::Result<VecDeque<Action>> {
        let GyraState::Playing { game } = &mut self.state else {
            return Error::custom("The game is not in the playing state");
        };

        let mut buff = VecDeque::new();

        /* Poll all remaining play */
        while let Ok(action) = game.poll_play() {
            if let Action::Noop = action {
                continue;
            }

            buff.push_back(action);
        }

        Ok(buff)
    }

    fn handle_action(&mut self, action: Action) -> crate::Result<()> {
        match action {
            Action::SyncPlayerPosition { pos, yaw, pitch } => {
                self.base_mut().emit_signal("player_position_sync", &[
                    Vector3::new(pos.x, pos.y, pos.z).to_variant(),
                    yaw.to_variant(),
                    pitch.to_variant(),
                ]);

                Ok(())
            }
            _ => Ok(()),
        }
    }

    #[func]
    pub fn poll_play(&mut self) -> GdResult {
        let actions = match self.fetch_actions() {
            Ok(actions) => actions,
            Err(e) => return GdResult::err(e.to_string()),
        };

        for action in actions {
            if let Err(e) = self.handle_action(action) {
                return GdResult::err(e.to_string());
            }
        }

        GdResult::ok("poll_ok")
    }

    #[func]
    pub fn switch_to_play_state(&mut self) -> GdResult {
        if let GyraState::Logging { game, .. } = replace(&mut self.state, GyraState::MainMenu) {
            self.state = GyraState::Playing { game };
            GdResult::ok("Switched to play state")
        } else {
            GdResult::err("The game is not in the logging state")
        }
    }

    #[func]
    pub fn query_login_status(&self) -> GdResult {
        match &self.state {
            GyraState::Logging { game, .. } => GdResult::ok(dict! {
                "status": game.transport_state().to_string(),
                "switch_scene": false,
            }),
            GyraState::Playing { game } => {
                if let GameState::InGame { .. } = &game.state {
                    GdResult::ok(dict! {
                        "status": "Get Ready!",
                        "switch_scene": true,
                    })
                } else {
                    GdResult::ok(dict! {
                        "status": "Downloading Terrain",
                        "switch_scene": false,
                    })
                }
            }
            _ => GdResult::ok(dict! {
                "status": "unknown",
                "switch_scene": false,
            }),
        }
    }

    #[func]
    pub fn connect_to(&mut self, address: String) -> GdResult {
        godot_print!("Connecting to server at {}", address);

        let result = NetworkGame::connect(address);
        match result {
            Ok(game) => {
                self.state = GyraState::Logging {
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
            GyraState::Logging { game, .. } => {
                game.transport.stream.shutdown(Shutdown::Both).ok();
                self.state = GyraState::MainMenu;
                GdResult::ok("Connection reset")
            }

            GyraState::Playing { game } => {
                game.transport.stream.shutdown(Shutdown::Both).ok();
                self.state = GyraState::MainMenu;
                GdResult::ok("Connection reset")
            }

            _ => GdResult::err("The game is not in the logging state"),
        }
    }

    #[signal]
    pub fn player_position_sync(&self, pos: Vector3, yaw: f32, pitch: f32);
}
