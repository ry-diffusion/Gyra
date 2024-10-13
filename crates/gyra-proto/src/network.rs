pub use crate::handshake::*;
pub use crate::login::{Disconnect as LoginDisconnect, LoginStart, LoginSuccess, SetCompression};
pub use crate::play::*;
pub use crate::status::*;

use flate2::write::ZlibEncoder;
use gyra_codec::coding::Encoder;
use gyra_codec::packet::Packet;
use gyra_codec::variadic_int::VarInt;
use log::{debug, trace};
use std::io::Write;

pub fn put_uncompressed<P: Packet>(
    writer: &mut impl Write,
    packet: &P,
) -> gyra_codec::error::Result<usize> {
    let mut data_buffer = Vec::new();
    let mut packet_buffer = Vec::new();

    /* Packet ID */
    VarInt::from(P::ID).encode(&mut data_buffer)?;
    packet.encode(&mut data_buffer)?;

    debug!(
        "Sending uncompressed packet of length: {} bytes.",
        data_buffer.len()
    );

    /* Packet Length */
    VarInt::from(data_buffer.len() as i32).encode(&mut packet_buffer)?;
    packet_buffer.append(&mut data_buffer);

    writer.write_all(&packet_buffer)?;

    trace!("[Client->Server] Packet data: {:02X?}", packet_buffer);

    Ok(packet_buffer.len())
}

// Sends a uncompressed packet as a compressed packet
// So the data length is ZERO.
// This is used when the packet is smaller than the threshold
// It doesn´t need to compress the data via Zlib
// IDK Why minecraft did that.. But it´s how it works, nothing to do about it
pub fn put_compressed_uncompressed<P: Packet>(
    writer: &mut impl Write,
    packet: &P,
) -> gyra_codec::error::Result<usize> {
    let mut packet_buff = vec![];
    let mut staging_buff = vec![];

    /* Packet ID */
    VarInt::from(P::ID).encode(&mut packet_buff)?;

    /* Packet Data */
    packet.encode(&mut packet_buff)?;

    // 1. Packet Length
    VarInt::from(packet_buff.len() as i32 + 1).encode(&mut staging_buff)?;

    // 2. Data Length
    VarInt::from(0).encode(&mut staging_buff)?;

    // 3. Packet Data
    staging_buff.append(&mut packet_buff);

    debug!(
        "Sending uncompressed packet of length: {} bytes.",
        packet_buff.len()
    );

    writer.write_all(&staging_buff)?;

    Ok(packet_buff.len())
}

pub fn put_compressed<P: Packet>(
    writer: &mut impl Write,
    packet: &P,
    threshold: u32,
) -> gyra_codec::error::Result<usize> {
    let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    let mut uncompressed_size = 0;
    let mut packet_buffer = Vec::new();
    let mut data_buffer = Vec::new();

    /* Packet ID */
    uncompressed_size += VarInt::from(P::ID).encode(&mut encoder)?;
    uncompressed_size += packet.encode(&mut encoder)?;

    if (uncompressed_size as u32) < threshold {
        debug!(
            "Packet size is less than threshold, sending {uncompressed_size} bytes uncompressed"
        );
        return put_compressed_uncompressed(writer, packet);
    }

    debug!("Packet size is greater than threshold, sending {uncompressed_size} bytes compressed.");

    let compressed_data = encoder.finish()?;
    let mut packet_size = compressed_data.len();

    /* FORMAT:
     * packet length
     * uncompressed length
     * compressed data *  */

    packet_size += VarInt::from(uncompressed_size as i32).encode(&mut data_buffer)?;

    VarInt::from(packet_size as i32).encode(&mut packet_buffer)?;
    packet_buffer.append(&mut data_buffer);

    writer.write_all(&packet_buffer)?;

    Ok(packet_buffer.len())
}

pub fn put<P: Packet>(
    writer: &mut impl Write,
    packet: &P,
    threshold: Option<u32>,
) -> gyra_codec::error::Result<usize> {
    debug!(
        "[Client->Server] Sending packet with ID: 0x{:02X}/{:?}",
        P::ID,
        P::WHEN
    );

    if let Some(threshold) = threshold {
        put_compressed(writer, packet, threshold)
    } else {
        put_uncompressed(writer, packet)
    }
}

// generate an enum of all proto
macro_rules! mk_proto {
    ($name:ident
       => $($packet:ident),*) => {
        #[derive(Debug, PartialEq)]
        pub enum $name {
            $($packet($packet)),*
        }

        impl $name {
          #[inline]
          pub fn decode(packet_id: gyra_codec::packet::PacketId, when: gyra_codec::packet::When, direction: gyra_codec::packet::Direction, reader: &mut impl std::io::Read) -> gyra_codec::error::Result<Self> {
              use gyra_codec::coding::Decoder;
              use gyra_codec::packet::Packet;

              #[allow(unused_variables, unreachable_patterns)]
              match (packet_id, when, direction) {
                  $(($packet::ID, $packet::WHEN, $packet::DIRECTION) => {
                      let packet = $packet::decode(reader)?;
                      Ok(Proto::$packet(packet))
                  })*

                  (id, when, direction) => Err(gyra_codec::error::CodecError::IllegalPacket(id, when))
              }
          }

          #[allow(unused_variables, unreachable_patterns)]
          #[inline]
          pub fn encode<W: std::io::Write>(&self, writer: &mut W) -> gyra_codec::error::Result<usize> {
                use gyra_codec::coding::Encoder;

                match self {
                    $(Proto::$packet(packet) => {
                        packet.encode(writer).map_err(Into::into)
                    })*
                }
            }

            #[inline]
            pub fn put(&self, writer: &mut impl std::io::Write, threshold: Option<u32>) -> gyra_codec::error::Result<usize> {
                match self {
                    $(Proto::$packet(packet) => crate::network::put(writer, packet, threshold),)*
                }
            }
        }

       $(impl From<$packet> for $name {
            fn from(packet: $packet) -> Self {
                $name::$packet(packet)
            }
        })*
    };
}

mk_proto!(Proto => PingPong, StatusRequest, StatusResponse, Handshake,
    LoginStart, LoginSuccess, ServerKeepAlive, SetCompression, KeepAlive, JoinGame, ChatMessage,
    EntityRelativeMove, Entity, Disconnect, LoginDisconnect, SendChatMessage, PlayerLook,
    ChunkData, MapChunkBulk, PlayerPositionAndLook, PlayerPosition);
