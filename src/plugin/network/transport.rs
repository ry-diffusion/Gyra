use crate::error;
use crate::net::resolve;
use bevy::log::{debug, info};
use bevy::prelude::Resource;
use flate2::read::ZlibDecoder;
use gyra_codec::coding::Decoder;
use gyra_codec::packet::{Direction, When};
use gyra_codec::variadic_int::VarInt;
use gyra_proto::network::put_uncompressed;
use gyra_proto::network::{Handshake, LoginStart, Proto};
use std::io::{self, Cursor, Read};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub struct TrackedReader<R: Read> {
    reader: R,
    bytes_read: u64,
}

impl<R: Read> TrackedReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            bytes_read: 0,
        }
    }
}

impl<R: Read> Read for TrackedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.reader.read(buf)?;
        self.bytes_read += bytes_read as u64;
        Ok(bytes_read)
    }
}

#[derive(Debug, Resource)]
pub struct NetworkTransport {
    pub stream: TcpStream,
    pub addr: SocketAddr,
    pub server_compress_threshold: Option<u32>,
    pub state: When,
}

impl NetworkTransport {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> Self {
        Self {
            stream,
            addr,
            state: When::Handshake,
            server_compress_threshold: None,
        }
    }

    pub fn login(&mut self, username: String) -> error::Result<()> {
        let host = self.addr.ip().to_string();
        let port = self.addr.port();

        put_uncompressed(&mut self.stream, &Handshake::login_handshake(host, port))?;

        info!("Logging in");
        put_uncompressed(&mut self.stream, &LoginStart { username })?;

        Ok(())
    }

    pub fn set_compression_threshold(&mut self, threshold: u32) {
        info!("Setting compression threshold to {threshold}");
        self.server_compress_threshold.replace(threshold);
    }

    fn poll_uncompressed_packet(cursor: &mut impl Read, state: When) -> error::Result<Proto> {
        let packet_id = VarInt::decode(cursor)?.0;

        debug!("Received packet id: 0x{packet_id:02X?}");

        Proto::decode(packet_id as _, state, Direction::ToClient, cursor).map_err(Into::into)
    }

    fn poll_compressed_packet(cursor: &mut impl Read, state: When) -> error::Result<Proto> {
        let uncompressed_size = VarInt::decode(cursor)?.0;

        if 0 == uncompressed_size {
            debug!("Received uncompressed packet of length: 0");
            return Self::poll_uncompressed_packet(cursor, state);
        }

        debug!("Received compressed packet of length: {uncompressed_size}");

        let mut decoder = ZlibDecoder::new(cursor);

        Self::poll_uncompressed_packet(&mut decoder, state)
    }

    pub fn poll_packet(&mut self) -> error::Result<Proto> {
        let length = VarInt::decode(&mut self.stream)?.0;
        debug!("Received packet of length: {length:?}");

        let mut buff = vec![0; length as usize];
        self.stream.read_exact(&mut buff)?;

        let mut cursor = Cursor::new(buff);

        match self.server_compress_threshold {
            Some(_) => Self::poll_compressed_packet(&mut cursor, self.state),
            _ => Self::poll_uncompressed_packet(&mut cursor, self.state),
        }
    }

    pub fn connect(address: impl ToString) -> io::Result<Self> {
        let address = address.to_string();
        let addr = resolve(address.as_str())?;

        info!("Connecting to server at: {addr:#?}");

        let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(1500))?;

        // stream.set_nonblocking(true)?;
        // stream.set_nodelay(true)?;

        let addr = stream.peer_addr()?;

        Ok(Self::new(stream, addr))
    }
}
