use std::{
    io::{self, Cursor, Read},
    net::{SocketAddr, TcpStream},
};

use crate::{
    codec::{coding::Decoder, packet::When, variadic_int::VarInt},
    error,
    proto::{Protocol, handshaking::Handshake, io::put_uncompressed, login::LoginStart},
    resolvers::resolve,
};
use flate2::read::ZlibDecoder;
use gyra_codec::packet::Direction;
use log::{debug, info};

type NetResult<T> = Result<T, crate::error::Error>;

pub struct Transport {
    pub stream: TcpStream,
    pub addr: SocketAddr,
    pub server_compress_threshold: Option<u32>,
    pub state: When,
}

impl Transport {
    pub fn login(&mut self, username: String) -> NetResult<()> {
        let host = self.addr.ip().to_string();
        let port = self.addr.port();

        put_uncompressed(&mut self.stream, &Handshake::login_handshake(host, port))?;

        info!("Logging in");
        put_uncompressed(&mut self.stream, &LoginStart { username })?;

        self.state = When::Login;

        Ok(())
    }

    pub fn connect(addr: impl ToString) -> io::Result<Self> {
        let address = resolve(addr)?;
        let stream = TcpStream::connect(address)?;
        let addr = stream.peer_addr()?;

        Ok(Transport {
            stream,
            addr,
            server_compress_threshold: None,
            state: When::Handshake,
        })
    }

    fn poll_uncompressed_packet(
        cursor: &mut impl Read,
        state: When,
    ) -> Result<Protocol, error::Error> {
        let packet_id = VarInt::decode(cursor)?.0;

        debug!("Received packet id: 0x{packet_id:02X?}");

        Protocol::decode(packet_id as _, state, Direction::ToClient, cursor).map_err(Into::into)
    }

    fn poll_compressed_packet(cursor: &mut impl Read, state: When) -> crate::Result<Protocol> {
        let uncompressed_size = VarInt::decode(cursor)?.0;

        if 0 == uncompressed_size {
            debug!("Received an ambiguous packet of length 0");
            return Self::poll_uncompressed_packet(cursor, state);
        }

        debug!("Received compressed packet of length: {uncompressed_size}");

        let mut decoder = ZlibDecoder::new(cursor);

        Self::poll_uncompressed_packet(&mut decoder, state)
    }

    pub fn receive_data(&mut self) -> crate::Result<(Vec<u8>, When)> {
        let length = VarInt::decode(&mut self.stream)?.0;
        debug!("Received packet of length: {length:?}");

        let mut buff = vec![0; length as usize];
        self.stream.read_exact(&mut buff)?;

        Ok((buff, self.state))
    }

    pub fn proccess_packet(
        buff: Vec<u8>,
        state: When,
        compress_threshould: Option<u32>,
    ) -> crate::Result<Protocol> {
        let mut cursor = Cursor::new(buff);

        match compress_threshould {
            Some(_) => Self::poll_compressed_packet(&mut cursor, state),
            _ => Self::poll_uncompressed_packet(&mut cursor, state),
        }
    }

    pub fn poll_packet(&mut self) -> crate::Result<Protocol> {
        let (buff, state) = self.receive_data()?;
        Self::proccess_packet(buff, state, self.server_compress_threshold)
    }
}
