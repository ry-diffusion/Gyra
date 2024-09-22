use gyra_codec::coding::{Decoder, Encoder};
use gyra_codec::variadic_int::VarInt;
use gyra_macros::{packet, CodecDecode, CodecEncode};
use crate::smp;

#[derive(Clone, Debug,  PartialEq)]
#[packet(id: 0x21, when: Play)]
pub struct ChunkData {
    pub x: i32,
    pub z: i32,
    pub full_chunk: bool,
    pub primary_bit_mask: u16,
    pub chunk_size: VarInt,
    pub sections: Vec<smp::ChunkSection>,
}

impl Decoder for ChunkData {
    fn decode<R: std::io::Read>(reader: &mut R) -> gyra_codec::error::Result<Self> {
        log::info!("Decoding ChunkData");
        let x = i32::decode(reader)?;
        let z = i32::decode(reader)?;
        let full_chunk = bool::decode(reader)?;
        let primary_bit_mask = u16::decode(reader)?;
        let chunk_size = VarInt::decode(reader)?;
        
        let mut sections = Vec::new();
        
        for i in 0..16 {
            if primary_bit_mask & (1 << i) != 0 {
                sections.push(smp::ChunkSection::decode(reader)?);
            }
        }
        
        Ok(Self {
            x,
            z,
            full_chunk,
            primary_bit_mask,
            chunk_size,
            sections,
        })
    }
}

impl Encoder for ChunkData {
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> gyra_codec::error::Result<usize> {
        unreachable!("ChunkData is not a packet that should be sent by the client")
    }
}