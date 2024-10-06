use crate::smp;
use gyra_codec::coding::{Decoder, Encoder};
use gyra_codec::variadic_int::VarInt;
use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(Clone, Debug, PartialEq, CodecDecode, CodecEncode)]
pub struct ChunkMetadata {
    pub x: i32,
    pub z: i32,
    pub primary_bit_mask: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[packet(id: 0x26, when: Play)]
pub struct MapChunkBulk {
    pub sky_light_sent: bool,
    pub chunk_column_sent: VarInt,
    pub chunk_metadata: Vec<ChunkMetadata>,
    pub sections: Vec<smp::ChunkSection>,
}

impl Decoder for MapChunkBulk {
    fn decode<R: std::io::Read>(reader: &mut R) -> gyra_codec::error::Result<Self> {
        let sky_light_sent = bool::decode(reader)?;
        let chunk_column_sent = VarInt::decode(reader)?.0;
        let mut chunk_metadata = Vec::new();
        let mut sections = Vec::new();

        for _ in 0..chunk_column_sent {
            let x = i32::decode(reader)?;
            let z = i32::decode(reader)?;
            let primary_bit_mask = u16::decode(reader)?;
            chunk_metadata.push(ChunkMetadata {
                x,
                z,
                primary_bit_mask,
            });
        }

        for i in 0..chunk_column_sent {
            let metadata = &chunk_metadata[i as usize];
            let bitmask = metadata.primary_bit_mask;
            for i in 0..15 {
                if 0 != (bitmask & (1 << i)) {
                    log::debug!("Bitmask: {}/{i}", bitmask);
                    let resp = smp::ChunkSection::decode(reader)?;
                    log::debug!(
                        "Decoded section for x: {}, z: {}",
                        metadata.x * 16,
                        metadata.z * 16
                    );
                    sections.push(resp);
                }
            }
        }

        Ok(Self {
            sky_light_sent,
            chunk_column_sent: VarInt(chunk_column_sent),
            chunk_metadata,
            sections,
        })
    }
}

impl Encoder for MapChunkBulk {
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> gyra_codec::error::Result<usize> {
        unreachable!("MapChunkData is not a packet that should be sent by the client")
    }
}
