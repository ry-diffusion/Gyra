// The Minecraft SMP Format is a format used by Minecraft to store the world data.

use gyra_codec::coding::{Decoder, Encoder};
use gyra_codec::nibble::NibbleArray;
use std::io::Read;
use crate::play::ChunkMetadata;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NetworkBlock {
    id: u16,      // Block ID (higher 12 bits)
    metadata: u8, // Metadata (lower 4 bits)
}

impl NetworkBlock {
    fn from_u16(num: u16) -> Self {
        let id = (num >> 4) as u16; // Extract the higher 12 bits as the block ID
        let metadata = (num & 0xF) as u8; // Extract the lower 4 bits as the metadata
        NetworkBlock { id, metadata }
    }

    fn to_u16(&self) -> u16 {
        ((self.id as u16) << 4) | (self.metadata as u16 & 0xF)
    }
}

const WIDTH: usize = 16;
const HEIGHT: usize = 16;
const Z_SIZE: usize = 16;

const ARRAY_SIZE: usize = WIDTH * HEIGHT * Z_SIZE;

/**
 * A chunk section is a 16x16x16 block of the world.
 * It contains the blocks, the skylight and the blocklight.
 */
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkSection {
    // blockID (4) : (4) nibble(metadata)
    pub blocks: Vec<NetworkBlock>,
    pub skylight: NibbleArray,
    pub blocklight: NibbleArray,
    pub count: u32,
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self {
            blocks: vec![NetworkBlock::default(); ARRAY_SIZE],
            skylight: NibbleArray::from_bytes(vec![0; ARRAY_SIZE / 2]),
            blocklight: NibbleArray::from_bytes(vec![0; ARRAY_SIZE / 2]),
            count: 0,
        }
    }
}

impl ChunkSection {
    fn new(blocks: Vec<NetworkBlock>, skylight: NibbleArray, blocklight: NibbleArray) -> Self {
        let count = blocks.iter().filter(|&b| b.id != 0).count() as u32;
        Self {
            blocks,
            skylight,
            blocklight,
            count,
        }
    }

    fn recount(&mut self) {
        self.count = self.blocks.iter().filter(|&b| b.id != 0).count() as u32;
    }

    fn index(x: u32, y: u32, z: u32) -> u32 {
        ((y & 0xf) << 8) | (z << 4) | x
    }

    pub fn metadata(&self, x: u32, y: u32, z: u32) -> u8 {
        self.blocks[ChunkSection::index(x, y, z) as usize].metadata
    }

    pub fn block_id(&self, x: u32, y: u32, z: u32) -> u16 {
        let index = ChunkSection::index(x, y, z) as usize;
        if index >= self.blocks.len() {
            return 0; // AIR
        }
        self.blocks[index].id
    }
}

impl Decoder for ChunkSection {
    fn decode<R: Read>(reader: &mut R) -> gyra_codec::error::Result<Self> {
        let mut blocks = vec![];

        for i in 0..ARRAY_SIZE {
            let mut buff = [0; 2];
            reader.read_exact(&mut buff)?;

            let num = u16::from_le_bytes(buff);
            let block = NetworkBlock::from_u16(num);

            if block.id == 4095 && 0 != i {
                log::warn!(
                    "ChunkSection::decode: block.id == 4095, invalid block id, truncating blocks."
                );

                blocks.truncate(i);
                break;
            }

            blocks.push(block)
        }

        let mut blocklight = vec![0; ARRAY_SIZE / 2];
        reader.read_exact(&mut blocklight)?;

        let mut skylight = vec![0; ARRAY_SIZE / 2];
        reader.read_exact(&mut skylight)?;

        Ok(Self::new(
            blocks,
            NibbleArray::from_bytes(skylight),
            NibbleArray::from_bytes(blocklight),
        ))
    }
}

impl Encoder for ChunkSection {
    fn encode<W: std::io::Write>(&self, _writer: &mut W) -> gyra_codec::error::Result<usize> {
        unreachable!("ChunkSection::encode is not a server side operation.")
    }
}

/**
* A chunk is a 16x16x256 block of the world.
* It contains the sections and the biomes.
* Did you know that the world height is limit is caused because of the chunk section counter
* is a nibble? So it only can store 16 values.
*/
#[derive(Debug, Clone)]
pub struct Chunk {
    pub sections: [Option<ChunkSection>; 15],
    pub biomes: Vec<u8>,
}

impl Chunk {
    pub fn from_sections_metadata(
        mut sections: Vec<ChunkSection>,
        metadata: Vec<ChunkMetadata>,
        column_sent: i32,
    ) -> Self {
        let mut chunk = Chunk {
            sections: [const { None }; 15],
            biomes: vec![0; 256],
        };
    
        for i in 0..column_sent {
            let metadata = &metadata[i as usize];
            let bitmask = metadata.primary_bit_mask;
            for i in 0..15 {
                if 0 != (bitmask & (1 << i)) {
                    let section = sections.remove(0);
                    chunk.sections[i as usize] = Some(section);
                }
            }
        }
        
        chunk
    }

    fn get_block_id(&self, x: u32, y: u32, z: u32) -> Option<u8> {
        let section = &self.sections[(y / 16) as usize];
        Some(section.as_ref()?.block_id(x, y, z) as u8)
    }

    fn get_metadata(&self, x: u32, y: u32, z: u32) -> u8 {
        let section = &self.sections[(y / 16) as usize];
        section.as_ref().map_or(0, |s| s.metadata(x, y, z))
    }
}

pub fn coord_to_index(x: usize, y: usize, z: usize) -> usize {
    assert!(x <= WIDTH, "x is out of bounds");
    assert!(y <= HEIGHT, "y is out of bounds");
    assert!(z <= Z_SIZE, "z is out of bounds");

    (y * HEIGHT + z) * WIDTH + x
}

#[test]
fn parse_end_stone_block() {
    let num = 1936;
    let block = NetworkBlock::from_u16(num);
    assert_eq!(block.id, 121);
    assert_eq!(block.metadata, 0);
}

#[test]
fn parse_example_chunk_section() {
    // a single section full of end stone
    let mut data = vec![];

    for _ in 0..4096 {
        let raw = 1936u16.to_le_bytes();
        data.push(raw[0]);
        data.push(raw[1]);
    }

    for _ in 0..2048 {
        data.push(0);
    }

    for _ in 0..2048 {
        data.push(0);
    }

    let mut reader = std::io::Cursor::new(data);
    let section = ChunkSection::decode(&mut reader).unwrap();

    let id = section.block_id(1, 1, 1);
    assert_eq!(id, 121);
}
