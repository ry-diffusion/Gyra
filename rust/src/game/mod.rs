use crate::math::ChunkPos;
use crate::player::Player;
use crate::world::World;
use crate::world::chunk::index_to_coords;
use glam::{Vec3A, vec3a};
use godot::builtin::Vector3;
use godot::classes::{BoxMesh, CsgBox3D, MeshInstance3D};
use godot::obj::{Gd, NewAlloc, NewGd};
use gyra_net::proto::Protocol;
use gyra_net::transport::Transport;
use gyra_net::{codec::packet::When, proto::play::ClientKeepAlive};
use log::{info, warn};
use std::collections::{HashMap, HashSet};

pub enum Action {
    SyncPlayerPosition {
        pos: Vec3A,
        yaw: f32,
        pitch: f32,
    },

    ShowChunks {
        chunks: HashMap<ChunkPos, Vec<Gd<MeshInstance3D>>>,
    },

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
    pub built_chunks: HashSet<ChunkPos>,
}

impl NetworkGame {
    pub fn connect(addr: impl ToString) -> std::io::Result<Self> {
        Ok(NetworkGame {
            transport: Transport::connect(addr)?,
            state: GameState::LoggingIn,
            built_chunks: HashSet::new(),
            player: Player {
                pitch: 0.0,
                yaw: 0.0,
                position: Default::default(),
            },
        })
    }

    pub fn build_cubes(&mut self) -> HashMap<ChunkPos, Vec<Gd<MeshInstance3D>>> {
        let world = match &self.state {
            GameState::InGame { world } => world,
            _ => return HashMap::new(),
        };

        
        let mut cubes = HashMap::new();

        for (pos, chunk) in &world.chunks {
            if self.built_chunks.contains(pos) {
                continue;
            }
            
            self.built_chunks.insert(*pos);
            let cubes_entry = cubes.entry(*pos).or_insert_with(|| Vec::new());

            for (section_idx, section) in chunk.sections.iter().enumerate() {
                for (idx, block) in section.blocks.iter().enumerate() {
                    if block.is_air() {
                        continue;
                    }
                    let block_idx = index_to_coords(idx);
                    let world_pos = pos.to_world_pos();

                    let block_pos = Vector3::new(
                        world_pos.x as f32 + block_idx.x as f32,
                        section_idx as f32 * 16.0 + section_idx as f32,
                        world_pos.z as f32 + block_idx.z as f32,
                    );

                    let mut cube_mesh = BoxMesh::new_gd();
                    let mut mesh_instance = MeshInstance3D::new_alloc();
                    cube_mesh.set_size(Vector3::new(1.0, 1.0, 1.0));
                    mesh_instance.set_mesh(&cube_mesh);
                    
                    mesh_instance.set_position(block_pos);
                    cubes_entry.push(mesh_instance);
                }
            }
        }

        cubes
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
                Ok(Action::ShowChunks {
                    chunks: self.build_cubes(),
                })
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
