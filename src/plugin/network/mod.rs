use crate::error::Error;
use crate::message::{ClientMessage, ServerMessage};
use crate::plugin::transport::NetworkTransport;
use crate::resources::PlayerAccount;
use bevy::log;
use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, IoTaskPool, Task};
use gyra_codec::error::CodecError;
use gyra_codec::packet::When;
use gyra_proto::network::{PlayerLook, PlayerPosition, Proto, SendChatMessage, ServerKeepAlive};
use gyra_proto::smp::ChunkColumn;

pub mod transport;

#[derive(Event)]
pub struct ChangedState {
    pub to: When,
}

#[derive(Event)]
pub struct UploadPacket {
    pub packet: Proto,
}

impl From<Proto> for UploadPacket {
    fn from(packet: Proto) -> Self {
        Self { packet }
    }
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

#[derive(Resource)]
pub struct ProcessTasks {
    pub tasks: Vec<Task<crate::error::Result<Proto>>>,
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
            .insert_resource(ProcessTasks { tasks: Vec::new() })
            .add_systems(
                FixedUpdate,
                (
                    // 1. RECEIVE PACKETS
                    // 2. HANDLE CLIENT MESSAGES
                    // 3. HANDLE PACKET
                    // 4. WRITE PACKET
                    receive_packets,
                    process_packets.after(receive_packets),
                    handle_client_messages.after(process_packets),
                    packet_handler.after(handle_client_messages),
                    packet_writer.after(packet_handler),
                )
                    .run_if(resource_exists::<NetworkTransport>),
            );
    }
}

fn process_packets(
    mut iotasks: ResMut<ProcessTasks>,
    mut tx: EventWriter<DownloadInfo>,
    mut error_writer: EventWriter<ErrorFound>,
) {
    if iotasks.tasks.is_empty() {
        return;
    }

    let mut done = vec![];
    let mut to_remove = vec![];

    debug!("processing {} packets...", iotasks.tasks.len());

    for (idx, task) in iotasks.tasks.iter_mut().enumerate() {
        let status = if task.is_finished() {
            Some(block_on(task))
        } else {
            block_on(poll_once(task))
        };

        match status {
            Some(res) => {
                match res {
                    Ok(packet) => {
                        done.push(packet);
                    }

                    Err(Error::Codec(CodecError::IllegalPacket(pkg, when))) => {
                        // log::warn!("Illegal packet received: {pkg:?} when {when:?}");
                    }

                    Err(e) => {
                        error_writer.send(ErrorFound {
                            why: format!("{e}"),
                        });
                    }
                }

                to_remove.push(idx);
            }

            None => {}
        }
    }

    for idx in to_remove.iter().rev() {
        let _ = iotasks.tasks.remove(*idx);
    }

    if !done.is_empty() {
        log::info!("Processed {} packets.", done.len());
        tx.send_batch(done.into_iter().map(DownloadInfo::Packet));
    }
}

fn receive_packets(
    mut world: ResMut<NetworkTransport>,
    mut error_writer: EventWriter<ErrorFound>,
    mut tx: EventWriter<DownloadInfo>,
    mut iotasks: ResMut<ProcessTasks>,
) {
    match world.state {
        When::Handshake => {
            info!("Requesting login!");
            tx.send(DownloadInfo::LoginRequest);
        }

        When::Login | When::Play => {
            for _ in 0..200 {
                match world.receive_data() {
                    Ok((data, state)) => {
                        let iopool = IoTaskPool::get();
                        let compress_threshould = world.server_compress_threshold;
                        let task = iopool.spawn(async move {
                            NetworkTransport::proccess_packet(data, state, compress_threshould)
                        });

                        iotasks.tasks.push(task);
                    }

                    Err(Error::Io(e)) | Err(Error::Codec(CodecError::Io(e)))
                        if e.kind() == std::io::ErrorKind::WouldBlock =>
                    {
                        break;
                    }

                    Err(e) => {
                        log::error!("Error downloading packet: {e}");
                        error_writer.send(ErrorFound {
                            why: format!("{e}"),
                        });
                        break;
                    }
                }
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
    player_account: Res<PlayerAccount>,
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

                loop {
                    let packet = world.poll_packet().unwrap();

                    match packet {
                        Proto::LoginSuccess(packet) => {
                            info!("Received {packet:?}");
                            world.state = When::Play;
                            changed_state_writer.send(ChangedState { to: When::Play });
                            world.stream.set_nonblocking(true).unwrap();
                            break;
                        }

                        Proto::SetCompression(packet) => {
                            info!("Received SetCompression packet: {packet:?}, changing.");
                            world.set_compression_threshold(packet.threshold.into());
                        }

                        Proto::Disconnect(dis) => {
                            info!("Received {dis:?}");
                            server_message_writer.send(ServerMessage::Disconnected {
                                why: dis.reason.clone(),
                            });
                        }

                        _ => {
                            error!("Unexpected packet received: {packet:?}");
                        }
                    }
                }
            }

            DownloadInfo::Packet(packet) => match packet {
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
                    info!("Received SetCompression packet: {packet:?}, changing.");
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
                    let keep_alive = ServerKeepAlive { id: packet.id }.into();
                    tx.send(UploadPacket { packet: keep_alive });
                }

                Proto::PlayerPositionAndLook(look) => {
                    server_message_writer.send(ServerMessage::PlayerPositionAndLook {
                        position: Vec3::new(look.x as _, look.y as _, look.z as _),
                        yaw: look.yaw,
                        pitch: look.pitch,
                    });
                }

                Proto::MapChunkBulk(bulk) => {
                    let chunks = bulk
                        .columns
                        .clone()
                        .into_iter()
                        .map(|chunk| ServerMessage::NewChunk { chunk });

                    info!("Received {} chunks via Bulk.", chunks.len());

                    server_message_writer.send_batch(chunks);
                }

                Proto::ChunkData(chunk_data) => {
                    info!(
                        "Received ChunkData packet for x: {}, y: {}",
                        chunk_data.x * 16,
                        chunk_data.z * 16
                    );

                    let column = ChunkColumn::from_sections(
                        chunk_data.sections.clone(),
                        chunk_data.primary_bit_mask,
                        chunk_data.x,
                        chunk_data.z,
                    );

                    server_message_writer.send(ServerMessage::NewChunk { chunk: column });
                }
                _ => {
                    warn!("Unexpected packet: {packet:?}");
                }
            },
        }
    }
}

fn packet_writer(
    mut world: ResMut<NetworkTransport>,
    mut error_writer: EventWriter<ErrorFound>,
    mut packets: EventReader<UploadPacket>,
) {
    let threshold = world.server_compress_threshold;

    for payload in packets.read() {
        info!("[Client->Server] Sending packet {:?}", payload.packet);

        match payload.packet.put(&mut world.stream, threshold) {
            Ok(wrote) => debug!("Wrote {wrote} bytes to server!"),
            Err(e) => {
                log::error!("Error sending packet: {e}");
                error_writer.send(ErrorFound {
                    why: format!("{e}"),
                });
            }
        }
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
                let look = PlayerLook {
                    yaw: *yaw,
                    pitch: *pitch,
                    on_ground: *on_ground,
                };

                packet_writer.send(UploadPacket {
                    packet: look.into(),
                });
            }

            ClientMessage::Moved {
                x,
                feet_y,
                z,
                on_ground,
            } => {
                let move_packet: Proto = PlayerPosition {
                    x: *x,
                    feet_y: *feet_y,
                    z: *z,
                    on_ground: *on_ground,
                }
                .into();

                packet_writer.send(move_packet.into());
            }

            ClientMessage::ChatMessage { message } => {
                let mut message = message.trim().to_string();

                if message.len() > 100 {
                    log::warn!("Chat message too long! Truncating to 100 characters.");
                    message = message.chars().take(100).collect();
                }

                let chat_message: Proto = SendChatMessage {
                    content: message.to_owned(),
                }
                .into();

                packet_writer.send(chat_message.into());
            }
        }
    }
}
