use crate::net::resolvers::resolve;
use gyra_codec::coding::Decoder;
use gyra_codec::variadic_int::VarInt;
use gyra_proto::network::put_uncompressed;
use gyra_proto::network::{Handshake, PingPong, StatusRequest, StatusResponse};
use bevy::log::{info, trace};
use std::io;
use std::net::TcpStream;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct QueryStatus {
    pub latency: u64,
    pub server_info: String,
}

pub struct QueryClient {
    stream: TcpStream,
}

impl QueryClient {
    pub(crate) fn query_status(mut self) -> crate::error::Result<QueryStatus> {
        let Self { stream } = &mut self;
        stream.set_read_timeout(Some(Duration::from_millis(200)))?;
        stream.set_write_timeout(Some(Duration::from_millis(200)))?;

        trace!("Writing handshake packet");
        put_uncompressed(stream, &Handshake::status_handshake("127.0.0.1", 25565))?;

        trace!("Writing status request packet");
        put_uncompressed(stream, &StatusRequest)?;

        let packet_size = VarInt::decode(stream)?;
        trace!("[Status:Server->Client] Packet size: {:?}", packet_size);

        let packet_id = VarInt::decode(stream)?;
        trace!("[Status:Server->Client] Packet ID: {:?}", packet_id);

        let status_response = StatusResponse::decode(stream)?;
        info!(
            "[Status:Server->Client] Status response: {:?}",
            status_response
        );

        let now = Instant::now();
        let ping_packet = PingPong::now();
        put_uncompressed(stream, &ping_packet)?;

        let packet_size = VarInt::decode(stream)?;
        trace!("[Status:Server->Client] Packet size: {:?}", packet_size);

        let _packet_id = VarInt::decode(stream)?;

        let packet = PingPong::decode(stream)?;

        info!("[Status:Server->Client] Received pong: {packet:#?}",);

        Ok(QueryStatus {
            latency: now.elapsed().as_millis() as u64,
            server_info: status_response.json_response,
        })
    }

    pub fn connect(address: impl ToString) -> io::Result<Self> {
        let addr = resolve(address.to_string().as_str())?;
        info!("Connecting to server at: {addr:#?}");
        let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(500))?;

        Ok(Self { stream })
    }
}
