use std::{
    net::TcpStream,
    time::{Duration, Instant},
};

use gyra_codec::{coding::Decoder, variadic_int::VarInt};
use gyra_proto::{
    handshaking::Handshake,
    io::put_uncompressed,
    status::{PingPong, StatusRequest, StatusResponse},
};
use log::{info, trace};

use crate::resolvers::resolve;

#[derive(Debug, Clone)]
pub struct QueryStatus {
    pub latency: u64,
    pub server_info: String,
}

pub fn fetch_status_of(address: impl ToString) -> crate::Result<QueryStatus> {
    let addr = resolve(address)?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(1500))?;

    stream.set_read_timeout(Some(Duration::from_millis(500)))?;
    stream.set_write_timeout(Some(Duration::from_millis(500)))?;

    log::debug!("[Query] Connected to {addr:#?}");

    put_uncompressed(
        &mut stream,
        &Handshake::status_handshake("127.0.0.1", 25565),
    )?;

    put_uncompressed(&mut stream, &StatusRequest)?;

    let packet_size = VarInt::decode(&mut stream)?;
    trace!("[Status:Server->Client] Packet size: {:?}", packet_size);

    let packet_id = VarInt::decode(&mut stream)?;
    trace!("[Status:Server->Client] Packet ID: {:?}", packet_id);

    let status_response = StatusResponse::decode(&mut stream)?;
    info!(
        "[Status:Server->Client] Status response: {:?}",
        status_response
    );

    let now = Instant::now();
    let ping_packet = PingPong::now();
    put_uncompressed(&mut stream, &ping_packet)?;

    let packet_size = VarInt::decode(&mut stream)?;
    trace!("[Status:Server->Client] Packet size: {:?}", packet_size);

    let _packet_id = VarInt::decode(&mut stream)?;

    let packet = PingPong::decode(&mut stream)?;

    info!("[Status:Server->Client] Received pong: {packet:#?}",);

    Ok(QueryStatus {
        latency: now.elapsed().as_millis() as u64,
        server_info: status_response.json_response,
    })
}
