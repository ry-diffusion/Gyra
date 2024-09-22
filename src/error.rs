use gyra_codec::packet::{PacketId, When};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Codec error: {0}")]
    Codec(#[from] gyra_codec::error::CodecError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("NetworkThread Send error")]
    SendError,



    #[error("Poisoned lock: {0}")]
    PoisonedLock(String),

    #[error("Network World is not initialized")]
    NetworkWorldNotInitialized,

    #[error("Disconnected from server")]
    DisconnectChannel,

    #[error("Unable to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Unable to parse TOML: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("Unable to serialize TOML: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),
}

// for any SendError in Result<T>
impl<T> From<crossbeam_channel::SendError<T>> for Error {
    fn from(_: crossbeam_channel::SendError<T>) -> Self {
        Error::SendError
    }
}

pub type Result<T> = std::result::Result<T, Error>;
