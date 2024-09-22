use thiserror::Error;
use crate::packet::{PacketId, When};

#[derive(Error, Debug)]
pub enum VarIntError {
    #[error("VarInt is too big")]
    TooBig,
}

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("VarInt error: {0}")]
    VarInt(#[from] VarIntError),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Can't parse field {field}: {source}")]
    CantParseField {
        field: String,

        source: Box<CodecError>,
    },

    #[error("Illegal packet: 0x{0:02X} on {1:?}")]
    IllegalPacket(PacketId, When),
}

pub type Result<T> = std::result::Result<T, CodecError>;