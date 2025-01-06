use glam::{Vec3A, vec3a};
use gyra_net::proto::Protocol;
use gyra_net::proto::pallete_delta::PalleteChunk;
use gyra_net::proto::play::ServerKeepAlive;
use gyra_net::transport::Transport;
use gyra_net::{codec::packet::When, proto::play::ClientKeepAlive};
use log::{info, warn};

use crate::player::Player;
use crate::world::World;

pub enum Action {
    SyncPlayerPosition { pos: Vec3A, yaw: f32, pitch: f32 },

    Noop,
}

pub enum GameState {
    InGame { world: World },
    LoggingIn,
}

pub struct NetworkGame {
    pub transport: Transport,
    pub state: GameState,
    pub player: Player,
}

impl NetworkGame {
    pub fn connect(addr: impl ToString) -> std::io::Result<Self> {
        Ok(NetworkGame {
            transport: Transport::connect(addr)?,
            state: GameState::LoggingIn,
            player: Player {
                pitch: 0.0,
                yaw: 0.0,
                position: Default::default(),
            },
        })
    }

    pub fn transport_state(&self) -> When {
        self.transport.state
    }

    fn init_game(&mut self) {
        let world = World::new();
        self.state = GameState::InGame { world };
    }

    pub fn poll_play(&mut self) -> crate::Result<Action> {
        let packet = self.transport.poll_packet()?;
        match packet {
            Protocol::JoinGame(packet) => {
                info!("Joined game: {:?}", packet);
                self.init_game();

                Ok(Action::Noop)
            }

            Protocol::ClientPluginMessage(packet) => {
                info!("Received plugin message: {:?}", packet);
                Ok(Action::Noop)
            }

            Protocol::PlayerPositionAndLook(packet) => {
                info!("Received player position and look packet: {:?}", packet);
                self.player.position = vec3a(packet.x as _, packet.y as _, packet.z as _);
                self.player.pitch = packet.pitch;
                self.player.yaw = packet.yaw;

                Ok(Action::SyncPlayerPosition {
                    pos: self.player.position,
                    yaw: self.player.yaw,
                    pitch: self.player.pitch,
                })
            }

            Protocol::ChunkData(packet) => {
                match self.state {
                    GameState::InGame { ref mut world } => {
                        info!(
                            "Importing chunk of ({}, {})",
                            packet.chunk_x, packet.chunk_z
                        );
                        world.import_from_network(packet)?;
                        info!("Total chunks loaded until now: {}", world.chunks.len());
                    }
                    _ => {
                        warn!("Received chunk data packet while not in game state");
                    }
                }
                Ok(Action::Noop)
            }

            Protocol::ServerKeepAlive(packet) => {
                info!("Received keep alive packet: {:?}", packet);
                let protocol = Protocol::ClientKeepAlive(ClientKeepAlive { id: packet.id });

                protocol.put(
                    &mut self.transport.stream,
                    self.transport.server_compress_threshold,
                )?;
                Ok(Action::Noop)
            }

            _ => {
                warn!("Received unexpected packet: {:?}", packet);
                Ok(Action::Noop)
            }
        }
    }

    pub fn poll_login(&mut self) -> gyra_net::Result<()> {
        let packet = self.transport.poll_packet()?;
        match packet {
            Protocol::LoginSuccess(packet) => {
                info!("Login successful :D {:?}", packet);
                self.transport.state = When::Play;
                self.transport.stream.set_nonblocking(true)?;
                Ok(())
            }

            Protocol::SetCompression(packet) => {
                info!(
                    "Server is compressing packets, threshould = {}",
                    packet.threshold.0
                );

                self.transport.server_compress_threshold = Some(packet.threshold.into());
                Ok(())
            }

            _ => {
                info!("Received unexpected packet: {:?}", packet);
                Err(gyra_net::error::Error::UnexpectedPacket)
            }
        }
    }
}
