use crate::smp;
use gyra_codec::coding::{Decoder, Encoder};
use gyra_codec::variadic_int::VarInt;
use gyra_macros::{packet, CodecDecode, CodecEncode};
use std::collections::HashSet;

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
    // pub chunk_metadata: Vec<ChunkMetadata>,
    // pub sections: Vec<smp::ChunkSection>,
    pub columns: Vec<smp::ChunkColumn>,
}

impl Decoder for MapChunkBulk {
    fn decode<R: std::io::Read>(reader: &mut R) -> gyra_codec::error::Result<Self> {
        let is_overworld = bool::decode(reader)?;
        let chunk_column_sent = VarInt::decode(reader)?.0;

        let mut metadata = Vec::with_capacity(chunk_column_sent as usize);
        let mut columns = Vec::with_capacity(chunk_column_sent as usize);

        for _ in 0..chunk_column_sent {
            let x = i32::decode(reader)?;
            let z = i32::decode(reader)?;
            let primary_bit_mask = u16::decode(reader)?;

            metadata.push(ChunkMetadata {
                x,
                z,
                primary_bit_mask,
            });
        }

        log::info!("Decoding {} chunk columns", chunk_column_sent);

        for i in 0..chunk_column_sent {
            let metadata = &metadata[i as usize];
            let bitmask = metadata.primary_bit_mask;

            let mut column = smp::ChunkColumn {
                sections: [const { None }; 16],
                biomes: [0; 256],
                x: metadata.x,
                z: metadata.z,
            };

            for i in 0..=15 {
                if 0 != (bitmask & (1 << i)) {
                    log::info!(
                        "Decoding section for x: {}, z: {}, y: {} at {i}",
                        metadata.x * 16,
                        metadata.z * 16,
                        i << 4,
                    );

                    let resp = smp::ChunkSection::decode(reader)?;

                    column.sections[i as usize] = Some(resp);
                }
            }

            // TODO: Use this
            let mut biome = [0; 256];
            reader.read_exact(&mut biome)?;

            columns.push(column);
        }

        Ok(Self {
            sky_light_sent: is_overworld,
            chunk_column_sent: VarInt(chunk_column_sent),
            columns,
        })
    }
}

impl Encoder for MapChunkBulk {
    fn encode<W: std::io::Write>(&self, _writer: &mut W) -> gyra_codec::error::Result<usize> {
        unreachable!("MapChunkData is not a packet that should be sent by the client")
    }
}
