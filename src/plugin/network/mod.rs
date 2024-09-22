use crate::error::Error;
use crate::message::{ClientMessage, ServerMessage};
use crate::plugin::transport::NetworkTransport;
use crate::resources::PlayerAccount;
use bevy::app::RunFixedMainLoop;
use bevy::prelude::*;
use gyra_codec::error::CodecError;
use gyra_codec::packet::{Packet, When};
use gyra_proto::network::{put, PlayerLook, Proto, SendChatMessage};

pub mod transport;

#[derive(Event)]
pub struct ChangedState {
    pub to: When,
}

#[derive(Event)]
pub struct UploadPacket {
    pub packet: Proto,
}

#[derive(Event)]
enum DownloadInfo {
    Packet(Proto),
    LoginRequest,
}

#[derive(Event, Debug)]
pub struct ErrorFound {
    pub why: String,
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChangedState>()
            .add_event::<UploadPacket>()
            .add_event::<DownloadInfo>()
            .add_event::<ErrorFound>()
            .add_event::<ServerMessage>()
            .add_event::<ClientMessage>()
            .add_systems(
                FixedUpdate,
                receive_packets.run_if(resource_exists::<NetworkTransport>),
            )
            .add_systems(
                FixedUpdate,
                handle_client_messages.run_if(resource_exists::<NetworkTransport>),
            )
            .add_systems(
                FixedUpdate,
                packet_handler
                    .run_if(resource_exists::<NetworkTransport>)
                    .after(receive_packets),
            )
            .add_systems(
                FixedUpdate,
                packet_writer
                    .run_if(resource_exists::<NetworkTransport>)
                    .after(packet_handler),
            );
    }
}

fn receive_packets(mut world: ResMut<NetworkTransport>, mut tx: EventWriter<DownloadInfo>) {
    match world.state {
        When::Handshake => {
            info!("Requesting login!");
            tx.send(DownloadInfo::LoginRequest);
        }

        When::Login | When::Play => {
            let mut used = 0;
            for i in 0..200 {
                used = i;
                match world.poll_packet() {
                    Ok(packet) => {
                        tx.send(DownloadInfo::Packet(packet));
                    }

                    Err(Error::Io(e)) | Err(Error::Codec(CodecError::Io(e)))
                        if e.kind() == std::io::ErrorKind::WouldBlock =>
                    {
                        break;
                    }

                    e => {
                        log::error!("Error receiving packet: {e:?}");
                        break;
                    }
                }
            }

            if used == 200 {
                log::warn!(
                    "Server handler is overloaded. Waiting for next update to receive new packets!"
                );
            }
        }
        When::Status => {
            unreachable!("Status packets should not be received");
        }
    }
}

fn packet_handler(
    mut world: ResMut<NetworkTransport>,
    mut changed_state_writer: EventWriter<ChangedState>,
    // mut error_writer: EventWriter<ErrorFound>,
    mut player_account: Res<PlayerAccount>,
    mut rx: EventReader<DownloadInfo>,
    mut tx: EventWriter<UploadPacket>,
    mut server_message_writer: EventWriter<ServerMessage>,
) {
    for info in rx.read() {
        match info {
            DownloadInfo::LoginRequest => {
                world.state = When::Login;
                changed_state_writer.send(ChangedState { to: When::Login });

                let username = player_account.username.clone();
                world.login(username).unwrap();
            }

            DownloadInfo::Packet(packet) => {
                match packet {
                    Proto::ChatMessage(msg) => {
                        info!("Received {msg:?}");
                        server_message_writer.send(ServerMessage::ChatMessage {
                            message: msg.content.clone(),
                        });
                    }

                    Proto::LoginDisconnect(dis) => {
                        info!("Received {dis:?}");
                        server_message_writer.send(ServerMessage::DisconnectedOnLogin {
                            why: dis.reason.clone(),
                        });
                    }

                    Proto::Disconnect(dis) => {
                        info!("Received {dis:?}");
                        server_message_writer.send(ServerMessage::Disconnected {
                            why: dis.reason.clone(),
                        });
                    }

                    Proto::LoginSuccess(packet) => {
                        info!("Received {packet:?}");
                        world.state = When::Play;
                        changed_state_writer.send(ChangedState { to: When::Play });
                    }

                    Proto::SetCompression(packet) => {
                        info!("Received SetCompression packet: {packet:?}");
                        world.set_compression_threshold(packet.threshold.into());
                    }

                    Proto::JoinGame(packet) => {
                        info!("Received JoinGame packet: {packet:?}");
                        server_message_writer.send(ServerMessage::GameReady {
                            base: packet.to_owned(),
                        });
                    }

                    Proto::KeepAlive(packet) => {
                        info!("Received KeepAlive packet: {packet:?}");
                        let keep_alive = Proto::KeepAlive(packet.to_owned());
                        tx.send(UploadPacket { packet: keep_alive });
                    }

                    Proto::MapChunkBulk(bulk) => {
                        for metadata in &bulk.chunk_metadata {
                            info!(
                                "Received metadata for x: {}, y: {}",
                                metadata.x * 16,
                                metadata.z * 16
                            );
                        }

                        for section in &bulk.sections {
                            for x in 0..16 {
                                for y in 0..16 {
                                    for z in 0..16 {
                                        let block_id = section.block_id(x, y, z);
                                        if 0 != block_id {
                                            info!("[ChunkData] Block at {x}, {y}, {z} is {block_id}");
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Proto::ChunkData(chunk_data) => {
                        info!(
                            "Received ChunkData packet for x: {}, y: {}",
                            chunk_data.x * 16,
                            chunk_data.z * 16
                        );

                        for section in &chunk_data.sections {
                            for x in 0..16 {
                                for y in 0..16 {
                                    for z in 0..16 {
                                        let block_id = section.block_id(x, y, z);
                                        if 0 != block_id {
                                            info!(
                                                "[ChunkData] Block at {x}, {y}, {z} is {block_id}"
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Proto::Disconnect(packet) => {
                    //     info!("Received Disconnect packet: {packet:?}");
                    //     error_writer.send(ErrorFound { why: packet.reason });
                    // }
                    _ => {
                        log::warn!("Unexpected packet: {packet:?}");
                    }
                }
            }
        }
    }
}

fn packet_writer(mut world: ResMut<NetworkTransport>, mut packets: EventReader<UploadPacket>) {
    let threshold = world.server_compress_threshold;

    for payload in packets.read() {
        debug!("[Client->Server] Sending packet {:?}", payload.packet);
        let wrote_size = payload.packet.put(&mut world.stream, threshold).unwrap();
        debug!("Wrote {wrote_size} bytes to server!");
    }
}

fn handle_client_messages(
    mut client_reader: EventReader<ClientMessage>,
    mut packet_writer: EventWriter<UploadPacket>,
) {
    for message in client_reader.read() {
        match message {
            ClientMessage::Look {
                yaw,
                pitch,
                on_ground,
            } => {
                let look = Proto::PlayerLook(PlayerLook {
                    yaw: *yaw,
                    pitch: *pitch,
                    on_ground: *on_ground,
                });

                packet_writer.send(UploadPacket { packet: look });
            }

            ClientMessage::ChatMessage { message } => {
                let mut message = message.trim().to_string();

                if message.len() > 100 {
                    log::warn!("Chat message too long! Truncating to 100 characters.");
                    message = message.chars().take(100).collect();
                }

                let chat_message = Proto::SendChatMessage(SendChatMessage {
                    content: message.to_owned(),
                });

                packet_writer.send(UploadPacket {
                    packet: chat_message,
                });
            }
        }
    }
}
