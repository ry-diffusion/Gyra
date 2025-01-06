use std::io::Read;

use gyra_codec::{array::Array, coding::Decoder, variadic_int::VarInt};

///! 1.16 Chunk Data
#[derive(Debug, PartialEq, Clone)]
pub struct PalleteChunk {
    pub primary_bit_mask: u16,
    pub data: Vec<ChunkSection>,
}

impl PalleteChunk {
    pub fn parse(
        reader: &mut impl Read,
        primary_bit_mask: u16,
    ) -> gyra_codec::error::Result<PalleteChunk> {
        let mut chunk = PalleteChunk {
            primary_bit_mask,
            data: Vec::new(),
        };

        for i in 0..16 {
            if primary_bit_mask & (1 << i) != 0 {
                let chunk_section = ChunkSection::decode(reader)?;
                chunk.data.push(chunk_section);
            }
        }

        Ok(chunk)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ChunkSection {
    /// Number of non-air blocks present in the chunk section. "Non-air" is defined as any block other than air, cave air, and void air (in particular, note that fluids such as water are still counted). The client uses this to unload chunks empty chunks. It will keep count of the blocks as they are broken/placed and if this integer reaches 0 the whole chunk section is not rendered, even though there may still be blocks left, if the server sends an incorrect block count.
    pub block_count: i16,
    /// Determines how many bits are used to encode a block. Note that not all numbers are valid here.
    pub bits_per_block: u8,

    pub pallete: Pallete,

    // pub data_array_length: VarInt,
    pub data_state_id_array: Vec<u64>,
}

impl ChunkSection {
    pub fn decode(reader: &mut impl Read) -> gyra_codec::error::Result<ChunkSection> {
        let block_count = i16::decode(reader)?;
        let bits_per_block = u8::decode(reader)?;

        let pallete = match bits_per_block {
            0..=8 => Pallete::parse_indirect(reader, bits_per_block)?,
            _ => Pallete::parse_direct(reader, bits_per_block)?,
        };

        // let data_array_length = VarInt::decode(reader)?;
        let data = Array::decode(reader)?;

        Ok(ChunkSection {
            block_count,
            bits_per_block,
            pallete,
            // data_array_length,
            data_state_id_array: data.elements,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Pallete {
    /**
     * This format is used for bits per block values greater than or equal to 9. The number of bits used to represent a block are the base 2 logarithm of the number of block states, rounded up. For the current vanilla release, this is 15 bits per block.
     */
    Direct { values: Vec<u64> },

    /**
     * There are two variants of this:
     * For bits per block <= 4, 4 bits are used to represent a block.
     * For bits per block between 5 and 8, the given value is used.
     * This is an actual palette which lists the block states used. Values in the chunk section's data array are indices into the palette, which in turn gives a proper block state.
     */
    Indirect {
        bits_per_block: u8,
        values: Vec<VarInt>,
    },
}

impl Pallete {
    pub fn parse_indirect(
        reader: &mut impl Read,
        bits_per_block: u8,
    ) -> gyra_codec::error::Result<Pallete> {
        let count = VarInt::decode(reader)?.0 as usize;
        let mut values = vec![];

        log::debug!(">>[parse indirect] count: {count}");
        for _ in 0..count {
            values.push(VarInt::decode(reader)?);
        }

        Ok(Pallete::Indirect {
            bits_per_block,
            values,
        })
    }

    pub fn parse_direct(
        reader: &mut impl Read,
        bits_per_block: u8,
    ) -> gyra_codec::error::Result<Pallete> {
        let values = Array::decode(reader)?.elements;
        Ok(Pallete::Direct { values })
    }
}
