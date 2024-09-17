use gyra_codec::variadic_int::VarInt;
use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(Debug, CodecDecode, CodecEncode, PartialEq)]
#[packet(id: 0x00, when: Handshake)]
pub struct Handshake {
    protocol_version: VarInt,
    server_address: String,
    server_port: u16,
    next_state: VarInt,
}

impl Handshake {
    pub fn status_handshake<S: ToString>(server_address: S, server_port: u16) -> Self {
        Self {
            protocol_version: 47.into(),
            server_address: server_address.to_string(),
            next_state: 1.into(),
            server_port,
        }
    }

    pub fn login_handshake<S: ToString>(server_address: S, server_port: u16) -> Self {
        Self {
            protocol_version: 47.into(),
            server_address: server_address.to_string(),
            next_state: 2.into(),
            server_port,
        }
    }
}
