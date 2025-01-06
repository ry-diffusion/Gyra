use crate::handshaking::*;
use crate::login::*;
use crate::play::*;
use crate::status::*;

// generate an enum of all proto
macro_rules! generate {
    ($name:ident: $($packet:ident),*) => {
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
                      Ok($name::$packet(packet))
                  })*

                  (id, when, direction) => Err(gyra_codec::error::CodecError::IllegalPacket(id, when))
              }
          }

          #[allow(unused_variables, unreachable_patterns)]
          #[inline]
          pub fn encode<W: std::io::Write>(&self, writer: &mut W) -> gyra_codec::error::Result<usize> {
                use gyra_codec::coding::Encoder;

                match self {
                    $($name::$packet(packet) => {
                        packet.encode(writer).map_err(Into::into)
                    })*
                }
            }

            #[inline]
            pub fn put(&self, writer: &mut impl std::io::Write, threshold: Option<u32>) -> gyra_codec::error::Result<usize> {
                match self {
                    $($name::$packet(packet) => crate::io::put(writer, packet, threshold),)*
                }
            }
        }
    };
}

generate!(Protocol: Handshake, LoginStart, LoginSuccess, StatusResponse, SetCompression, 
            ClientKeepAlive, ServerKeepAlive, JoinGame, ChunkData,
            PlayerPositionAndLook, ClientPluginMessage);
