use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Net error: {0}")]
    Net(#[from] gyra_net::error::Error),

    #[error("Codec error: {0}")]
    Codec(#[from] gyra_net::codec::error::CodecError),
    

    
    #[error("Runtime Error: {0}")]
    Custom(String),
}

impl Error {
    pub fn custom<T>(msg: impl ToString) -> Result<T, Self> {
        Err(Error::Custom(msg.to_string()))
    }
    
    pub fn is_eagain(&self) -> bool {
        match self {
            Error::Io(e) => e.kind() == std::io::ErrorKind::WouldBlock,
            Error::Net(e) if e.is_eagain() => e.is_eagain(),
            _ => false,
        }
    }
}
