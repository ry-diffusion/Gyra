pub mod handshake;
mod login;
mod ping_pong;
mod play;
mod status_request;
mod status_response;

pub use handshake::*;
pub use login::*;
pub use ping_pong::*;
pub use status_request::*;
pub use status_response::*;
pub use crate::proto::play::*;

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
          pub fn decode<R: std::io::Read>(packet_id: gyra_codec::packet::PacketId, when: gyra_codec::packet::When, reader: &mut R) -> crate::error::Result<Self> {
              use gyra_codec::coding::Decoder;
              use gyra_codec::packet::Packet;

              #[allow(unused_variables, unreachable_patterns)]
              match (packet_id, when) {
                  $(($packet::ID, $packet::WHEN) => {
                      let packet = $packet::decode(reader)?;
                      Ok(Proto::$packet(packet))
                  })*

                  (id, when) => Err(crate::error::Error::IllegalPacket(id, when))
              }
          }

          #[allow(unused_variables, unreachable_patterns)]
          #[inline]
          pub fn encode<W: std::io::Write>(&self, writer: &mut W) -> crate::error::Result<usize> {
                use gyra_codec::coding::Encoder;
                use gyra_codec::packet::Packet;

                match self {
                    $(Proto::$packet(packet) => {
                        packet.encode(writer).map_err(Into::into)
                    })*
                }
            }

            #[inline]
            pub fn put(&self, writer: &mut impl std::io::Write, threshold: Option<u32>) -> crate::error::Result<()> {
                use crate::net::put;
                use gyra_codec::packet::Packet;

                match self {
                    $(Proto::$packet(packet) => put(writer, packet, threshold),)*
                }
            }
        }
    };
}

mk_proto!(Proto => PingPong, StatusRequest, StatusResponse, Handshake,
    LoginStart, LoginSuccess, SetCompression, KeepAlive, JoinGame, ChatMessage,
    EntityRelativeMove, Entity, Disconnect);
