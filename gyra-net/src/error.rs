use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Codec error: {0}")]
    Codec(#[from] gyra_codec::error::CodecError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unable to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Unexpected packet")]
    UnexpectedPacket,
}

impl Error {
    pub fn is_eagain(&self) -> bool {
        match self {
            Error::Io(e) => e.kind() == std::io::ErrorKind::WouldBlock,
            Error::Codec(e) if e.is_eagain() => true,
            _ => false,
        }
    }
}
