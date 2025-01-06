pub mod handshaking;
pub mod io;
pub mod login;
pub mod nbt;
pub mod pallete_delta;
pub mod play;
pub mod protocol;
pub mod status;

pub const PROTOCOL_VERSION: i32 = 754;
pub const SERVER_VERSION: &str = "1.16.5";

pub use protocol::Protocol;
