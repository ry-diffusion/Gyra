mod packet;
pub mod query;
mod resolvers;

use gyra_codec::coding::{Decoder, Encoder};
use gyra_codec::packet::Packet;
use std::io::{Read, Write};
use std::net::ToSocketAddrs;

pub use packet::*;
pub use resolvers::resolve;
