use std::fmt::Display;
use crate::coding::{Decoder, Encoder};

pub type PacketId = u32;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum When {
    Status,
    Login,
    Play,
    Handshake,
}

impl Display for When {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            When::Status => "getting status".to_string(),
            When::Login => "logging in".to_string(),
            When::Play => "playing".to_string(),
            When::Handshake => "handshaking".to_string(),
        };

        write!(f, "{}", str)
    }
}

pub trait Packet: Encoder + Decoder + Sized {
    const ID: PacketId;
    const WHEN: When;

    fn id(&self) -> PacketId;
    fn when(&self) -> When;
}
