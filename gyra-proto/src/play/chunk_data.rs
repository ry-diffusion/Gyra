use gyra_codec::{
    array::Array,
    coding::{Decoder, Encoder},
    nbt::Nbt,
    variadic_int::VarInt,
};
use gyra_macros::packet;

use crate::nbt::heightmaps::Heightmaps;

#[derive(Debug, PartialEq)]
#[packet(id: 0x20, when: Play, client)]
pub struct ChunkData {
    // Chunk coordinate (block coordinate divided by 16, rounded down).
    pub chunk_x: i32,
    /// Chunk coordinate (block coordinate divided by 16, rounded down).
    pub chunk_z: i32,

    pub full_chunk: bool,

    /// Bitmask with bits set to 1 for every 16×16×16 chunk section whose data is included in Data. The least significant bit represents the chunk section at the bottom of the chunk column (from y=0 to y=15).
    pub primary_bit_mask: VarInt,

    pub heightmaps: Nbt<Heightmaps>,

    /// Size of the following array; should always be 1024. Not present if full chunk is false.
    /// 1024 biome IDs, ordered by x then z then y, in 4×4×4 blocks. Not present if full chunk is false. See Chunk Format § Biomes
    pub biomes: Option<Array<VarInt>>,

    /// Size of Data in bytes.
    // pub size: VarInt,

    /// See data structure in Chunk Format
    pub data: Array<u8>,

    pub number_of_block_entities: VarInt,
    pub block_entities: Option<Nbt<fastnbt::Value>>,
}

impl Decoder for ChunkData {
    fn decode<R: std::io::Read>(reader: &mut R) -> gyra_codec::error::Result<Self> {
        let chunk_x =
            i32::decode(reader).map_err(|e| gyra_codec::error::CodecError::CantParseField {
                field: "chunk_x".to_string(),
                source: e.into(),
            })?;

        let chunk_z =
            i32::decode(reader).map_err(|e| gyra_codec::error::CodecError::CantParseField {
                field: "chunk_z".to_string(),
                source: e.into(),
            })?;

        let full_chunk =
            bool::decode(reader).map_err(|e| gyra_codec::error::CodecError::CantParseField {
                field: "full_chunk".to_string(),
                source: e.into(),
            })?;

        let primary_bit_mask =
            VarInt::decode(reader).map_err(|e| gyra_codec::error::CodecError::CantParseField {
                field: "primary_bit_mask".to_string(),
                source: e.into(),
            })?;

        let heightmaps =
            Nbt::decode(reader).map_err(|e| gyra_codec::error::CodecError::CantParseField {
                field: "heightmaps".to_string(),
                source: e.into(),
            })?;

        let biomes = if full_chunk {
            Some(Array::<VarInt>::decode(reader).map_err(|e| {
                gyra_codec::error::CodecError::CantParseField {
                    field: "biomes".to_string(),
                    source: e.into(),
                }
            })?)
        } else {
            None
        };

        let data = Array::<u8>::decode(reader).map_err(|e| {
            gyra_codec::error::CodecError::CantParseField {
                field: "data".to_string(),
                source: e.into(),
            }
        })?;

        let number_of_block_entities =
            VarInt::decode(reader).map_err(|e| gyra_codec::error::CodecError::CantParseField {
                field: "number_of_block_entities".to_string(),
                source: e.into(),
            })?;

        let mut block_entities = None;
        for _ in 0..number_of_block_entities.0 {
            let x = Nbt::<fastnbt::Value>::decode(reader).map_err(|e| {
                gyra_codec::error::CodecError::CantParseField {
                    field: "block_entities".to_string(),
                    source: e.into(),
                }
            })?;

            block_entities = Some(x);
        }

        Ok(ChunkData {
            chunk_x,
            chunk_z,
            full_chunk,
            primary_bit_mask,
            heightmaps,
            biomes,
            data,
            number_of_block_entities,
            block_entities,
        })
    }
}

impl Encoder for ChunkData {
    fn encode<W: std::io::Write>(&self, _writer: &mut W) -> gyra_codec::error::Result<usize> {
        todo!();
    }
}
